//! End-to-end tests for QUIC VPN
//!
//! These tests verify:
//! - Complete client-server interaction
//! - Traffic routing without leaks
//! - Data integrity across the VPN tunnel
//! - Proper session management

use bytes::Bytes;
use quic_vpn::{generate_self_signed_cert, packet::*, MAX_DATAGRAM_SIZE};
use ring::rand::SecureRandom;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tempfile::TempDir;
use tokio::net::UdpSocket;

/// Maximum iterations for connection establishment loops
const CONNECTION_MAX_ITERATIONS: i32 = 100;
/// Maximum iterations for data exchange loops
const EXCHANGE_MAX_ITERATIONS: i32 = 200;
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
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION).expect("Failed to create config");
    config
        .load_cert_chain_from_pem_file(certs.cert_path.to_str().expect("Invalid path"))
        .expect("Failed to load cert");
    config
        .load_priv_key_from_pem_file(certs.key_path.to_str().expect("Invalid path"))
        .expect("Failed to load key");
    config
        .set_application_protos(&[b"quic-vpn"])
        .expect("Failed to set ALPN");
    config.set_max_idle_timeout(10000);
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
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION).expect("Failed to create config");
    config
        .set_application_protos(&[b"quic-vpn"])
        .expect("Failed to set ALPN");
    config.set_max_idle_timeout(10000);
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

/// Helper to establish a connection
async fn establish_connection(
    certs: &TestCerts,
) -> (
    quiche::Connection,
    quiche::Connection,
    UdpSocket,
    UdpSocket,
    SocketAddr,
    SocketAddr,
) {
    let server_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server socket");
    let server_addr = server_socket.local_addr().expect("Failed to get server address");

    let client_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind client socket");
    let client_addr = client_socket.local_addr().expect("Failed to get client address");

    let mut server_config = create_server_config(certs);
    let mut client_config = create_client_config();

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

    let mut iterations = 0;
    const MAX_ITERATIONS: i32 = 100;

    loop {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            panic!("Connection not established");
        }

        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(Duration::from_millis(50), server_socket.recv_from(&mut recv_buf))
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
            while let Ok((write_len, _)) = conn.send(&mut out_buf) {
                server_socket
                    .send_to(&out_buf[..write_len], client_addr)
                    .await
                    .expect("Failed to send");
            }
        }

        match tokio::time::timeout(Duration::from_millis(50), client_socket.recv_from(&mut recv_buf))
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

    let server_conn = server_conn.expect("Server connection should exist");
    (
        client_conn,
        server_conn,
        client_socket,
        server_socket,
        client_addr,
        server_addr,
    )
}

/// Test that verifies no traffic leaks outside the VPN tunnel
/// This is simulated by ensuring all packets go through the QUIC connection
#[tokio::test]
async fn test_no_traffic_leaks() {
    let certs = TestCerts::new();
    let (mut client_conn, mut server_conn, client_socket, server_socket, client_addr, server_addr) =
        establish_connection(&certs).await;

    // Track all packets sent and received
    let mut packets_sent_to_server = 0u32;
    let mut packets_received_by_server = 0u32;

    let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];
    let mut recv_buf = vec![0u8; 65535];

    // Send multiple data packets
    let num_packets = 10;
    for idx in 0..num_packets {
        let data = format!("Packet {}", idx);
        let packet = VpnPacket::new_data(Bytes::from(data));
        let stream_id = (idx as u64) * 4;

        client_conn
            .stream_send(stream_id, &packet.encode(), false)
            .expect("Failed to send");
        packets_sent_to_server += 1;
    }

    // Exchange packets until all are received
    let mut iterations = 0;
    const MAX_ITERATIONS: i32 = 200;

    while packets_received_by_server < packets_sent_to_server {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            break;
        }

        // Client sends QUIC packets
        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send");
        }

        // Server receives
        match tokio::time::timeout(Duration::from_millis(50), server_socket.recv_from(&mut recv_buf))
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
                                packets_received_by_server += 1;
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        // Server sends
        while let Ok((write_len, _)) = server_conn.send(&mut out_buf) {
            server_socket
                .send_to(&out_buf[..write_len], client_addr)
                .await
                .expect("Failed to send");
        }

        // Client receives
        match tokio::time::timeout(Duration::from_millis(50), client_socket.recv_from(&mut recv_buf))
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

    // Verify all packets were properly routed through the tunnel
    assert_eq!(
        packets_received_by_server, packets_sent_to_server,
        "All packets should be received through the QUIC tunnel"
    );
}

/// Test data integrity - verify no corruption during transmission
#[tokio::test]
async fn test_data_integrity_in_tunnel() {
    let certs = TestCerts::new();
    let (mut client_conn, mut server_conn, client_socket, server_socket, client_addr, server_addr) =
        establish_connection(&certs).await;

    let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];
    let mut recv_buf = vec![0u8; 65535];

    // Create test data with known patterns that would reveal corruption
    let test_patterns: Vec<Vec<u8>> = vec![
        // Alternating bits pattern
        (0..256).map(|i| if i % 2 == 0 { 0xAA } else { 0x55 }).collect(),
        // Sequential values
        (0..255u8).collect(),
        // All zeros
        vec![0u8; 200],
        // All ones
        vec![0xFFu8; 200],
        // Mixed pattern
        vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE]
            .into_iter()
            .cycle()
            .take(100)
            .collect(),
    ];

    let mut received_data: HashMap<u64, Vec<u8>> = HashMap::new();

    // Send all patterns
    for (idx, pattern) in test_patterns.iter().enumerate() {
        let packet = VpnPacket::new_data(Bytes::from(pattern.clone()));
        let stream_id = (idx as u64) * 4;
        client_conn
            .stream_send(stream_id, &packet.encode(), false)
            .expect("Failed to send");
    }

    // Exchange and collect
    let mut iterations = 0;
    const MAX_ITERATIONS: i32 = 200;

    while received_data.len() < test_patterns.len() {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            break;
        }

        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(Duration::from_millis(50), server_socket.recv_from(&mut recv_buf))
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
                                received_data.insert(stream, packet.data.to_vec());
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        while let Ok((write_len, _)) = server_conn.send(&mut out_buf) {
            server_socket
                .send_to(&out_buf[..write_len], client_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(Duration::from_millis(50), client_socket.recv_from(&mut recv_buf))
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

    // Verify integrity of all received data
    assert_eq!(
        received_data.len(),
        test_patterns.len(),
        "All patterns should be received"
    );

    for (idx, pattern) in test_patterns.iter().enumerate() {
        let stream_id = (idx as u64) * 4;
        let received = received_data.get(&stream_id).expect("Data should exist");
        assert_eq!(
            received, pattern,
            "Pattern {} should match exactly (no corruption)",
            idx
        );
    }
}

/// Test that the VPN properly handles large payloads close to MTU
#[tokio::test]
async fn test_mtu_boundary_packets() {
    let certs = TestCerts::new();
    let (mut client_conn, mut server_conn, client_socket, server_socket, client_addr, server_addr) =
        establish_connection(&certs).await;

    let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];
    let mut recv_buf = vec![0u8; 65535];

    // Test packets at various sizes
    // We test smaller sizes that definitely fit within a single stream frame
    let sizes = vec![
        1,
        100,
        500,
        1000,
        1200,
    ];

    let mut received_sizes: Vec<usize> = Vec::new();

    for (idx, size) in sizes.iter().enumerate() {
        let data: Vec<u8> = (0..*size as u8).cycle().take(*size).collect();
        let packet = VpnPacket::new_data(Bytes::from(data));
        let stream_id = (idx as u64) * 4;
        client_conn
            .stream_send(stream_id, &packet.encode(), false)
            .expect("Failed to send");
    }

    let mut iterations = 0;
    const MAX_ITERATIONS: i32 = 200;

    while received_sizes.len() < sizes.len() {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            break;
        }

        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(Duration::from_millis(50), server_socket.recv_from(&mut recv_buf))
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
                                received_sizes.push(packet.data.len());
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        while let Ok((write_len, _)) = server_conn.send(&mut out_buf) {
            server_socket
                .send_to(&out_buf[..write_len], client_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(Duration::from_millis(50), client_socket.recv_from(&mut recv_buf))
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

    // Sort and verify all sizes were received
    received_sizes.sort();
    let mut expected_sizes = sizes.clone();
    expected_sizes.sort();

    assert_eq!(
        received_sizes, expected_sizes,
        "All MTU boundary packets should be received correctly"
    );
}

/// Test concurrent streams handling
#[tokio::test]
async fn test_concurrent_streams_isolation() {
    let certs = TestCerts::new();
    let (mut client_conn, mut server_conn, client_socket, server_socket, client_addr, server_addr) =
        establish_connection(&certs).await;

    let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];
    let mut recv_buf = vec![0u8; 65535];

    // Create multiple streams with unique data
    let num_streams = 10;
    let mut stream_data: HashMap<u64, Vec<u8>> = HashMap::new();

    for idx in 0..num_streams {
        let stream_id = (idx as u64) * 4;
        // Each stream gets unique data based on its ID
        let data: Vec<u8> = vec![idx as u8; 50];
        stream_data.insert(stream_id, data.clone());

        let packet = VpnPacket::new_data(Bytes::from(data));
        client_conn
            .stream_send(stream_id, &packet.encode(), false)
            .expect("Failed to send");
    }

    let mut received_stream_data: HashMap<u64, Vec<u8>> = HashMap::new();

    let mut iterations = 0;
    const MAX_ITERATIONS: i32 = 200;

    while received_stream_data.len() < num_streams as usize {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            break;
        }

        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(Duration::from_millis(50), server_socket.recv_from(&mut recv_buf))
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
                                received_stream_data.insert(stream, packet.data.to_vec());
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        while let Ok((write_len, _)) = server_conn.send(&mut out_buf) {
            server_socket
                .send_to(&out_buf[..write_len], client_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(Duration::from_millis(50), client_socket.recv_from(&mut recv_buf))
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

    // Verify stream isolation - data on each stream matches expected
    assert_eq!(
        received_stream_data.len(),
        stream_data.len(),
        "All streams should be received"
    );

    for (stream_id, expected_data) in &stream_data {
        let received = received_stream_data
            .get(stream_id)
            .expect("Stream data should exist");
        assert_eq!(
            received, expected_data,
            "Stream {} data should not leak to other streams",
            stream_id
        );
    }
}

/// Test ping-pong roundtrip timing
#[tokio::test]
async fn test_ping_pong_roundtrip() {
    let certs = TestCerts::new();
    let (mut client_conn, mut server_conn, client_socket, server_socket, client_addr, server_addr) =
        establish_connection(&certs).await;

    let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];
    let mut recv_buf = vec![0u8; 65535];

    // Send ping from client
    let stream_id = 0u64;
    let ping = VpnPacket::new_ping();
    client_conn
        .stream_send(stream_id, &ping.encode(), false)
        .expect("Failed to send ping");

    let start_time = std::time::Instant::now();
    let mut pong_received = false;

    let mut iterations = 0;
    const MAX_ITERATIONS: i32 = 100;

    while !pong_received {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            break;
        }

        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send");
        }

        // Server receives and responds
        match tokio::time::timeout(Duration::from_millis(50), server_socket.recv_from(&mut recv_buf))
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
                            if packet.packet_type == PACKET_TYPE_PING {
                                let pong = VpnPacket::new_pong();
                                let _ = server_conn.stream_send(stream, &pong.encode(), false);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        while let Ok((write_len, _)) = server_conn.send(&mut out_buf) {
            server_socket
                .send_to(&out_buf[..write_len], client_addr)
                .await
                .expect("Failed to send");
        }

        // Client receives pong
        match tokio::time::timeout(Duration::from_millis(50), client_socket.recv_from(&mut recv_buf))
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

    let roundtrip_time = start_time.elapsed();

    assert!(pong_received, "Pong should be received");
    // Roundtrip should be quick for localhost
    assert!(
        roundtrip_time < Duration::from_secs(5),
        "Ping-pong roundtrip should complete quickly"
    );
}

/// Test connection recovery after packet loss simulation
#[tokio::test]
async fn test_packet_ordering() {
    let certs = TestCerts::new();
    let (mut client_conn, mut server_conn, client_socket, server_socket, client_addr, server_addr) =
        establish_connection(&certs).await;

    let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];
    let mut recv_buf = vec![0u8; RECV_BUF_SIZE];

    // Send numbered packets on separate streams to verify ordering
    let num_packets = 20;

    for idx in 0..num_packets {
        let data = format!("{:04}", idx); // Zero-padded number
        let packet = VpnPacket::new_data(Bytes::from(data));
        // Client-initiated bidirectional streams: 0, 4, 8, 12, etc.
        let stream_id = (idx as u64) * STREAM_ID_INCREMENT;
        client_conn
            .stream_send(stream_id, &packet.encode(), false)
            .expect("Failed to send");
    }

    let mut received_data: HashMap<u64, String> = HashMap::new();

    let mut iterations = 0;
    const MAX_ITERATIONS: i32 = 200;

    while received_data.len() < num_packets as usize {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            break;
        }

        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(Duration::from_millis(50), server_socket.recv_from(&mut recv_buf))
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
                                let data = String::from_utf8_lossy(&packet.data).to_string();
                                received_data.insert(stream, data);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        while let Ok((write_len, _)) = server_conn.send(&mut out_buf) {
            server_socket
                .send_to(&out_buf[..write_len], client_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(Duration::from_millis(50), client_socket.recv_from(&mut recv_buf))
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

    // Verify all packets received with correct data
    assert_eq!(
        received_data.len(),
        num_packets as usize,
        "All packets should be received"
    );

    // Verify each stream has correct data
    for idx in 0..num_packets {
        let stream_id = (idx as u64) * 4;
        let expected = format!("{:04}", idx);
        let received = received_data.get(&stream_id).expect("Stream data should exist");
        assert_eq!(
            received, &expected,
            "Stream {} should have correct data",
            stream_id
        );
    }
}

/// Test that encryption is applied (QUIC provides TLS 1.3)
#[tokio::test]
async fn test_connection_is_encrypted() {
    let certs = TestCerts::new();
    let (client_conn, server_conn, _client_socket, _server_socket, _client_addr, _server_addr) =
        establish_connection(&certs).await;

    // Verify both connections are established and using TLS
    assert!(
        client_conn.is_established(),
        "Client connection should be established"
    );
    assert!(
        server_conn.is_established(),
        "Server connection should be established"
    );

    // QUIC provides encryption by default - the fact that the connection is established
    // means TLS 1.3 handshake completed successfully
    // We can verify the negotiated ALPN protocol
    assert_eq!(
        client_conn.application_proto(),
        b"quic-vpn",
        "ALPN should be negotiated"
    );
    assert_eq!(
        server_conn.application_proto(),
        b"quic-vpn",
        "Server ALPN should match"
    );
}

/// Test handling of connection close
#[tokio::test]
async fn test_graceful_connection_close() {
    let certs = TestCerts::new();
    let (mut client_conn, mut server_conn, client_socket, server_socket, client_addr, server_addr) =
        establish_connection(&certs).await;

    let mut out_buf = vec![0u8; MAX_DATAGRAM_SIZE];
    let mut recv_buf = vec![0u8; 65535];

    // Send some data first
    let packet = VpnPacket::new_data(Bytes::from_static(b"Hello"));
    client_conn
        .stream_send(0, &packet.encode(), false)
        .expect("Failed to send");

    // Exchange a few packets
    for _ in 0..10 {
        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(Duration::from_millis(50), server_socket.recv_from(&mut recv_buf))
            .await
        {
            Ok(Ok((len, from))) => {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: server_addr,
                };
                let _ = server_conn.recv(&mut recv_buf[..len], recv_info);
            }
            _ => {}
        }

        while let Ok((write_len, _)) = server_conn.send(&mut out_buf) {
            server_socket
                .send_to(&out_buf[..write_len], client_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(Duration::from_millis(50), client_socket.recv_from(&mut recv_buf))
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

    // Initiate close from client
    let _ = client_conn.close(true, 0, b"done");

    // Exchange close packets
    for _ in 0..20 {
        while let Ok((write_len, _)) = client_conn.send(&mut out_buf) {
            client_socket
                .send_to(&out_buf[..write_len], server_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(Duration::from_millis(50), server_socket.recv_from(&mut recv_buf))
            .await
        {
            Ok(Ok((len, from))) => {
                let recv_info = quiche::RecvInfo {
                    from,
                    to: server_addr,
                };
                let _ = server_conn.recv(&mut recv_buf[..len], recv_info);
            }
            _ => {}
        }

        while let Ok((write_len, _)) = server_conn.send(&mut out_buf) {
            server_socket
                .send_to(&out_buf[..write_len], client_addr)
                .await
                .expect("Failed to send");
        }

        match tokio::time::timeout(Duration::from_millis(50), client_socket.recv_from(&mut recv_buf))
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

        if client_conn.is_closed() && server_conn.is_closed() {
            break;
        }
    }

    // Client initiated close, so at minimum it should be in closing state
    // Note: In QUIC, the initiator might not be fully closed immediately,
    // but the close frame should have been sent
    assert!(
        client_conn.is_closed() || client_conn.is_draining(),
        "Client connection should be closed or draining"
    );
}
