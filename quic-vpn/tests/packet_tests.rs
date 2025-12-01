use bytes::Bytes;
use quic_vpn::packet::*;

#[test]
fn test_data_packet_encode_decode() {
    let original_data = b"Hello, QUIC VPN!";
    let packet = VpnPacket::new_data(Bytes::from_static(original_data));

    assert_eq!(packet.packet_type, PACKET_TYPE_DATA);
    assert_eq!(&packet.data[..], original_data);

    let encoded = packet.encode();
    let decoded = VpnPacket::decode(encoded).expect("Failed to decode packet");

    assert_eq!(decoded.packet_type, PACKET_TYPE_DATA);
    assert_eq!(&decoded.data[..], original_data);
}

#[test]
fn test_ping_packet_encode_decode() {
    let packet = VpnPacket::new_ping();

    assert_eq!(packet.packet_type, PACKET_TYPE_PING);
    assert!(packet.data.is_empty());

    let encoded = packet.encode();
    let decoded = VpnPacket::decode(encoded).expect("Failed to decode packet");

    assert_eq!(decoded.packet_type, PACKET_TYPE_PING);
    assert!(decoded.data.is_empty());
}

#[test]
fn test_pong_packet_encode_decode() {
    let packet = VpnPacket::new_pong();

    assert_eq!(packet.packet_type, PACKET_TYPE_PONG);
    assert!(packet.data.is_empty());

    let encoded = packet.encode();
    let decoded = VpnPacket::decode(encoded).expect("Failed to decode packet");

    assert_eq!(decoded.packet_type, PACKET_TYPE_PONG);
    assert!(decoded.data.is_empty());
}

#[test]
fn test_empty_packet_decode() {
    let empty_bytes = Bytes::new();
    let result = VpnPacket::decode(empty_bytes);

    assert!(result.is_none(), "Empty bytes should return None");
}

#[test]
fn test_large_data_packet() {
    let large_data = vec![0xAB; 1400];
    let packet = VpnPacket::new_data(Bytes::from(large_data.clone()));

    assert_eq!(packet.packet_type, PACKET_TYPE_DATA);
    assert_eq!(&packet.data[..], &large_data[..]);

    let encoded = packet.encode();
    assert_eq!(encoded.len(), 1 + large_data.len());

    let decoded = VpnPacket::decode(encoded).expect("Failed to decode large packet");

    assert_eq!(decoded.packet_type, PACKET_TYPE_DATA);
    assert_eq!(&decoded.data[..], &large_data[..]);
}

#[test]
fn test_packet_type_preserved() {
    let test_cases: Vec<(u8, &[u8])> = vec![
        (PACKET_TYPE_DATA, b"some data"),
        (PACKET_TYPE_PING, b""),
        (PACKET_TYPE_PONG, b""),
    ];

    for (packet_type, data) in test_cases {
        let packet = VpnPacket {
            packet_type,
            data: Bytes::copy_from_slice(data),
        };

        let encoded = packet.encode();
        let decoded = VpnPacket::decode(encoded).expect("Failed to decode packet");

        assert_eq!(
            decoded.packet_type, packet_type,
            "Packet type should be preserved"
        );
    }
}

#[test]
fn test_encoding_format() {
    let data = b"test";
    let packet = VpnPacket::new_data(Bytes::from_static(data));
    let encoded = packet.encode();

    assert_eq!(encoded[0], PACKET_TYPE_DATA);
    assert_eq!(&encoded[1..], data);
}
