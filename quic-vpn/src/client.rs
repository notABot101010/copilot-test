use anyhow::{Context, Result};
use bytes::Bytes;
use clap::Parser;
use quic_vpn::{packet::*, MAX_DATAGRAM_SIZE};
use ring::rand::SecureRandom;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::net::UdpSocket;
use tracing::{debug, error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "quic-vpn-client")]
#[command(about = "QUIC VPN Client", long_about = None)]
struct Args {
    /// Server URL (format: hostname:port or IP:port)
    #[arg(short, long)]
    server: String,

    /// TUN device name
    #[arg(long, default_value = "tun1")]
    tun_name: String,

    /// Client IP address on VPN
    #[arg(long, default_value = "10.8.0.2")]
    client_ip: String,

    /// Skip certificate verification (for self-signed certs)
    #[arg(long, default_value = "true")]
    insecure: bool,
}

struct VpnClient {
    conn: quiche::Connection,
    socket: UdpSocket,
    server_addr: SocketAddr,
    tun: tun2::AsyncDevice,
    stream_id: Option<u64>,
    local_addr: SocketAddr,
}

impl VpnClient {
    fn new(
        conn: quiche::Connection,
        socket: UdpSocket,
        server_addr: SocketAddr,
        tun: tun2::AsyncDevice,
        local_addr: SocketAddr,
    ) -> Self {
        Self {
            conn,
            socket,
            server_addr,
            tun,
            stream_id: None,
            local_addr,
        }
    }

    async fn run(&mut self) -> Result<()> {
        let mut recv_buf = vec![0u8; 65535];
        let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];
        let mut tun_buf = vec![0u8; MAX_DATAGRAM_SIZE];

        // Send initial packet
        self.send_packets(&mut out_buf).await?;

        info!("Client running, waiting for connection to establish...");

        loop {
            tokio::select! {
                // Receive from UDP socket
                result = self.socket.recv_from(&mut recv_buf) => {
                    let (len, from) = match result {
                        Ok(v) => v,
                        Err(err) => {
                            error!("UDP recv error: {}", err);
                            continue;
                        }
                    };

                    if from != self.server_addr {
                        warn!("Received packet from unexpected address: {}", from);
                        continue;
                    }

                    let recv_info = quiche::RecvInfo {
                        from,
                        to: self.local_addr,
                    };

                    match self.conn.recv(&mut recv_buf[..len], recv_info) {
                        Ok(read) => {
                            debug!("Received {} bytes from server", read);
                        }
                        Err(err) => {
                            error!("QUIC recv error: {}", err);
                            continue;
                        }
                    }

                    // Process readable streams
                    if self.conn.is_established() {
                        if self.stream_id.is_none() {
                            // Use stream ID 0 for client-initiated bidirectional stream
                            let stream_id = 0u64;
                            info!("VPN connection established!");
                            self.stream_id = Some(stream_id);

                            // Send initial ping
                            let ping = VpnPacket::new_ping();
                            if let Err(err) = self.conn.stream_send(stream_id, &ping.encode(), false) {
                                warn!("Failed to send ping: {}", err);
                            }
                        }

                        for stream_id in self.conn.readable() {
                            self.handle_stream_read(stream_id).await?;
                        }
                    }

                    // Send packets
                    self.send_packets(&mut out_buf).await?;
                }

                // Read from TUN device and forward to server
                result = self.tun.recv(&mut tun_buf) => {
                    match result {
                        Ok(len) => {
                            if self.conn.is_established() {
                                if let Some(stream_id) = self.stream_id {
                                    debug!("Read {} bytes from TUN, forwarding to server", len);

                                    // Wrap packet in VPN protocol
                                    let packet = VpnPacket::new_data(Bytes::copy_from_slice(&tun_buf[..len]));
                                    let encoded = packet.encode();

                                    // Send to server
                                    if let Err(err) = self.conn.stream_send(stream_id, &encoded, false) {
                                        error!("Failed to send to server: {}", err);
                                    }

                                    // Flush packets
                                    self.send_packets(&mut out_buf).await?;
                                }
                            }
                        }
                        Err(err) => {
                            error!("TUN recv error: {}", err);
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    }
                }

                // Handle connection timeout
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                    self.conn.on_timeout();
                    self.send_packets(&mut out_buf).await?;
                }
            }

            // Exit if connection closed
            if self.conn.is_closed() {
                warn!("Connection closed");
                break;
            }
        }

        Ok(())
    }

    async fn handle_stream_read(&mut self, stream_id: u64) -> Result<()> {
        let mut buf = vec![0u8; MAX_DATAGRAM_SIZE];
        match self.conn.stream_recv(stream_id, &mut buf) {
            Ok((len, _fin)) => {
                let packet = VpnPacket::decode(Bytes::copy_from_slice(&buf[..len]));
                if let Some(pkt) = packet {
                    match pkt.packet_type {
                        PACKET_TYPE_DATA => {
                            // Write to TUN device
                            debug!("Forwarding {} bytes to TUN", pkt.data.len());
                            if let Err(err) = self.tun.send(&pkt.data).await {
                                error!("Failed to write to TUN: {}", err);
                            }
                        }
                        PACKET_TYPE_PONG => {
                            debug!("Received pong");
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

    async fn send_packets(&mut self, out_buf: &mut [u8]) -> Result<()> {
        loop {
            match self.conn.send(out_buf) {
                Ok((written, _send_info)) => {
                    if let Err(err) = self
                        .socket
                        .send_to(&out_buf[..written], self.server_addr)
                        .await
                    {
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
        Ok(())
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

    // Resolve server address
    let server_addr = args
        .server
        .to_socket_addrs()?
        .next()
        .context("Failed to resolve server address")?;

    info!("Connecting to server at {}", server_addr);

    // Create QUIC config
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)?;
    config.set_application_protos(&[b"quic-vpn"])?;
    config.set_max_idle_timeout(30000);
    config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_initial_max_data(10_000_000);
    config.set_initial_max_stream_data_bidi_local(1_000_000);
    config.set_initial_max_stream_data_bidi_remote(1_000_000);
    config.set_initial_max_streams_bidi(100);
    config.set_disable_active_migration(true);

    if args.insecure {
        config.verify_peer(false);
    }

    // Create TUN device
    info!("Creating TUN device: {}", args.tun_name);
    let mut tun_config = tun2::Configuration::default();
    tun_config
        .tun_name(&args.tun_name)
        .address(&args.client_ip)
        .netmask("255.255.255.0")
        .destination("10.8.0.1")
        .up();

    #[cfg(target_os = "linux")]
    tun_config.platform_config(|platform_config| {
        platform_config.ensure_root_privileges(true);
    });

    let tun = tun2::create_as_async(&tun_config)
        .context("Failed to create TUN device (are you running as root?)")?;

    info!("TUN device created: {}", args.tun_name);

    // Bind local UDP socket
    let local_addr: SocketAddr = if server_addr.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    }
    .parse()?;

    let socket = UdpSocket::bind(local_addr).await?;
    let local_sock_addr = socket.local_addr()?;
    info!("Client socket bound to {}", local_sock_addr);

    // Generate connection ID
    let mut scid = [0u8; quiche::MAX_CONN_ID_LEN];
    ring::rand::SystemRandom::new()
        .fill(&mut scid)
        .map_err(|_| anyhow::anyhow!("Failed to generate connection ID"))?;
    let scid = quiche::ConnectionId::from_ref(&scid);

    // Create QUIC connection
    let conn = quiche::connect(
        Some("quic-vpn"),
        &scid,
        local_sock_addr,
        server_addr,
        &mut config,
    )?;

    info!("QUIC connection initiated");

    let mut client = VpnClient::new(conn, socket, server_addr, tun, local_sock_addr);

    // Run client
    client.run().await
}
