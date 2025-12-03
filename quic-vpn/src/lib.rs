use anyhow::{Context, Result};
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};
use std::fs;
use std::path::Path;

/// Generate self-signed certificate and private key for QUIC
pub fn generate_self_signed_cert(cert_path: &Path, key_path: &Path) -> Result<()> {
    // Generate key pair first
    let key_pair = KeyPair::generate().context("Failed to generate key pair")?;
    let key_pem = key_pair.serialize_pem();

    // Create certificate parameters
    let mut params = CertificateParams::default();
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, "quic-vpn");
    dn.push(DnType::OrganizationName, "QUIC VPN Demo");
    params.distinguished_name = dn;

    // Generate certificate with the key pair
    let cert = params
        .self_signed(&key_pair)
        .context("Failed to generate self-signed certificate")?;

    // Write certificate (PEM format)
    fs::write(cert_path, cert.pem()).context("Failed to write certificate")?;

    // Write private key (PEM format)
    fs::write(key_path, key_pem).context("Failed to write private key")?;

    Ok(())
}

/// Ensure certificates exist, generate if they don't
pub fn ensure_certificates(cert_path: &Path, key_path: &Path) -> Result<()> {
    if !cert_path.exists() || !key_path.exists() {
        tracing::info!("Generating self-signed certificates...");

        // Create parent directories if they don't exist
        if let Some(parent) = cert_path.parent() {
            fs::create_dir_all(parent).context("Failed to create certificate directory")?;
        }

        generate_self_signed_cert(cert_path, key_path)?;
        tracing::info!("Certificates generated successfully");
    } else {
        tracing::info!("Using existing certificates");
    }
    Ok(())
}

/// Common VPN packet format
pub mod packet {
    use bytes::{Buf, BufMut, Bytes, BytesMut};

    pub const PACKET_TYPE_DATA: u8 = 0x01;
    pub const PACKET_TYPE_PING: u8 = 0x02;
    pub const PACKET_TYPE_PONG: u8 = 0x03;

    pub struct VpnPacket {
        pub packet_type: u8,
        pub data: Bytes,
    }

    impl VpnPacket {
        pub fn new_data(data: Bytes) -> Self {
            Self {
                packet_type: PACKET_TYPE_DATA,
                data,
            }
        }

        pub fn new_ping() -> Self {
            Self {
                packet_type: PACKET_TYPE_PING,
                data: Bytes::new(),
            }
        }

        pub fn new_pong() -> Self {
            Self {
                packet_type: PACKET_TYPE_PONG,
                data: Bytes::new(),
            }
        }

        pub fn encode(&self) -> Bytes {
            let mut buf = BytesMut::with_capacity(1 + self.data.len());
            buf.put_u8(self.packet_type);
            buf.put(self.data.clone());
            buf.freeze()
        }

        pub fn decode(mut data: Bytes) -> Option<Self> {
            if data.is_empty() {
                return None;
            }
            let packet_type = data.get_u8();
            Some(Self { packet_type, data })
        }
    }
}

/// QUIC configuration constants
pub const MAX_DATAGRAM_SIZE: usize = 1350;
pub const DEFAULT_SERVER_PORT: u16 = 4433;
pub const DEFAULT_SERVER_ADDR: &str = "0.0.0.0";
