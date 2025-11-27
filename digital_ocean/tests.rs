//! Tests for the DigitalOcean API client.

use super::*;

#[test]
fn test_client_creation() {
    let client = Client::new(reqwest::Client::new(), "test-token".to_string());
    assert_eq!(client.access_token, "test-token");
}

#[test]
fn test_api_base_url() {
    assert_eq!(API_BASE_URL, "https://api.digitalocean.com/v2");
}

#[test]
fn test_api_error_display() {
    let error = ApiError {
        id: "not_found".to_string(),
        message: "Resource not found".to_string(),
        request_id: Some("abc123".to_string()),
    };
    assert_eq!(format!("{}", error), "[not_found] Resource not found");
}

#[test]
fn test_error_display() {
    let api_error = ApiError {
        id: "server_error".to_string(),
        message: "Internal error".to_string(),
        request_id: None,
    };
    let error = Error::API(500, api_error);
    let display = format!("{}", error);
    assert!(display.contains("Internal error"));
}

#[test]
fn test_droplet_image_serialization() {
    let image_id = DropletImage::Id(12345);
    let json = serde_json::to_string(&image_id).unwrap();
    assert_eq!(json, "12345");

    let image_slug = DropletImage::Slug("ubuntu-22-04-x64".to_string());
    let json = serde_json::to_string(&image_slug).unwrap();
    assert_eq!(json, "\"ubuntu-22-04-x64\"");
}

#[test]
fn test_ssh_key_identifier_serialization() {
    let key_id = SshKeyIdentifier::Id(67890);
    let json = serde_json::to_string(&key_id).unwrap();
    assert_eq!(json, "67890");

    let key_fp = SshKeyIdentifier::Fingerprint("aa:bb:cc:dd:ee:ff".to_string());
    let json = serde_json::to_string(&key_fp).unwrap();
    assert_eq!(json, "\"aa:bb:cc:dd:ee:ff\"");
}

#[test]
fn test_create_droplet_request_serialization() {
    let request = CreateDropletRequest {
        name: "my-droplet".to_string(),
        region: "nyc1".to_string(),
        size: "s-1vcpu-1gb".to_string(),
        image: DropletImage::Slug("ubuntu-22-04-x64".to_string()),
        ssh_keys: Some(vec![SshKeyIdentifier::Id(12345)]),
        backups: Some(false),
        ipv6: Some(true),
        monitoring: Some(true),
        user_data: None,
        tags: Some(vec!["web".to_string()]),
        vpc_uuid: None,
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["name"], "my-droplet");
    assert_eq!(json["region"], "nyc1");
    assert_eq!(json["size"], "s-1vcpu-1gb");
    assert_eq!(json["image"], "ubuntu-22-04-x64");
    assert_eq!(json["backups"], false);
    assert_eq!(json["ipv6"], true);
    assert_eq!(json["monitoring"], true);
    assert_eq!(json["tags"][0], "web");
}

#[test]
fn test_create_ssh_key_request_serialization() {
    let request = CreateSshKeyRequest {
        name: "my-key".to_string(),
        public_key: "ssh-rsa AAAA...".to_string(),
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["name"], "my-key");
    assert_eq!(json["public_key"], "ssh-rsa AAAA...");
}

#[test]
fn test_create_volume_request_serialization() {
    let request = CreateVolumeRequest {
        size_gigabytes: 100,
        name: "my-volume".to_string(),
        description: Some("A test volume".to_string()),
        region: "nyc1".to_string(),
        filesystem_type: Some("ext4".to_string()),
        filesystem_label: Some("my-volume".to_string()),
        tags: Some(vec!["database".to_string()]),
        snapshot_id: None,
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["size_gigabytes"], 100);
    assert_eq!(json["name"], "my-volume");
    assert_eq!(json["description"], "A test volume");
    assert_eq!(json["region"], "nyc1");
    assert_eq!(json["filesystem_type"], "ext4");
}

#[test]
fn test_create_domain_request_serialization() {
    let request = CreateDomainRequest {
        name: "example.com".to_string(),
        ip_address: Some("1.2.3.4".to_string()),
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["name"], "example.com");
    assert_eq!(json["ip_address"], "1.2.3.4");
}

#[test]
fn test_create_domain_record_request_serialization() {
    let request = CreateDomainRecordRequest {
        record_type: "A".to_string(),
        name: "www".to_string(),
        data: "1.2.3.4".to_string(),
        priority: None,
        port: None,
        ttl: Some(3600),
        weight: None,
        flags: None,
        tag: None,
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["type"], "A");
    assert_eq!(json["name"], "www");
    assert_eq!(json["data"], "1.2.3.4");
    assert_eq!(json["ttl"], 3600);
    assert!(json.get("priority").is_none());
}

#[test]
fn test_create_database_cluster_request_serialization() {
    let request = CreateDatabaseClusterRequest {
        name: "my-database".to_string(),
        engine: "pg".to_string(),
        version: "15".to_string(),
        size: "db-s-1vcpu-1gb".to_string(),
        region: "nyc1".to_string(),
        num_nodes: 1,
        tags: Some(vec!["production".to_string()]),
        private_network_uuid: None,
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["name"], "my-database");
    assert_eq!(json["engine"], "pg");
    assert_eq!(json["version"], "15");
    assert_eq!(json["size"], "db-s-1vcpu-1gb");
    assert_eq!(json["region"], "nyc1");
    assert_eq!(json["num_nodes"], 1);
}

#[test]
fn test_droplet_deserialization() {
    let json = r#"{
        "id": 12345,
        "name": "my-droplet",
        "memory": 1024,
        "vcpus": 1,
        "disk": 25,
        "locked": false,
        "status": "active",
        "size_slug": "s-1vcpu-1gb",
        "tags": ["web"],
        "volume_ids": [],
        "created_at": "2021-01-01T00:00:00Z"
    }"#;

    let droplet: Droplet = serde_json::from_str(json).unwrap();
    assert_eq!(droplet.id, 12345);
    assert_eq!(droplet.name, "my-droplet");
    assert_eq!(droplet.memory, 1024);
    assert_eq!(droplet.vcpus, 1);
    assert_eq!(droplet.disk, 25);
    assert!(!droplet.locked);
    assert_eq!(droplet.status, "active");
}

#[test]
fn test_ssh_key_deserialization() {
    let json = r#"{
        "id": 67890,
        "name": "my-key",
        "fingerprint": "aa:bb:cc:dd:ee:ff",
        "public_key": "ssh-rsa AAAA..."
    }"#;

    let key: SshKey = serde_json::from_str(json).unwrap();
    assert_eq!(key.id, 67890);
    assert_eq!(key.name, "my-key");
    assert_eq!(key.fingerprint, "aa:bb:cc:dd:ee:ff");
}

#[test]
fn test_domain_deserialization() {
    let json = r#"{
        "name": "example.com",
        "ttl": 1800,
        "zone_file": "..."
    }"#;

    let domain: Domain = serde_json::from_str(json).unwrap();
    assert_eq!(domain.name, "example.com");
    assert_eq!(domain.ttl, Some(1800));
}

#[test]
fn test_region_info_deserialization() {
    let json = r#"{
        "slug": "nyc1",
        "name": "New York 1",
        "sizes": ["s-1vcpu-1gb"],
        "available": true,
        "features": ["metadata", "install_agent"]
    }"#;

    let region: RegionInfo = serde_json::from_str(json).unwrap();
    assert_eq!(region.slug, "nyc1");
    assert_eq!(region.name, "New York 1");
    assert!(region.available);
}

#[test]
fn test_size_info_deserialization() {
    let json = r#"{
        "slug": "s-1vcpu-1gb",
        "memory": 1024,
        "vcpus": 1,
        "disk": 25,
        "transfer": 1.0,
        "price_monthly": 5.0,
        "price_hourly": 0.007,
        "regions": ["nyc1", "sfo1"],
        "available": true,
        "description": "Basic"
    }"#;

    let size: SizeInfo = serde_json::from_str(json).unwrap();
    assert_eq!(size.slug, "s-1vcpu-1gb");
    assert_eq!(size.memory, 1024);
    assert_eq!(size.vcpus, 1);
    assert_eq!(size.price_monthly, 5.0);
}

#[test]
fn test_account_deserialization() {
    let json = r#"{
        "droplet_limit": 25,
        "floating_ip_limit": 5,
        "reserved_ip_limit": 5,
        "volume_limit": 10,
        "email": "test@example.com",
        "uuid": "abc123",
        "email_verified": true,
        "status": "active",
        "status_message": ""
    }"#;

    let account: Account = serde_json::from_str(json).unwrap();
    assert_eq!(account.droplet_limit, 25);
    assert_eq!(account.email, "test@example.com");
    assert!(account.email_verified);
}

#[test]
fn test_database_cluster_deserialization() {
    let json = r#"{
        "id": "abc123",
        "name": "my-database",
        "engine": "pg",
        "version": "15",
        "num_nodes": 1,
        "size": "db-s-1vcpu-1gb",
        "region": "nyc1",
        "status": "online",
        "created_at": "2021-01-01T00:00:00Z"
    }"#;

    let cluster: DatabaseCluster = serde_json::from_str(json).unwrap();
    assert_eq!(cluster.id, "abc123");
    assert_eq!(cluster.name, "my-database");
    assert_eq!(cluster.engine, "pg");
    assert_eq!(cluster.status, "online");
}

#[test]
fn test_volume_deserialization() {
    let json = r#"{
        "id": "abc123",
        "name": "my-volume",
        "description": "A test volume",
        "size_gigabytes": 100,
        "created_at": "2021-01-01T00:00:00Z",
        "region": {
            "slug": "nyc1",
            "name": "New York 1",
            "sizes": ["s-1vcpu-1gb"],
            "available": true,
            "features": ["metadata"]
        },
        "droplet_ids": [12345],
        "tags": ["database"]
    }"#;

    let volume: Volume = serde_json::from_str(json).unwrap();
    assert_eq!(volume.id, "abc123");
    assert_eq!(volume.name, "my-volume");
    assert_eq!(volume.size_gigabytes, 100);
    assert_eq!(volume.region.slug, "nyc1");
}
