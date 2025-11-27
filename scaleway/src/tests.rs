//! Tests for Scaleway API client

use crate::{Client, Error};

/// Create a test client (doesn't make real API calls)
fn create_test_client() -> Client {
    Client::new(
        reqwest::Client::new(),
        "test_secret_key".to_string(),
        Some("project-id-123".to_string()),
        Some("fr-par".to_string()),
    )
}

#[test]
fn test_client_creation() {
    let client = create_test_client();
    // Verify the client was created successfully by checking accessible methods
    assert_eq!(client.get_default_region().unwrap(), "fr-par");
    assert_eq!(client.get_default_project_id().unwrap(), "project-id-123");
}

#[test]
fn test_get_default_region() {
    let client = create_test_client();
    let region = client.get_default_region().unwrap();
    assert_eq!(region, "fr-par");
}

#[test]
fn test_get_default_region_none() {
    let client = Client::new(
        reqwest::Client::new(),
        "test_secret_key".to_string(),
        None,
        None,
    );
    let result = client.get_default_region();
    assert!(matches!(result, Err(Error::API(_))));
}

#[test]
fn test_get_default_project_id() {
    let client = create_test_client();
    let project_id = client.get_default_project_id().unwrap();
    assert_eq!(project_id, "project-id-123");
}

#[test]
fn test_get_default_project_id_none() {
    let client = Client::new(
        reqwest::Client::new(),
        "test_secret_key".to_string(),
        None,
        None,
    );
    let result = client.get_default_project_id();
    assert!(matches!(result, Err(Error::API(_))));
}

#[test]
fn test_api_error_display() {
    let error = crate::ApiError {
        status_code: 404,
        message: "Not found".to_string(),
    };
    assert_eq!(format!("{}", error), "API error (404): Not found");
}

#[test]
fn test_error_display() {
    let api_error = crate::ApiError {
        status_code: 500,
        message: "Internal server error".to_string(),
    };
    let error = Error::API(api_error);
    assert!(format!("{}", error).contains("Internal server error"));
}

// ============================================================================
// Key Manager types tests
// ============================================================================

#[test]
fn test_key_usage_serialization() {
    use crate::key_manager::KeyUsage;

    let usage = KeyUsage {
        symmetric_encryption: Some("aes_256_gcm".to_string()),
        asymmetric_encryption: None,
        asymmetric_signing: None,
    };

    let json = serde_json::to_string(&usage).unwrap();
    assert!(json.contains("aes_256_gcm"));
    assert!(!json.contains("asymmetric_encryption")); // None fields are skipped
}

#[test]
fn test_create_key_request_serialization() {
    use crate::key_manager::{CreateKeyRequest, KeyUsage};

    let request = CreateKeyRequest {
        project_id: "project-123".to_string(),
        name: "test-key".to_string(),
        usage: KeyUsage {
            symmetric_encryption: Some("aes_256_gcm".to_string()),
            asymmetric_encryption: None,
            asymmetric_signing: None,
        },
        description: Some("Test key".to_string()),
        tags: Some(vec!["test".to_string()]),
        rotation_policy: None,
        unprotected: None,
        origin: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("project-123"));
    assert!(json.contains("test-key"));
    assert!(json.contains("aes_256_gcm"));
}

// ============================================================================
// Instance types tests
// ============================================================================

#[test]
fn test_create_server_request_serialization() {
    use crate::instances::CreateServerRequest;

    let request = CreateServerRequest {
        name: "test-server".to_string(),
        commercial_type: "DEV1-S".to_string(),
        project: "project-123".to_string(),
        image: Some("image-123".to_string()),
        tags: Some(vec!["test".to_string()]),
        volumes: None,
        enable_ipv6: Some(true),
        public_ip: None,
        security_group: None,
        placement_group: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("test-server"));
    assert!(json.contains("DEV1-S"));
    assert!(json.contains("project-123"));
}

#[test]
fn test_create_volume_request_serialization() {
    use crate::instances::CreateVolumeRequest;

    let request = CreateVolumeRequest {
        name: "test-volume".to_string(),
        size: Some(20_000_000_000),
        volume_type: "l_ssd".to_string(),
        project: "project-123".to_string(),
        tags: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("test-volume"));
    assert!(json.contains("l_ssd"));
}

#[test]
fn test_create_security_group_request_serialization() {
    use crate::instances::CreateSecurityGroupRequest;

    let request = CreateSecurityGroupRequest {
        name: "test-sg".to_string(),
        description: Some("Test security group".to_string()),
        inbound_default_policy: "drop".to_string(),
        outbound_default_policy: "accept".to_string(),
        stateful: Some(true),
        project: "project-123".to_string(),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("test-sg"));
    assert!(json.contains("drop"));
    assert!(json.contains("accept"));
}

// ============================================================================
// Inference types tests
// ============================================================================

#[test]
fn test_create_deployment_request_serialization() {
    use crate::inference::CreateDeploymentRequest;

    let request = CreateDeploymentRequest {
        project_id: "project-123".to_string(),
        name: "test-deployment".to_string(),
        model_id: "model-123".to_string(),
        node_type: "L4".to_string(),
        min_size: 1,
        max_size: 3,
        accept_eula: Some(true),
        tags: Some(vec!["test".to_string()]),
        endpoints: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("test-deployment"));
    assert!(json.contains("model-123"));
    assert!(json.contains("L4"));
}

// ============================================================================
// Elastic Metal types tests
// ============================================================================

#[test]
fn test_create_baremetal_server_request_serialization() {
    use crate::elastic_metal::{CreateBaremetalServerRequest, InstallServerConfig};

    let request = CreateBaremetalServerRequest {
        offer_id: "offer-123".to_string(),
        project_id: "project-123".to_string(),
        name: "test-server".to_string(),
        description: Some("Test server".to_string()),
        tags: Some(vec!["test".to_string()]),
        install: Some(InstallServerConfig {
            os_id: "os-123".to_string(),
            hostname: "test.example.com".to_string(),
            ssh_key_ids: vec!["ssh-key-123".to_string()],
            user: Some("ubuntu".to_string()),
            password: None,
            service_user: None,
            service_password: None,
        }),
        option_ids: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("offer-123"));
    assert!(json.contains("test-server"));
    assert!(json.contains("test.example.com"));
}

#[test]
fn test_update_baremetal_server_request_serialization() {
    use crate::elastic_metal::UpdateBaremetalServerRequest;

    let request = UpdateBaremetalServerRequest {
        name: Some("updated-name".to_string()),
        description: Some("Updated description".to_string()),
        tags: Some(vec!["updated".to_string()]),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("updated-name"));
    assert!(json.contains("Updated description"));
}
