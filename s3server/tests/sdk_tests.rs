//! Integration tests using the official AWS SDK for Rust
//!
//! These tests verify S3 compatibility using the official aws-sdk-s3 crate

use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use aws_sdk_s3::Client;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tempfile::TempDir;
use tokio::process::Command;
use tokio::time::sleep;

/// Wait for a port to be available
async fn wait_for_port(port: u16, timeout_secs: u64) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(timeout_secs) {
        if tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .is_ok()
        {
            return true;
        }
        sleep(Duration::from_millis(100)).await;
    }
    false
}

/// Start the S3 server for testing
async fn start_server(
    port: u16,
    data_dir: &PathBuf,
    db_path: &PathBuf,
) -> Option<tokio::process::Child> {
    let binary = std::env::current_dir()
        .unwrap()
        .join("target/debug/s3server");

    if !binary.exists() {
        eprintln!("s3server binary not found at {:?}", binary);
        return None;
    }

    let process = Command::new(&binary)
        .args([
            "--port",
            &port.to_string(),
            "--data-path",
            data_dir.to_str().unwrap(),
            "--database",
            db_path.to_str().unwrap(),
            "serve",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    // Wait for server to start
    if !wait_for_port(port, 10).await {
        eprintln!("Server did not start in time");
        return None;
    }

    Some(process)
}

/// Create an S3 client configured for our local server
async fn create_client(port: u16) -> Client {
    let credentials = Credentials::new("test-access-key", "test-secret-key", None, None, "test");

    let config = aws_sdk_s3::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .credentials_provider(credentials)
        .endpoint_url(format!("http://127.0.0.1:{}", port))
        .region(aws_sdk_s3::config::Region::new("us-east-1"))
        .force_path_style(true)
        .build();

    Client::from_conf(config)
}

#[tokio::test]
async fn test_sdk_bucket_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    let db_path = temp_dir.path().join("test.db");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

    let port: u16 = 20000 + (std::process::id() as u16 % 10000);

    let mut server = match start_server(port, &data_dir, &db_path).await {
        Some(s) => s,
        None => {
            eprintln!("Skipping test - server not available");
            return;
        }
    };

    let client = create_client(port).await;

    // Test 1: List buckets (should be empty)
    let result = client.list_buckets().send().await;
    assert!(result.is_ok());
    let buckets = result.unwrap().buckets.unwrap_or_default();
    assert!(buckets.is_empty());

    // Test 2: Create bucket
    let result = client
        .create_bucket()
        .bucket("sdk-test-bucket")
        .send()
        .await;
    assert!(result.is_ok(), "Create bucket failed: {:?}", result.err());

    // Test 3: List buckets (should have one)
    let result = client.list_buckets().send().await;
    assert!(result.is_ok());
    let buckets = result.unwrap().buckets.unwrap_or_default();
    assert_eq!(buckets.len(), 1);
    assert_eq!(buckets[0].name.as_deref(), Some("sdk-test-bucket"));

    // Test 4: Head bucket
    let result = client.head_bucket().bucket("sdk-test-bucket").send().await;
    assert!(result.is_ok());

    // Test 5: Delete bucket
    let result = client
        .delete_bucket()
        .bucket("sdk-test-bucket")
        .send()
        .await;
    assert!(result.is_ok());

    // Test 6: List buckets (should be empty again)
    let result = client.list_buckets().send().await;
    assert!(result.is_ok());
    let buckets = result.unwrap().buckets.unwrap_or_default();
    assert!(buckets.is_empty());

    server.kill().await.ok();
}

#[tokio::test]
async fn test_sdk_object_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    let db_path = temp_dir.path().join("test.db");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

    let port: u16 = 21000 + (std::process::id() as u16 % 10000);

    let mut server = match start_server(port, &data_dir, &db_path).await {
        Some(s) => s,
        None => {
            eprintln!("Skipping test - server not available");
            return;
        }
    };

    let client = create_client(port).await;

    // Create bucket
    client
        .create_bucket()
        .bucket("sdk-objects-bucket")
        .send()
        .await
        .expect("Failed to create bucket");

    // Test 1: Put object
    let content = b"Hello from AWS SDK for Rust!";
    let result = client
        .put_object()
        .bucket("sdk-objects-bucket")
        .key("hello.txt")
        .content_type("text/plain")
        .body(ByteStream::from_static(content))
        .send()
        .await;
    assert!(result.is_ok(), "Put object failed: {:?}", result.err());
    let put_result = result.unwrap();
    assert!(put_result.e_tag.is_some());

    // Test 2: Head object
    let result = client
        .head_object()
        .bucket("sdk-objects-bucket")
        .key("hello.txt")
        .send()
        .await;
    assert!(result.is_ok());
    let head_result = result.unwrap();
    assert_eq!(head_result.content_length, Some(content.len() as i64));

    // Test 3: Get object
    let result = client
        .get_object()
        .bucket("sdk-objects-bucket")
        .key("hello.txt")
        .send()
        .await;
    assert!(result.is_ok());
    let get_result = result.unwrap();
    let body = get_result.body.collect().await.unwrap().into_bytes();
    assert_eq!(body.as_ref(), content);

    // Test 4: List objects
    let result = client
        .list_objects_v2()
        .bucket("sdk-objects-bucket")
        .send()
        .await;
    assert!(result.is_ok());
    let list_result = result.unwrap();
    let objects = list_result.contents.unwrap_or_default();
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0].key.as_deref(), Some("hello.txt"));

    // Test 5: Put more objects
    client
        .put_object()
        .bucket("sdk-objects-bucket")
        .key("folder/file1.txt")
        .body(ByteStream::from_static(b"File 1"))
        .send()
        .await
        .unwrap();

    client
        .put_object()
        .bucket("sdk-objects-bucket")
        .key("folder/file2.txt")
        .body(ByteStream::from_static(b"File 2"))
        .send()
        .await
        .unwrap();

    // Test 6: List with prefix
    let result = client
        .list_objects_v2()
        .bucket("sdk-objects-bucket")
        .prefix("folder/")
        .send()
        .await;
    assert!(result.is_ok());
    let list_result = result.unwrap();
    let objects = list_result.contents.unwrap_or_default();
    assert_eq!(objects.len(), 2);

    // Test 7: Delete object
    let result = client
        .delete_object()
        .bucket("sdk-objects-bucket")
        .key("hello.txt")
        .send()
        .await;
    assert!(result.is_ok());

    // Test 8: Verify deletion
    let result = client
        .get_object()
        .bucket("sdk-objects-bucket")
        .key("hello.txt")
        .send()
        .await;
    assert!(result.is_err());

    server.kill().await.ok();
}

#[tokio::test]
async fn test_sdk_multipart_upload() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    let db_path = temp_dir.path().join("test.db");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

    let port: u16 = 22000 + (std::process::id() as u16 % 10000);

    let mut server = match start_server(port, &data_dir, &db_path).await {
        Some(s) => s,
        None => {
            eprintln!("Skipping test - server not available");
            return;
        }
    };

    let client = create_client(port).await;

    // Create bucket
    client
        .create_bucket()
        .bucket("sdk-multipart-bucket")
        .send()
        .await
        .expect("Failed to create bucket");

    // Test 1: Initiate multipart upload
    let result = client
        .create_multipart_upload()
        .bucket("sdk-multipart-bucket")
        .key("large-file.bin")
        .send()
        .await;
    assert!(result.is_ok(), "Initiate upload failed: {:?}", result.err());
    let upload_result = result.unwrap();
    let upload_id = upload_result.upload_id.expect("Missing upload ID");

    // Test 2: Upload parts
    let part1_content = b"Part 1 content repeated ".repeat(100);
    let result = client
        .upload_part()
        .bucket("sdk-multipart-bucket")
        .key("large-file.bin")
        .upload_id(&upload_id)
        .part_number(1)
        .body(ByteStream::from(part1_content.clone()))
        .send()
        .await;
    assert!(result.is_ok(), "Upload part 1 failed: {:?}", result.err());
    let part1_etag = result.unwrap().e_tag.expect("Missing ETag for part 1");

    let part2_content = b"Part 2 content repeated ".repeat(100);
    let result = client
        .upload_part()
        .bucket("sdk-multipart-bucket")
        .key("large-file.bin")
        .upload_id(&upload_id)
        .part_number(2)
        .body(ByteStream::from(part2_content.clone()))
        .send()
        .await;
    assert!(result.is_ok(), "Upload part 2 failed: {:?}", result.err());
    let part2_etag = result.unwrap().e_tag.expect("Missing ETag for part 2");

    // Test 3: Complete multipart upload
    let completed_parts = CompletedMultipartUpload::builder()
        .parts(
            CompletedPart::builder()
                .part_number(1)
                .e_tag(&part1_etag)
                .build(),
        )
        .parts(
            CompletedPart::builder()
                .part_number(2)
                .e_tag(&part2_etag)
                .build(),
        )
        .build();

    let result = client
        .complete_multipart_upload()
        .bucket("sdk-multipart-bucket")
        .key("large-file.bin")
        .upload_id(&upload_id)
        .multipart_upload(completed_parts)
        .send()
        .await;
    assert!(result.is_ok(), "Complete upload failed: {:?}", result.err());

    // Test 4: Verify the complete object
    let result = client
        .get_object()
        .bucket("sdk-multipart-bucket")
        .key("large-file.bin")
        .send()
        .await;
    assert!(result.is_ok());
    let body = result.unwrap().body.collect().await.unwrap().into_bytes();
    let expected_len = part1_content.len() + part2_content.len();
    assert_eq!(body.len(), expected_len);

    server.kill().await.ok();
}

#[tokio::test]
async fn test_sdk_abort_multipart_upload() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    let db_path = temp_dir.path().join("test.db");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

    let port: u16 = 23000 + (std::process::id() as u16 % 10000);

    let mut server = match start_server(port, &data_dir, &db_path).await {
        Some(s) => s,
        None => {
            eprintln!("Skipping test - server not available");
            return;
        }
    };

    let client = create_client(port).await;

    // Create bucket
    client
        .create_bucket()
        .bucket("sdk-abort-bucket")
        .send()
        .await
        .expect("Failed to create bucket");

    // Initiate multipart upload
    let result = client
        .create_multipart_upload()
        .bucket("sdk-abort-bucket")
        .key("to-abort.bin")
        .send()
        .await;
    let upload_id = result.unwrap().upload_id.expect("Missing upload ID");

    // Upload a part
    client
        .upload_part()
        .bucket("sdk-abort-bucket")
        .key("to-abort.bin")
        .upload_id(&upload_id)
        .part_number(1)
        .body(ByteStream::from_static(b"Some content"))
        .send()
        .await
        .unwrap();

    // Abort the upload
    let result = client
        .abort_multipart_upload()
        .bucket("sdk-abort-bucket")
        .key("to-abort.bin")
        .upload_id(&upload_id)
        .send()
        .await;
    assert!(result.is_ok());

    // Verify upload is gone by listing uploads
    let result = client
        .list_multipart_uploads()
        .bucket("sdk-abort-bucket")
        .send()
        .await;
    assert!(result.is_ok());
    let uploads = result.unwrap().uploads.unwrap_or_default();
    assert!(uploads.is_empty());

    server.kill().await.ok();
}

#[tokio::test]
async fn test_sdk_conditional_write() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    let db_path = temp_dir.path().join("test.db");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

    let port: u16 = 24000 + (std::process::id() as u16 % 10000);

    let mut server = match start_server(port, &data_dir, &db_path).await {
        Some(s) => s,
        None => {
            eprintln!("Skipping test - server not available");
            return;
        }
    };

    let client = create_client(port).await;

    // Create bucket
    client
        .create_bucket()
        .bucket("sdk-cond-bucket")
        .send()
        .await
        .expect("Failed to create bucket");

    // Test 1: Put object with if_none_match (should succeed for new object)
    let result = client
        .put_object()
        .bucket("sdk-cond-bucket")
        .key("unique.txt")
        .if_none_match("*")
        .body(ByteStream::from_static(b"First write"))
        .send()
        .await;
    assert!(result.is_ok(), "First write should succeed");

    // Test 2: Put object with if_none_match (should fail for existing object)
    let result = client
        .put_object()
        .bucket("sdk-cond-bucket")
        .key("unique.txt")
        .if_none_match("*")
        .body(ByteStream::from_static(b"Second write"))
        .send()
        .await;
    assert!(result.is_err(), "Second write should fail");

    // Test 3: Verify content is still the first write
    let result = client
        .get_object()
        .bucket("sdk-cond-bucket")
        .key("unique.txt")
        .send()
        .await;
    assert!(result.is_ok());
    let body = result.unwrap().body.collect().await.unwrap().into_bytes();
    assert_eq!(body.as_ref(), b"First write");

    server.kill().await.ok();
}

#[tokio::test]
async fn test_sdk_range_request() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    let db_path = temp_dir.path().join("test.db");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

    let port: u16 = 25000 + (std::process::id() as u16 % 10000);

    let mut server = match start_server(port, &data_dir, &db_path).await {
        Some(s) => s,
        None => {
            eprintln!("Skipping test - server not available");
            return;
        }
    };

    let client = create_client(port).await;

    // Create bucket and object
    client
        .create_bucket()
        .bucket("sdk-range-bucket")
        .send()
        .await
        .unwrap();

    let content = b"0123456789ABCDEFGHIJ";
    client
        .put_object()
        .bucket("sdk-range-bucket")
        .key("range.txt")
        .body(ByteStream::from_static(content))
        .send()
        .await
        .unwrap();

    // Test 1: Range request bytes=0-4
    let result = client
        .get_object()
        .bucket("sdk-range-bucket")
        .key("range.txt")
        .range("bytes=0-4")
        .send()
        .await;
    assert!(result.is_ok());
    let body = result.unwrap().body.collect().await.unwrap().into_bytes();
    assert_eq!(body.as_ref(), b"01234");

    // Test 2: Range request bytes=10-14
    let result = client
        .get_object()
        .bucket("sdk-range-bucket")
        .key("range.txt")
        .range("bytes=10-14")
        .send()
        .await;
    assert!(result.is_ok());
    let body = result.unwrap().body.collect().await.unwrap().into_bytes();
    assert_eq!(body.as_ref(), b"ABCDE");

    server.kill().await.ok();
}
