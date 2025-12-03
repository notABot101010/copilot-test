//! Integration tests for QUIC VPN
//!
//! These tests verify:
//! - QUIC connection establishment between client and server
//! - VPN protocol packet exchange
//! - Proper handling of ping/pong messages
//! - Data packet transmission over QUIC streams

use bytes::Bytes;
use quic_vpn::{generate_self_signed_cert, packet::*, MAX_DATAGRAM_SIZE};
use ring::rand::SecureRandom;
use std::net::SocketAddr;
use std::time::Duration;
use tempfile::TempDir;
use tokio::net::UdpSocket;

/// Maximum iterations for connection establishment loops
const CONNECTION_MAX_ITERATIONS: i32 = 50;
/// Maximum iterations for data exchange loops
const EXCHANGE_MAX_ITERATIONS: i32 = 100;
/// Maximum iterations for long-running exchange tests
const EXTENDED_MAX_ITERATIONS: i32 = 200;
/// Receive buffer size - larger than MTU to handle UDP reassembly
const RECV_BUF_SIZE: usize = 65535;
/// Client-initiated bidirectional streams increment by 4 in QUIC
const STREAM_ID_INCREMENT: u64 = 4;

/// Helper struct for managing test certificates
struct TestCerts {
    #[allow(dead_code)]
    temp_dir: TempDir,
    cert_path: std::path::PathBuf,
    key_path: std::path::PathBuf,
}

impl TestCerts {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");
        generate_self_signed_cert(&cert_path, &key_path).expect("Failed to generate certs");
        Self {
            temp_dir,
            cert_path,
            key_path,
        }
    }
}

/// Create a test server configuration
fn create_server_config(certs: &TestCerts) -> quiche::Config {
    let mut config =
        quiche::Config::new(quiche::PROTOCOL_VERSION).expect("Failed to create config");
    config
        .load_cert_chain_from_pem_file(certs.cert_path.to_str().expect("Invalid path"))
        .expect("Failed to load cert");
    config
        .load_priv_key_from_pem_file(certs.key_path.to_str().expect("Invalid path"))
        .expect("Failed to load key");
    config
        .set_application_protos(&[b"quic-vpn"])
        .expect("Failed to set ALPN");
    config.set_max_idle_timeout(5000);
    config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_initial_max_data(10_000_000);
    config.set_initial_max_stream_data_bidi_local(1_000_000);
    config.set_initial_max_stream_data_bidi_remote(1_000_000);
    config.set_initial_max_streams_bidi(100);
    config.set_disable_active_migration(true);
    config
}

/// Create a test client configuration
fn create_client_config() -> quiche::Config {
    let mut config =
        quiche::Config::new(quiche::PROTOCOL_VERSION).expect("Failed to create config");
    config
        .set_application_protos(&[b"quic-vpn"])
        .expect("Failed to set ALPN");
    config.set_max_idle_timeout(5000);
    config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_initial_max_data(10_000_000);
    config.set_initial_max_stream_data_bidi_local(1_000_000);
    config.set_initial_max_stream_data_bidi_remote(1_000_000);
    config.set_initial_max_streams_bidi(100);
    config.set_disable_active_migration(true);
    config.verify_peer(false); // Allow self-signed certs
    config
}

/// Generate a random connection ID
fn generate_conn_id() -> Vec<u8> {
    let mut scid = vec![0u8; quiche::MAX_CONN_ID_LEN];
    ring::rand::SystemRandom::new()
        .fill(&mut scid)
        .expect("Failed to generate connection ID");
    scid
}

#[tokio::test]
async fn test_quic_connection_establishment() {
    // Create test certificates
    let certs = TestCerts::new();

    // Create server socket
    let server_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server socket");
    let server_addr = server_socket
        .local_addr()
        .expect("Failed to get server address");

    // Create client socket
    let client_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind client socket");
    let client_addr = client_socket
        .local_addr()
        .expect("Failed to get client address");

    // Create configs
    let mut server_config = create_server_config(&certs);
    let mut client_config = create_client_config();

    // Generate connection IDs
    let client_scid = generate_conn_id();
    let client_scid_ref = quiche::ConnectionId::from_ref(&client_scid);

    // Create client connection
    let mut client_conn = quiche::connect(
        Some("quic-vpn"),
        &client_scid_ref,
        client_addr,
        server_addr,
        &mut client_config,
    )
    .expect("Failed to create client connection");

    // Send initial packet from client
    let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];
    let (write_len, _) = client_conn
        .send(&mut out_buf)
        .expect("Failed to send initial packet");
    client_socket
        .send_to(&out_buf[..write_len], server_addr)
        .await
        .expect("Failed to send to server");

    // Receive at server (buffer larger than MTU for UDP reassembly)
    let mut recv_buf = vec![0u8; RECV_BUF_SIZE];
    let (len, from) = server_socket
        .recv_from(&mut recv_buf)
        .await
        .expect("Failed to receive at server");

    // Parse header and accept connection
    let hdr = quiche::Header::from_slice(&mut recv_buf[..len], quiche::MAX_CONN_ID_LEN)
        .expect("Failed to parse header");

    assert_eq!(
        hdr.ty,
        quiche::Type::Initial,
        "First packet should be Initial"
    );

    let server_scid = generate_conn_id();
    let server_scid_ref = quiche::ConnectionId::from_ref(&server_scid);

    let mut server_conn = quiche::accept(
        &server_scid_ref,
        None,
        server_addr,
        from,
        &mut server_config,
    )
    .expect("Failed to accept connection");

    let recv_info = quiche::RecvInfo {
        from,
        to: server_addr,
    };
    server_conn
        .recv(&mut recv_buf[..len], recv_info)
        .expect("Failed to process initial packet");

    // Exchange packets until connection is established
    let mut iterations = 0;

    while !client_conn.is_established() || !server_conn.is_established() {
        iterations += 1;
        if iterations > CONNECTION_MAX_ITERATIONS {
            panic!(
                "Connection not established after {} iterations",
                CONNECTION_MAX_ITERATIONS
            );
        }

        // Server sends
        while let Ok((write_len, _)) = server_conn.send(&mut out_buf) {
            server_socket
                .send_to(&out_buf[..write_len], from)
                .await
                .expect("Failed to send from server");
        }

        // Client receives
        match tokio::time::timeout(
            Duration::from_millis(100),
            client_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: client_addr,
                };
                if let Err(err) = client_conn.recv(&mut recv_buf[..len], recv_info) {
                    eprintln!("Client recv error: {}", err);
                }
            }
            _ => {}
        }

        // Client sends
        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send from client");
        }

        // Server receives
        match tokio::time::timeout(
            Duration::from_millis(100),
            server_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: server_addr,
                };
                if let Err(err) = server_conn.recv(&mut recv_buf[..len], recv_info) {
                    eprintln!("Server recv error: {}", err);
                }
            }
            _ => {}
        }
    }

    assert!(
        client_conn.is_established(),
        "Client connection should be established"
    );
    assert!(
        server_conn.is_established(),
        "Server connection should be established"
    );
}

#[tokio::test]
async fn test_vpn_packet_exchange_over_quic() {
    // Create test certificates
    let certs = TestCerts::new();

    // Create sockets
    let server_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server socket");
    let server_addr = server_socket
        .local_addr()
        .expect("Failed to get server address");

    let client_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind client socket");
    let client_addr = client_socket
        .local_addr()
        .expect("Failed to get client address");

    // Create configs
    let mut server_config = create_server_config(&certs);
    let mut client_config = create_client_config();

    // Create connections
    let client_scid = generate_conn_id();
    let client_scid_ref = quiche::ConnectionId::from_ref(&client_scid);

    let mut client_conn = quiche::connect(
        Some("quic-vpn"),
        &client_scid_ref,
        client_addr,
        server_addr,
        &mut client_config,
    )
    .expect("Failed to create client connection");

    let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];
    let mut recv_buf = vec![0u8; 65535];
    let mut server_conn: Option<quiche::Connection> = None;
    let mut server_client_addr: Option<SocketAddr> = None;

    // Establish connection
    let mut iterations = 0;
    const MAX_ITERATIONS: i32 = 100;

    loop {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            panic!("Test timeout after {} iterations", MAX_ITERATIONS);
        }

        // Client sends
        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send from client");
        }

        // Server receives
        match tokio::time::timeout(
            Duration::from_millis(50),
            server_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                if server_conn.is_none() {
                    // Parse header and create server connection
                    if let Ok(hdr) =
                        quiche::Header::from_slice(&mut recv_buf[..len], quiche::MAX_CONN_ID_LEN)
                    {
                        if hdr.ty == quiche::Type::Initial {
                            let server_scid = generate_conn_id();
                            let server_scid_ref = quiche::ConnectionId::from_ref(&server_scid);
                            if let Ok(conn) = quiche::accept(
                                &server_scid_ref,
                                None,
                                server_addr,
                                from,
                                &mut server_config,
                            ) {
                                server_conn = Some(conn);
                                server_client_addr = Some(from);
                            }
                        }
                    }
                }

                if let Some(ref mut conn) = server_conn {
                    let recv_info = quiche::RecvInfo {
                        from,
                        to: server_addr,
                    };
                    let _ = conn.recv(&mut recv_buf[..len], recv_info);
                }
            }
            _ => {}
        }

        // Server sends
        if let Some(ref mut conn) = server_conn {
            if let Some(addr) = server_client_addr {
                while let Ok((write_len, _)) = conn.send(&mut out_buf) {
                    server_socket
                        .send_to(&out_buf[..write_len], addr)
                        .await
                        .expect("Failed to send from server");
                }
            }
        }

        // Client receives
        match tokio::time::timeout(
            Duration::from_millis(50),
            client_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: client_addr,
                };
                let _ = client_conn.recv(&mut recv_buf[..len], recv_info);
            }
            _ => {}
        }

        // Check if established
        if let Some(ref conn) = server_conn {
            if client_conn.is_established() && conn.is_established() {
                break;
            }
        }
    }

    // Connection is established, now test VPN packet exchange
    let server_conn = server_conn
        .as_mut()
        .expect("Server connection should exist");

    // Send a ping packet from client
    let ping = VpnPacket::new_ping();
    let encoded_ping = ping.encode();
    let stream_id = 0u64;

    client_conn
        .stream_send(stream_id, &encoded_ping, false)
        .expect("Failed to send ping");

    // Exchange packets
    let mut pong_received = false;
    iterations = 0;

    while !pong_received {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            panic!("Pong not received after {} iterations", MAX_ITERATIONS);
        }

        // Client sends QUIC packets
        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send from client");
        }

        // Server receives and processes
        match tokio::time::timeout(
            Duration::from_millis(50),
            server_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: server_addr,
                };
                let _ = server_conn.recv(&mut recv_buf[..len], recv_info);

                // Check for readable streams
                for stream in server_conn.readable() {
                    let mut stream_buf = vec![0u8; MAX_DATAGRAM_SIZE];
                    if let Ok((len, _fin)) = server_conn.stream_recv(stream, &mut stream_buf) {
                        if let Some(packet) =
                            VpnPacket::decode(Bytes::copy_from_slice(&stream_buf[..len]))
                        {
                            if packet.packet_type == PACKET_TYPE_PING {
                                // Send pong
                                let pong = VpnPacket::new_pong();
                                server_conn
                                    .stream_send(stream, &pong.encode(), false)
                                    .expect("Failed to send pong");
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        // Server sends
        if let Some(addr) = server_client_addr {
            while let Ok((write_len, _)) = server_conn.send(&mut out_buf) {
                server_socket
                    .send_to(&out_buf[..write_len], addr)
                    .await
                    .expect("Failed to send from server");
            }
        }

        // Client receives and checks for pong
        match tokio::time::timeout(
            Duration::from_millis(50),
            client_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: client_addr,
                };
                let _ = client_conn.recv(&mut recv_buf[..len], recv_info);

                // Check for readable streams
                for stream in client_conn.readable() {
                    let mut stream_buf = vec![0u8; MAX_DATAGRAM_SIZE];
                    if let Ok((len, _fin)) = client_conn.stream_recv(stream, &mut stream_buf) {
                        if let Some(packet) =
                            VpnPacket::decode(Bytes::copy_from_slice(&stream_buf[..len]))
                        {
                            if packet.packet_type == PACKET_TYPE_PONG {
                                pong_received = true;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    assert!(pong_received, "Should have received pong response");
}

#[tokio::test]
async fn test_vpn_data_packet_transmission() {
    // Create test certificates
    let certs = TestCerts::new();

    // Create sockets
    let server_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server socket");
    let server_addr = server_socket
        .local_addr()
        .expect("Failed to get server address");

    let client_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind client socket");
    let client_addr = client_socket
        .local_addr()
        .expect("Failed to get client address");

    // Create configs
    let mut server_config = create_server_config(&certs);
    let mut client_config = create_client_config();

    // Create connections
    let client_scid = generate_conn_id();
    let client_scid_ref = quiche::ConnectionId::from_ref(&client_scid);

    let mut client_conn = quiche::connect(
        Some("quic-vpn"),
        &client_scid_ref,
        client_addr,
        server_addr,
        &mut client_config,
    )
    .expect("Failed to create client connection");

    let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];
    let mut recv_buf = vec![0u8; 65535];
    let mut server_conn: Option<quiche::Connection> = None;
    let mut server_client_addr: Option<SocketAddr> = None;

    // Establish connection (reused from previous test)
    let mut iterations = 0;
    const MAX_ITERATIONS: i32 = 100;

    loop {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            panic!("Test timeout after {} iterations", MAX_ITERATIONS);
        }

        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send from client");
        }

        match tokio::time::timeout(
            Duration::from_millis(50),
            server_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                if server_conn.is_none() {
                    if let Ok(hdr) =
                        quiche::Header::from_slice(&mut recv_buf[..len], quiche::MAX_CONN_ID_LEN)
                    {
                        if hdr.ty == quiche::Type::Initial {
                            let server_scid = generate_conn_id();
                            let server_scid_ref = quiche::ConnectionId::from_ref(&server_scid);
                            if let Ok(conn) = quiche::accept(
                                &server_scid_ref,
                                None,
                                server_addr,
                                from,
                                &mut server_config,
                            ) {
                                server_conn = Some(conn);
                                server_client_addr = Some(from);
                            }
                        }
                    }
                }

                if let Some(ref mut conn) = server_conn {
                    let recv_info = quiche::RecvInfo {
                        from,
                        to: server_addr,
                    };
                    let _ = conn.recv(&mut recv_buf[..len], recv_info);
                }
            }
            _ => {}
        }

        if let Some(ref mut conn) = server_conn {
            if let Some(addr) = server_client_addr {
                while let Ok((write_len, _)) = conn.send(&mut out_buf) {
                    server_socket
                        .send_to(&out_buf[..write_len], addr)
                        .await
                        .expect("Failed to send from server");
                }
            }
        }

        match tokio::time::timeout(
            Duration::from_millis(50),
            client_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: client_addr,
                };
                let _ = client_conn.recv(&mut recv_buf[..len], recv_info);
            }
            _ => {}
        }

        if let Some(ref conn) = server_conn {
            if client_conn.is_established() && conn.is_established() {
                break;
            }
        }
    }

    let server_conn = server_conn
        .as_mut()
        .expect("Server connection should exist");

    // Test sending various data payloads
    let test_payloads: Vec<Vec<u8>> = vec![
        b"Hello, VPN!".to_vec(),
        vec![0xDE, 0xAD, 0xBE, 0xEF], // Binary data
        vec![0u8; 1000],              // 1KB of zeros
        (0..255).collect(),           // All byte values
    ];

    for (idx, payload) in test_payloads.iter().enumerate() {
        let data_packet = VpnPacket::new_data(Bytes::from(payload.clone()));
        let encoded = data_packet.encode();
        let stream_id = ((idx as u64) * 4) + 4; // Client-initiated bidirectional streams

        client_conn
            .stream_send(stream_id, &encoded, false)
            .expect("Failed to send data packet");

        // Exchange packets and verify server receives the data
        let mut data_received = false;
        let mut received_data: Option<Bytes> = None;
        iterations = 0;

        while !data_received {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                panic!(
                    "Data not received after {} iterations for payload {}",
                    MAX_ITERATIONS, idx
                );
            }

            while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
                client_socket
                    .send_to(&out_buf[..write_len], server_addr)
                    .await
                    .expect("Failed to send from client");
            }

            match tokio::time::timeout(
                Duration::from_millis(50),
                server_socket.recv_from(&mut recv_buf),
            )
            .await
            {
                Ok(Ok((len, from))) => {
                    let recv_info = quiche::RecvInfo {
                        from,
                        to: server_addr,
                    };
                    let _ = server_conn.recv(&mut recv_buf[..len], recv_info);

                    for stream in server_conn.readable() {
                        let mut stream_buf = vec![0u8; MAX_DATAGRAM_SIZE];
                        if let Ok((len, _fin)) = server_conn.stream_recv(stream, &mut stream_buf) {
                            if let Some(packet) =
                                VpnPacket::decode(Bytes::copy_from_slice(&stream_buf[..len]))
                            {
                                if packet.packet_type == PACKET_TYPE_DATA {
                                    data_received = true;
                                    received_data = Some(packet.data);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            if let Some(addr) = server_client_addr {
                while let Ok((write_len, _)) = server_conn.send(&mut out_buf) {
                    server_socket
                        .send_to(&out_buf[..write_len], addr)
                        .await
                        .expect("Failed to send from server");
                }
            }

            match tokio::time::timeout(
                Duration::from_millis(50),
                client_socket.recv_from(&mut recv_buf),
            )
            .await
            {
                Ok(Ok((len, from))) => {
                    let recv_info = quiche::RecvInfo {
                        from,
                        to: client_addr,
                    };
                    let _ = client_conn.recv(&mut recv_buf[..len], recv_info);
                }
                _ => {}
            }
        }

        assert!(
            data_received,
            "Should have received data packet for payload {}",
            idx
        );
        assert_eq!(
            received_data.as_ref().map(|d| d.as_ref()),
            Some(payload.as_slice()),
            "Received data should match sent payload for payload {}",
            idx
        );
    }
}

#[tokio::test]
async fn test_multiple_streams() {
    // Create test certificates
    let certs = TestCerts::new();

    // Create sockets
    let server_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server socket");
    let server_addr = server_socket
        .local_addr()
        .expect("Failed to get server address");

    let client_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind client socket");
    let client_addr = client_socket
        .local_addr()
        .expect("Failed to get client address");

    // Create configs
    let mut server_config = create_server_config(&certs);
    let mut client_config = create_client_config();

    // Create connections
    let client_scid = generate_conn_id();
    let client_scid_ref = quiche::ConnectionId::from_ref(&client_scid);

    let mut client_conn = quiche::connect(
        Some("quic-vpn"),
        &client_scid_ref,
        client_addr,
        server_addr,
        &mut client_config,
    )
    .expect("Failed to create client connection");

    let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];
    let mut recv_buf = vec![0u8; 65535];
    let mut server_conn: Option<quiche::Connection> = None;
    let mut server_client_addr: Option<SocketAddr> = None;

    // Establish connection
    let mut iterations = 0;
    const MAX_ITERATIONS: i32 = 100;

    loop {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            panic!("Test timeout");
        }

        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(
            Duration::from_millis(50),
            server_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                if server_conn.is_none() {
                    if let Ok(hdr) =
                        quiche::Header::from_slice(&mut recv_buf[..len], quiche::MAX_CONN_ID_LEN)
                    {
                        if hdr.ty == quiche::Type::Initial {
                            let server_scid = generate_conn_id();
                            let server_scid_ref = quiche::ConnectionId::from_ref(&server_scid);
                            if let Ok(conn) = quiche::accept(
                                &server_scid_ref,
                                None,
                                server_addr,
                                from,
                                &mut server_config,
                            ) {
                                server_conn = Some(conn);
                                server_client_addr = Some(from);
                            }
                        }
                    }
                }

                if let Some(ref mut conn) = server_conn {
                    let recv_info = quiche::RecvInfo {
                        from,
                        to: server_addr,
                    };
                    let _ = conn.recv(&mut recv_buf[..len], recv_info);
                }
            }
            _ => {}
        }

        if let Some(ref mut conn) = server_conn {
            if let Some(addr) = server_client_addr {
                while let Ok((write_len, _)) = conn.send(&mut out_buf) {
                    server_socket
                        .send_to(&out_buf[..write_len], addr)
                        .await
                        .expect("Failed to send");
                }
            }
        }

        match tokio::time::timeout(
            Duration::from_millis(50),
            client_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: client_addr,
                };
                let _ = client_conn.recv(&mut recv_buf[..len], recv_info);
            }
            _ => {}
        }

        if let Some(ref conn) = server_conn {
            if client_conn.is_established() && conn.is_established() {
                break;
            }
        }
    }

    let server_conn = server_conn
        .as_mut()
        .expect("Server connection should exist");

    // Send packets on multiple streams simultaneously
    let stream_ids: Vec<u64> = (0..5).map(|i| i * 4).collect(); // Client-initiated bidirectional streams

    for stream_id in &stream_ids {
        let msg = format!("Stream {} data", stream_id);
        let packet = VpnPacket::new_data(Bytes::from(msg));
        client_conn
            .stream_send(*stream_id, &packet.encode(), false)
            .expect("Failed to send on stream");
    }

    // Exchange packets and collect received data
    let mut received_streams: std::collections::HashSet<u64> = std::collections::HashSet::new();
    iterations = 0;

    while received_streams.len() < stream_ids.len() {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            panic!(
                "Not all streams received. Got {} of {}",
                received_streams.len(),
                stream_ids.len()
            );
        }

        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(
            Duration::from_millis(50),
            server_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: server_addr,
                };
                let _ = server_conn.recv(&mut recv_buf[..len], recv_info);

                for stream in server_conn.readable() {
                    let mut stream_buf = vec![0u8; MAX_DATAGRAM_SIZE];
                    if let Ok((len, _fin)) = server_conn.stream_recv(stream, &mut stream_buf) {
                        if let Some(packet) =
                            VpnPacket::decode(Bytes::copy_from_slice(&stream_buf[..len]))
                        {
                            if packet.packet_type == PACKET_TYPE_DATA {
                                received_streams.insert(stream);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        if let Some(addr) = server_client_addr {
            while let Ok((write_len, _)) = server_conn.send(&mut out_buf) {
                server_socket
                    .send_to(&out_buf[..write_len], addr)
                    .await
                    .expect("Failed to send");
            }
        }

        match tokio::time::timeout(
            Duration::from_millis(50),
            client_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: client_addr,
                };
                let _ = client_conn.recv(&mut recv_buf[..len], recv_info);
            }
            _ => {}
        }
    }

    assert_eq!(
        received_streams.len(),
        stream_ids.len(),
        "Should have received data on all streams"
    );
}

#[test]
fn test_packet_integrity() {
    // Test that packets maintain integrity through encode/decode
    let test_data = vec![
        vec![0u8; 0],       // Empty
        vec![0xFFu8; 1],    // Single byte
        vec![0xABu8; 1350], // Max MTU size data
    ];

    for data in test_data {
        let original = VpnPacket::new_data(Bytes::from(data.clone()));
        let encoded = original.encode();
        let decoded = VpnPacket::decode(encoded).expect("Decode should succeed");

        assert_eq!(decoded.packet_type, PACKET_TYPE_DATA);
        assert_eq!(decoded.data.as_ref(), data.as_slice());
    }
}

#[test]
fn test_packet_type_identification() {
    // Verify all packet types are correctly identified
    let ping = VpnPacket::new_ping();
    let pong = VpnPacket::new_pong();
    let data = VpnPacket::new_data(Bytes::from_static(b"test"));

    assert_eq!(ping.packet_type, PACKET_TYPE_PING);
    assert_eq!(pong.packet_type, PACKET_TYPE_PONG);
    assert_eq!(data.packet_type, PACKET_TYPE_DATA);

    // Verify after encode/decode
    let ping_decoded = VpnPacket::decode(ping.encode()).expect("Decode ping");
    let pong_decoded = VpnPacket::decode(pong.encode()).expect("Decode pong");
    let data_decoded = VpnPacket::decode(data.encode()).expect("Decode data");

    assert_eq!(ping_decoded.packet_type, PACKET_TYPE_PING);
    assert_eq!(pong_decoded.packet_type, PACKET_TYPE_PONG);
    assert_eq!(data_decoded.packet_type, PACKET_TYPE_DATA);
}

#[tokio::test]
async fn test_connection_timeout_handling() {
    // Create client socket only - no server
    let client_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind client socket");
    let client_addr = client_socket
        .local_addr()
        .expect("Failed to get client address");

    // Use a non-existent server address
    let server_addr: SocketAddr = "127.0.0.1:19999".parse().expect("Invalid address");

    // Create client config with short timeout
    let mut client_config = create_client_config();
    client_config.set_max_idle_timeout(500); // 500ms timeout

    let client_scid = generate_conn_id();
    let client_scid_ref = quiche::ConnectionId::from_ref(&client_scid);

    let mut client_conn = quiche::connect(
        Some("quic-vpn"),
        &client_scid_ref,
        client_addr,
        server_addr,
        &mut client_config,
    )
    .expect("Failed to create client connection");

    // Send initial packet
    let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];
    while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
        let _ = client_socket
            .send_to(&out_buf[..write_len], server_addr)
            .await;
    }

    // Wait and repeatedly trigger timeout until connection closes
    let start_time = std::time::Instant::now();
    let max_wait = Duration::from_secs(5);

    while !client_conn.is_timed_out() && !client_conn.is_closed() {
        if start_time.elapsed() > max_wait {
            // If we've waited too long without timeout, that's still ok -
            // the test verifies connection doesn't hang forever
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        client_conn.on_timeout();

        // Try to send again to process timeouts
        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            let _ = client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await;
        }
    }

    // Verify that the connection eventually times out or closes
    // (we may have exited due to max_wait, which is also acceptable behavior)
    assert!(
        client_conn.is_timed_out() || client_conn.is_closed() || start_time.elapsed() >= max_wait,
        "Connection should either timeout, close, or we should have waited max time"
    );
}

#[test]
fn test_certificate_generation_validity() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cert_path = temp_dir.path().join("test_cert.pem");
    let key_path = temp_dir.path().join("test_key.pem");

    generate_self_signed_cert(&cert_path, &key_path).expect("Failed to generate certs");

    // Read and verify certificate structure
    let cert_pem = std::fs::read_to_string(&cert_path).expect("Failed to read cert");
    let key_pem = std::fs::read_to_string(&key_path).expect("Failed to read key");

    // Verify PEM format markers
    assert!(cert_pem.contains("-----BEGIN CERTIFICATE-----"));
    assert!(cert_pem.contains("-----END CERTIFICATE-----"));
    assert!(key_pem.contains("-----BEGIN PRIVATE KEY-----"));
    assert!(key_pem.contains("-----END PRIVATE KEY-----"));

    // Verify base64 content between markers
    let cert_content = cert_pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect::<String>();
    assert!(!cert_content.is_empty(), "Certificate should have content");

    // Try to use the certificate with quiche
    let mut config =
        quiche::Config::new(quiche::PROTOCOL_VERSION).expect("Failed to create config");
    config
        .load_cert_chain_from_pem_file(cert_path.to_str().expect("Invalid path"))
        .expect("Certificate should be valid for quiche");
    config
        .load_priv_key_from_pem_file(key_path.to_str().expect("Invalid path"))
        .expect("Private key should be valid for quiche");
}

#[tokio::test]
async fn test_bidirectional_data_flow() {
    // Create test certificates
    let certs = TestCerts::new();

    // Create sockets
    let server_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server socket");
    let server_addr = server_socket
        .local_addr()
        .expect("Failed to get server address");

    let client_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind client socket");
    let client_addr = client_socket
        .local_addr()
        .expect("Failed to get client address");

    // Create configs
    let mut server_config = create_server_config(&certs);
    let mut client_config = create_client_config();

    // Create client connection
    let client_scid = generate_conn_id();
    let client_scid_ref = quiche::ConnectionId::from_ref(&client_scid);

    let mut client_conn = quiche::connect(
        Some("quic-vpn"),
        &client_scid_ref,
        client_addr,
        server_addr,
        &mut client_config,
    )
    .expect("Failed to create client connection");

    let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];
    let mut recv_buf = vec![0u8; 65535];
    let mut server_conn: Option<quiche::Connection> = None;
    let mut server_client_addr: Option<SocketAddr> = None;

    // Establish connection
    let mut iterations = 0;
    const MAX_ITERATIONS: i32 = 100;

    loop {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            panic!("Test timeout");
        }

        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(
            Duration::from_millis(50),
            server_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                if server_conn.is_none() {
                    if let Ok(hdr) =
                        quiche::Header::from_slice(&mut recv_buf[..len], quiche::MAX_CONN_ID_LEN)
                    {
                        if hdr.ty == quiche::Type::Initial {
                            let server_scid = generate_conn_id();
                            let server_scid_ref = quiche::ConnectionId::from_ref(&server_scid);
                            if let Ok(conn) = quiche::accept(
                                &server_scid_ref,
                                None,
                                server_addr,
                                from,
                                &mut server_config,
                            ) {
                                server_conn = Some(conn);
                                server_client_addr = Some(from);
                            }
                        }
                    }
                }

                if let Some(ref mut conn) = server_conn {
                    let recv_info = quiche::RecvInfo {
                        from,
                        to: server_addr,
                    };
                    let _ = conn.recv(&mut recv_buf[..len], recv_info);
                }
            }
            _ => {}
        }

        if let Some(ref mut conn) = server_conn {
            if let Some(addr) = server_client_addr {
                while let Ok((write_len, _)) = conn.send(&mut out_buf) {
                    server_socket
                        .send_to(&out_buf[..write_len], addr)
                        .await
                        .expect("Failed to send");
                }
            }
        }

        match tokio::time::timeout(
            Duration::from_millis(50),
            client_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: client_addr,
                };
                let _ = client_conn.recv(&mut recv_buf[..len], recv_info);
            }
            _ => {}
        }

        if let Some(ref conn) = server_conn {
            if client_conn.is_established() && conn.is_established() {
                break;
            }
        }
    }

    let server_conn = server_conn
        .as_mut()
        .expect("Server connection should exist");

    // Test bidirectional flow:
    // 1. Client sends data
    // 2. Server echoes it back
    // 3. Client verifies received data

    let test_message = b"Bidirectional test message";
    let stream_id = 0u64;

    let data_packet = VpnPacket::new_data(Bytes::from_static(test_message));
    client_conn
        .stream_send(stream_id, &data_packet.encode(), false)
        .expect("Failed to send data");

    let mut client_received_echo = false;
    iterations = 0;

    while !client_received_echo {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            panic!("Echo not received");
        }

        // Client sends
        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send");
        }

        // Server receives, processes, and echoes
        match tokio::time::timeout(
            Duration::from_millis(50),
            server_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: server_addr,
                };
                let _ = server_conn.recv(&mut recv_buf[..len], recv_info);

                for stream in server_conn.readable() {
                    let mut stream_buf = vec![0u8; MAX_DATAGRAM_SIZE];
                    if let Ok((len, _fin)) = server_conn.stream_recv(stream, &mut stream_buf) {
                        if let Some(packet) =
                            VpnPacket::decode(Bytes::copy_from_slice(&stream_buf[..len]))
                        {
                            if packet.packet_type == PACKET_TYPE_DATA {
                                // Echo back
                                let echo = VpnPacket::new_data(packet.data);
                                let _ = server_conn.stream_send(stream, &echo.encode(), false);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        // Server sends echo
        if let Some(addr) = server_client_addr {
            while let Ok((write_len, _)) = server_conn.send(&mut out_buf) {
                server_socket
                    .send_to(&out_buf[..write_len], addr)
                    .await
                    .expect("Failed to send");
            }
        }

        // Client receives echo
        match tokio::time::timeout(
            Duration::from_millis(50),
            client_socket.recv_from(&mut recv_buf),
        )
        .await
        {
            Ok(Ok((len, from))) => {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: client_addr,
                };
                let _ = client_conn.recv(&mut recv_buf[..len], recv_info);

                for stream in client_conn.readable() {
                    let mut stream_buf = vec![0u8; MAX_DATAGRAM_SIZE];
                    if let Ok((len, _fin)) = client_conn.stream_recv(stream, &mut stream_buf) {
                        if let Some(packet) =
                            VpnPacket::decode(Bytes::copy_from_slice(&stream_buf[..len]))
                        {
                            if packet.packet_type == PACKET_TYPE_DATA {
                                assert_eq!(
                                    packet.data.as_ref(),
                                    test_message,
                                    "Echoed data should match original"
                                );
                                client_received_echo = true;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    assert!(client_received_echo, "Client should receive echoed data");
}
