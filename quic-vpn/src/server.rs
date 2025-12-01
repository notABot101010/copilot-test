use anyhow::{Context, Result};
use bytes::Bytes;
use clap::Parser;
use quic_vpn::{ensure_certificates, packet::*, MAX_DATAGRAM_SIZE};
use ring::rand::SecureRandom;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "quic-vpn-server")]
#[command(about = "QUIC VPN Server", long_about = None)]
struct Args {
    /// Listen address
    #[arg(short, long, default_value = "0.0.0.0:4433")]
    listen: String,

    /// Certificate file path
    #[arg(long, default_value = "certs/cert.pem")]
    cert: PathBuf,

    /// Private key file path
    #[arg(long, default_value = "certs/key.pem")]
    key: PathBuf,

    /// TUN device name
    #[arg(long, default_value = "tun0")]
    tun_name: String,

    /// VPN subnet (server will use .1)
    #[arg(long, default_value = "10.8.0.0/24")]
    subnet: String,
}

struct Client {
    conn: quiche::Connection,
    addr: SocketAddr,
}

struct Server {
    socket: Arc<UdpSocket>,
    clients: Arc<Mutex<HashMap<Vec<u8>, Client>>>,
    tun: Arc<tun2::AsyncDevice>,
    config: quiche::Config,
    local_addr: SocketAddr,
}

impl Server {
    fn new(
        socket: UdpSocket,
        tun: tun2::AsyncDevice,
        config: quiche::Config,
        local_addr: SocketAddr,
    ) -> Self {
        Self {
            socket: Arc::new(socket),
            clients: Arc::new(Mutex::new(HashMap::new())),
            tun: Arc::new(tun),
            config,
            local_addr,
        }
    }

    async fn run(&mut self) -> Result<()> {
        let mut recv_buf = vec![0u8; 65535];
        let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];

        // Spawn TUN reader task
        let tun = self.tun.clone();
        let clients = self.clients.clone();
        let socket = self.socket.clone();
        tokio::spawn(async move {
            Self::handle_tun_to_clients(tun, clients, socket).await;
        });

        loop {
            // Receive from UDP socket
            let (len, from) = match self.socket.recv_from(&mut recv_buf).await {
                Ok(v) => v,
                Err(err) => {
                    error!("UDP recv error: {}", err);
                    continue;
                }
            };

            let pkt_buf = &mut recv_buf[..len];

            // Parse QUIC header
            let hdr = match quiche::Header::from_slice(pkt_buf, quiche::MAX_CONN_ID_LEN) {
                Ok(v) => v,
                Err(err) => {
                    warn!("Failed to parse QUIC header: {}", err);
                    continue;
                }
            };

            let conn_id = hdr.dcid.clone();
            let mut clients = self.clients.lock().await;

            // Check if this is from an existing client
            if let Some(client) = clients.get_mut(&conn_id.to_vec()) {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: self.local_addr,
                };

                match client.conn.recv(pkt_buf, recv_info) {
                    Ok(read) => {
                        debug!("Received {} bytes from client {}", read, from);
                    }
                    Err(err) => {
                        error!("QUIC recv error: {}", err);
                        continue;
                    }
                }

                // Process readable streams
                if client.conn.is_established() {
                    let tun = self.tun.clone();
                    for stream_id in client.conn.readable() {
                        if let Err(err) = Self::handle_stream_read(&mut client.conn, stream_id, tun.clone()).await {
                            error!("Stream read error: {}", err);
                        }
                    }
                }

                // Send packets
                loop {
                    match client.conn.send(&mut out_buf) {
                        Ok((written, _send_info)) => {
                            if let Err(err) = self.socket.send_to(&out_buf[..written], from).await {
                                error!("UDP send error: {}", err);
                                break;
                            }
                        }
                        Err(quiche::Error::Done) => break,
                        Err(err) => {
                            error!("QUIC send error: {}", err);
                            break;
                        }
                    }
                }
            } else if hdr.ty == quiche::Type::Initial {
                // New connection
                info!("New connection from {}", from);

                let mut scid = [0u8; quiche::MAX_CONN_ID_LEN];
                ring::rand::SystemRandom::new()
                    .fill(&mut scid)
                    .map_err(|_| anyhow::anyhow!("Failed to generate connection ID"))?;
                let scid = quiche::ConnectionId::from_ref(&scid);

                let mut conn = quiche::accept(&scid, None, self.local_addr, from, &mut self.config)
                    .context("Failed to accept QUIC connection")?;

                let recv_info = quiche::RecvInfo {
                    from,
                    to: self.local_addr,
                };

                conn.recv(pkt_buf, recv_info)?;

                // Send initial packets
                loop {
                    match conn.send(&mut out_buf) {
                        Ok((written, _send_info)) => {
                            if let Err(err) = self.socket.send_to(&out_buf[..written], from).await {
                                error!("UDP send error: {}", err);
                                break;
                            }
                        }
                        Err(quiche::Error::Done) => break,
                        Err(err) => {
                            error!("QUIC send error: {}", err);
                            break;
                        }
                    }
                }

                clients.insert(
                    scid.to_vec(),
                    Client {
                        conn,
                        addr: from,
                    },
                );
            }

            // Clean up closed connections
            clients.retain(|_, client| !client.conn.is_closed());
        }
    }

    async fn handle_stream_read(
        conn: &mut quiche::Connection,
        stream_id: u64,
        tun: Arc<tun2::AsyncDevice>,
    ) -> Result<()> {
        let mut buf = vec![0u8; MAX_DATAGRAM_SIZE];
        match conn.stream_recv(stream_id, &mut buf) {
            Ok((len, _fin)) => {
                let packet = VpnPacket::decode(Bytes::copy_from_slice(&buf[..len]));
                if let Some(pkt) = packet {
                    match pkt.packet_type {
                        PACKET_TYPE_DATA => {
                            // Write to TUN device
                            debug!("Forwarding {} bytes to TUN", pkt.data.len());
                            if let Err(err) = tun.send(&pkt.data).await {
                                error!("Failed to write to TUN: {}", err);
                            }
                        }
                        PACKET_TYPE_PING => {
                            debug!("Received ping");
                            let pong = VpnPacket::new_pong();
                            conn.stream_send(stream_id, &pong.encode(), false).ok();
                        }
                        _ => {
                            warn!("Unknown packet type: {}", pkt.packet_type);
                        }
                    }
                }
            }
            Err(quiche::Error::Done) => {}
            Err(err) => {
                error!("Stream recv error: {}", err);
            }
        }
        Ok(())
    }

    async fn handle_tun_to_clients(
        tun: Arc<tun2::AsyncDevice>,
        clients: Arc<Mutex<HashMap<Vec<u8>, Client>>>,
        socket: Arc<UdpSocket>,
    ) {
        let mut buf = vec![0u8; MAX_DATAGRAM_SIZE];
        let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];

        loop {
            match tun.recv(&mut buf).await {
                Ok(len) => {
                    debug!("Received {} bytes from TUN", len);
                    let packet = VpnPacket::new_data(Bytes::copy_from_slice(&buf[..len]));
                    let encoded = packet.encode();

                    // Send to all connected clients
                    let mut clients_lock = clients.lock().await;
                    for (_id, client) in clients_lock.iter_mut() {
                        if client.conn.is_established() {
                            // Use stream 0 for server-initiated data
                            if let Err(err) = client.conn.stream_send(0, &encoded, false) {
                                warn!("Failed to send to client: {}", err);
                                continue;
                            }

                            // Send QUIC packets
                            loop {
                                match client.conn.send(&mut out_buf) {
                                    Ok((written, _send_info)) => {
                                        if let Err(err) = socket.send_to(&out_buf[..written], client.addr).await {
                                            error!("UDP send error: {}", err);
                                            break;
                                        }
                                    }
                                    Err(quiche::Error::Done) => break,
                                    Err(err) => {
                                        error!("QUIC send error: {}", err);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    error!("TUN recv error: {}", err);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    // Ensure certificates exist
    ensure_certificates(&args.cert, &args.key)?;

    // Create QUIC config
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)?;
    config.load_cert_chain_from_pem_file(
        args.cert.to_str()
            .context("Certificate path contains invalid UTF-8")?
    )?;
    config.load_priv_key_from_pem_file(
        args.key.to_str()
            .context("Private key path contains invalid UTF-8")?
    )?;
    config.set_application_protos(&[b"quic-vpn"])?;
    config.set_max_idle_timeout(30000);
    config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_initial_max_data(10_000_000);
    config.set_initial_max_stream_data_bidi_local(1_000_000);
    config.set_initial_max_stream_data_bidi_remote(1_000_000);
    config.set_initial_max_streams_bidi(100);
    config.set_disable_active_migration(true);

    // Create TUN device
    info!("Creating TUN device: {}", args.tun_name);
    let mut tun_config = tun2::Configuration::default();
    tun_config
        .tun_name(&args.tun_name)
        .address("10.8.0.1")
        .netmask("255.255.255.0")
        .up();

    #[cfg(target_os = "linux")]
    tun_config.platform_config(|config| {
        config.ensure_root_privileges(true);
    });

    let tun = tun2::create_as_async(&tun_config)
        .context("Failed to create TUN device (are you running as root?)")?;

    info!("TUN device created: {}", args.tun_name);

    // Bind UDP socket
    let addr: SocketAddr = args.listen.parse()?;
    let socket = UdpSocket::bind(addr).await?;
    let local_addr = socket.local_addr()?;
    info!("Server listening on {}", local_addr);

    let mut server = Server::new(socket, tun, config, local_addr);

    // Run server
    server.run().await
}
