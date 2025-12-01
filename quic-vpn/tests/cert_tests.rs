use quic_vpn::{ensure_certificates, generate_self_signed_cert};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_generate_self_signed_cert() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cert_path = temp_dir.path().join("test_cert.pem");
    let key_path = temp_dir.path().join("test_key.pem");

    let result = generate_self_signed_cert(&cert_path, &key_path);
    assert!(result.is_ok(), "Failed to generate certificates: {:?}", result);

    assert!(cert_path.exists(), "Certificate file should exist");
    assert!(key_path.exists(), "Key file should exist");

    let cert_contents = fs::read_to_string(&cert_path)
        .expect("Failed to read certificate");
    let key_contents = fs::read_to_string(&key_path)
        .expect("Failed to read key");

    assert!(cert_contents.contains("BEGIN CERTIFICATE"));
    assert!(cert_contents.contains("END CERTIFICATE"));
    assert!(key_contents.contains("BEGIN PRIVATE KEY"));
    assert!(key_contents.contains("END PRIVATE KEY"));
}

#[test]
fn test_ensure_certificates_creates_if_missing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cert_path = temp_dir.path().join("certs").join("cert.pem");
    let key_path = temp_dir.path().join("certs").join("key.pem");

    assert!(!cert_path.exists(), "Cert should not exist initially");
    assert!(!key_path.exists(), "Key should not exist initially");

    let result = ensure_certificates(&cert_path, &key_path);
    assert!(result.is_ok(), "Failed to ensure certificates: {:?}", result);

    assert!(cert_path.exists(), "Certificate should be created");
    assert!(key_path.exists(), "Key should be created");
}

#[test]
fn test_ensure_certificates_preserves_existing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cert_path = temp_dir.path().join("cert.pem");
    let key_path = temp_dir.path().join("key.pem");

    generate_self_signed_cert(&cert_path, &key_path)
        .expect("Failed to generate initial certificates");

    let original_cert = fs::read(&cert_path).expect("Failed to read cert");
    let original_key = fs::read(&key_path).expect("Failed to read key");

    let result = ensure_certificates(&cert_path, &key_path);
    assert!(result.is_ok(), "Failed to ensure certificates: {:?}", result);

    let final_cert = fs::read(&cert_path).expect("Failed to read cert after ensure");
    let final_key = fs::read(&key_path).expect("Failed to read key after ensure");

    assert_eq!(original_cert, final_cert, "Certificate should not be regenerated");
    assert_eq!(original_key, final_key, "Key should not be regenerated");
}

#[test]
fn test_cert_generation_in_nested_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let nested_path = temp_dir.path().join("a").join("b").join("c");
    let cert_path = nested_path.join("cert.pem");
    let key_path = nested_path.join("key.pem");

    let result = ensure_certificates(&cert_path, &key_path);
    assert!(result.is_ok(), "Failed to create certs in nested directory: {:?}", result);

    assert!(cert_path.exists(), "Certificate should exist in nested directory");
    assert!(key_path.exists(), "Key should exist in nested directory");
}
