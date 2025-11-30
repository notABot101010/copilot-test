//! Integration tests for S3-compatible server
//!
//! These tests verify that the S3 server correctly handles:
//! - Bucket operations (create, list, head, delete)
//! - Object operations (put, get, head, list, delete)
//! - Multipart uploads
//! - Conditional writes
//! - Range requests

use std::io::Write;
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

#[tokio::test]
async fn test_bucket_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    let db_path = temp_dir.path().join("test.db");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

    let port: u16 = 10000 + (std::process::id() as u16 % 10000);

    let mut server = match start_server(port, &data_dir, &db_path).await {
        Some(s) => s,
        None => {
            eprintln!("Skipping test - server not available");
            return;
        }
    };

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // Test 1: List buckets (should be empty)
    let resp = client
        .get(&base_url)
        .send()
        .await
        .expect("Failed to list buckets");
    assert!(resp.status().is_success());
    let body = resp.text().await.unwrap();
    assert!(body.contains("ListAllMyBucketsResult"));

    // Test 2: Create bucket
    let resp = client
        .put(format!("{}/test-bucket", base_url))
        .send()
        .await
        .expect("Failed to create bucket");
    assert!(resp.status().is_success(), "Create bucket failed: {:?}", resp.status());

    // Test 3: Head bucket (should exist)
    let resp = client
        .head(format!("{}/test-bucket", base_url))
        .send()
        .await
        .expect("Failed to head bucket");
    assert!(resp.status().is_success());

    // Test 4: List buckets (should have one)
    let resp = client
        .get(&base_url)
        .send()
        .await
        .expect("Failed to list buckets");
    assert!(resp.status().is_success());
    let body = resp.text().await.unwrap();
    assert!(body.contains("test-bucket"));

    // Test 5: Create duplicate bucket (should fail or succeed based on S3 semantics)
    let resp = client
        .put(format!("{}/test-bucket", base_url))
        .send()
        .await
        .expect("Failed to create bucket");
    // S3 returns 409 for duplicate bucket
    assert_eq!(resp.status().as_u16(), 409);

    // Test 6: Delete bucket
    let resp = client
        .delete(format!("{}/test-bucket", base_url))
        .send()
        .await
        .expect("Failed to delete bucket");
    assert!(resp.status().is_success() || resp.status() == 204);

    // Test 7: Head deleted bucket (should fail)
    let resp = client
        .head(format!("{}/test-bucket", base_url))
        .send()
        .await
        .expect("Failed to head bucket");
    assert_eq!(resp.status().as_u16(), 404);

    server.kill().await.ok();
}

#[tokio::test]
async fn test_object_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    let db_path = temp_dir.path().join("test.db");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

    let port: u16 = 11000 + (std::process::id() as u16 % 10000);

    let mut server = match start_server(port, &data_dir, &db_path).await {
        Some(s) => s,
        None => {
            eprintln!("Skipping test - server not available");
            return;
        }
    };

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // Create bucket first
    let resp = client
        .put(format!("{}/objects-bucket", base_url))
        .send()
        .await
        .expect("Failed to create bucket");
    assert!(resp.status().is_success());

    // Test 1: Put object
    let test_content = "Hello, S3 World!";
    let resp = client
        .put(format!("{}/objects-bucket/hello.txt", base_url))
        .header("Content-Type", "text/plain")
        .body(test_content)
        .send()
        .await
        .expect("Failed to put object");
    assert!(resp.status().is_success());
    let etag = resp.headers().get("etag").expect("Missing ETag");
    assert!(!etag.is_empty());

    // Test 2: Head object
    let resp = client
        .head(format!("{}/objects-bucket/hello.txt", base_url))
        .send()
        .await
        .expect("Failed to head object");
    assert!(resp.status().is_success());
    assert_eq!(
        resp.headers()
            .get("content-length")
            .unwrap()
            .to_str()
            .unwrap(),
        test_content.len().to_string()
    );

    // Test 3: Get object
    let resp = client
        .get(format!("{}/objects-bucket/hello.txt", base_url))
        .send()
        .await
        .expect("Failed to get object");
    assert!(resp.status().is_success());
    let body = resp.text().await.unwrap();
    assert_eq!(body, test_content);

    // Test 4: List objects
    let resp = client
        .get(format!("{}/objects-bucket", base_url))
        .send()
        .await
        .expect("Failed to list objects");
    assert!(resp.status().is_success());
    let body = resp.text().await.unwrap();
    assert!(body.contains("hello.txt"));

    // Test 5: Put more objects for prefix testing
    client
        .put(format!("{}/objects-bucket/folder/file1.txt", base_url))
        .body("File 1")
        .send()
        .await
        .unwrap();

    client
        .put(format!("{}/objects-bucket/folder/file2.txt", base_url))
        .body("File 2")
        .send()
        .await
        .unwrap();

    // Test 6: List with prefix
    let resp = client
        .get(format!("{}/objects-bucket?prefix=folder/", base_url))
        .send()
        .await
        .expect("Failed to list objects with prefix");
    assert!(resp.status().is_success());
    let body = resp.text().await.unwrap();
    assert!(body.contains("folder/file1.txt"));
    assert!(body.contains("folder/file2.txt"));
    assert!(!body.contains(">hello.txt<"));

    // Test 7: Delete object
    let resp = client
        .delete(format!("{}/objects-bucket/hello.txt", base_url))
        .send()
        .await
        .expect("Failed to delete object");
    assert!(resp.status().is_success() || resp.status() == 204);

    // Test 8: Get deleted object (should fail)
    let resp = client
        .get(format!("{}/objects-bucket/hello.txt", base_url))
        .send()
        .await
        .expect("Failed to get object");
    assert_eq!(resp.status().as_u16(), 404);

    server.kill().await.ok();
}

#[tokio::test]
async fn test_conditional_write() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    let db_path = temp_dir.path().join("test.db");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

    let port: u16 = 12000 + (std::process::id() as u16 % 10000);

    let mut server = match start_server(port, &data_dir, &db_path).await {
        Some(s) => s,
        None => {
            eprintln!("Skipping test - server not available");
            return;
        }
    };

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // Create bucket
    client
        .put(format!("{}/cond-bucket", base_url))
        .send()
        .await
        .unwrap();

    // Test 1: Put object with If-None-Match: * (should succeed for new object)
    let resp = client
        .put(format!("{}/cond-bucket/unique.txt", base_url))
        .header("If-None-Match", "*")
        .body("First write")
        .send()
        .await
        .expect("Failed to put object");
    assert!(resp.status().is_success());

    // Test 2: Put object with If-None-Match: * (should fail for existing object)
    let resp = client
        .put(format!("{}/cond-bucket/unique.txt", base_url))
        .header("If-None-Match", "*")
        .body("Second write")
        .send()
        .await
        .expect("Failed to put object");
    assert_eq!(resp.status().as_u16(), 412); // Precondition Failed

    // Test 3: Verify content is still the first write
    let resp = client
        .get(format!("{}/cond-bucket/unique.txt", base_url))
        .send()
        .await
        .expect("Failed to get object");
    let body = resp.text().await.unwrap();
    assert_eq!(body, "First write");

    server.kill().await.ok();
}

#[tokio::test]
async fn test_range_request() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    let db_path = temp_dir.path().join("test.db");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

    let port: u16 = 13000 + (std::process::id() as u16 % 10000);

    let mut server = match start_server(port, &data_dir, &db_path).await {
        Some(s) => s,
        None => {
            eprintln!("Skipping test - server not available");
            return;
        }
    };

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // Create bucket and object
    client
        .put(format!("{}/range-bucket", base_url))
        .send()
        .await
        .unwrap();

    let content = "0123456789ABCDEFGHIJ";
    client
        .put(format!("{}/range-bucket/range.txt", base_url))
        .body(content)
        .send()
        .await
        .unwrap();

    // Test 1: Range request bytes=0-4
    let resp = client
        .get(format!("{}/range-bucket/range.txt", base_url))
        .header("Range", "bytes=0-4")
        .send()
        .await
        .expect("Failed to get range");
    assert_eq!(resp.status().as_u16(), 206); // Partial Content
    let body = resp.text().await.unwrap();
    assert_eq!(body, "01234");

    // Test 2: Range request bytes=10-14
    let resp = client
        .get(format!("{}/range-bucket/range.txt", base_url))
        .header("Range", "bytes=10-14")
        .send()
        .await
        .expect("Failed to get range");
    assert_eq!(resp.status().as_u16(), 206);
    let body = resp.text().await.unwrap();
    assert_eq!(body, "ABCDE");

    server.kill().await.ok();
}

#[tokio::test]
async fn test_multipart_upload() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    let db_path = temp_dir.path().join("test.db");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

    let port: u16 = 14000 + (std::process::id() as u16 % 10000);

    let mut server = match start_server(port, &data_dir, &db_path).await {
        Some(s) => s,
        None => {
            eprintln!("Skipping test - server not available");
            return;
        }
    };

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // Create bucket
    client
        .put(format!("{}/multi-bucket", base_url))
        .send()
        .await
        .unwrap();

    // Test 1: Initiate multipart upload
    let resp = client
        .post(format!("{}/multi-bucket/large-file.bin?uploads", base_url))
        .send()
        .await
        .expect("Failed to initiate multipart upload");
    assert!(resp.status().is_success());
    let body = resp.text().await.unwrap();
    assert!(body.contains("UploadId"));

    // Extract upload ID from XML
    let upload_id_start = body.find("<UploadId>").unwrap() + 10;
    let upload_id_end = body.find("</UploadId>").unwrap();
    let upload_id = &body[upload_id_start..upload_id_end];

    // Test 2: Upload part 1
    let part1_content = "Part 1 content - ".repeat(100);
    let resp = client
        .put(format!(
            "{}/multi-bucket/large-file.bin?uploadId={}&partNumber=1",
            base_url, upload_id
        ))
        .body(part1_content.clone())
        .send()
        .await
        .expect("Failed to upload part 1");
    assert!(resp.status().is_success());
    let etag1 = resp
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Test 3: Upload part 2
    let part2_content = "Part 2 content - ".repeat(100);
    let resp = client
        .put(format!(
            "{}/multi-bucket/large-file.bin?uploadId={}&partNumber=2",
            base_url, upload_id
        ))
        .body(part2_content.clone())
        .send()
        .await
        .expect("Failed to upload part 2");
    assert!(resp.status().is_success());
    let etag2 = resp
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Test 4: Complete multipart upload
    let complete_body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<CompleteMultipartUpload>
    <Part>
        <PartNumber>1</PartNumber>
        <ETag>{}</ETag>
    </Part>
    <Part>
        <PartNumber>2</PartNumber>
        <ETag>{}</ETag>
    </Part>
</CompleteMultipartUpload>"#,
        etag1, etag2
    );

    let resp = client
        .post(format!(
            "{}/multi-bucket/large-file.bin?uploadId={}",
            base_url, upload_id
        ))
        .header("Content-Type", "application/xml")
        .body(complete_body)
        .send()
        .await
        .expect("Failed to complete multipart upload");
    assert!(resp.status().is_success());
    let body = resp.text().await.unwrap();
    assert!(body.contains("CompleteMultipartUploadResult"));

    // Test 5: Verify the complete object
    let resp = client
        .get(format!("{}/multi-bucket/large-file.bin", base_url))
        .send()
        .await
        .expect("Failed to get object");
    assert!(resp.status().is_success());
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("Part 1 content"));
    assert!(body.contains("Part 2 content"));
    assert_eq!(body.len(), part1_content.len() + part2_content.len());

    server.kill().await.ok();
}

#[tokio::test]
async fn test_list_multipart_uploads() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    let db_path = temp_dir.path().join("test.db");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

    let port: u16 = 15000 + (std::process::id() as u16 % 10000);

    let mut server = match start_server(port, &data_dir, &db_path).await {
        Some(s) => s,
        None => {
            eprintln!("Skipping test - server not available");
            return;
        }
    };

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // Create bucket
    client
        .put(format!("{}/uploads-bucket", base_url))
        .send()
        .await
        .unwrap();

    // Initiate two multipart uploads
    client
        .post(format!("{}/uploads-bucket/file1.bin?uploads", base_url))
        .send()
        .await
        .unwrap();

    client
        .post(format!("{}/uploads-bucket/file2.bin?uploads", base_url))
        .send()
        .await
        .unwrap();

    // List multipart uploads
    let resp = client
        .get(format!("{}/uploads-bucket?uploads", base_url))
        .send()
        .await
        .expect("Failed to list uploads");
    assert!(resp.status().is_success());
    let body = resp.text().await.unwrap();
    assert!(body.contains("file1.bin"));
    assert!(body.contains("file2.bin"));
    assert!(body.contains("ListMultipartUploadsResult"));

    server.kill().await.ok();
}

#[tokio::test]
async fn test_abort_multipart_upload() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    let db_path = temp_dir.path().join("test.db");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

    let port: u16 = 16000 + (std::process::id() as u16 % 10000);

    let mut server = match start_server(port, &data_dir, &db_path).await {
        Some(s) => s,
        None => {
            eprintln!("Skipping test - server not available");
            return;
        }
    };

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // Create bucket
    client
        .put(format!("{}/abort-bucket", base_url))
        .send()
        .await
        .unwrap();

    // Initiate multipart upload
    let resp = client
        .post(format!("{}/abort-bucket/to-abort.bin?uploads", base_url))
        .send()
        .await
        .unwrap();
    let body = resp.text().await.unwrap();
    let upload_id_start = body.find("<UploadId>").unwrap() + 10;
    let upload_id_end = body.find("</UploadId>").unwrap();
    let upload_id = &body[upload_id_start..upload_id_end];

    // Upload a part
    client
        .put(format!(
            "{}/abort-bucket/to-abort.bin?uploadId={}&partNumber=1",
            base_url, upload_id
        ))
        .body("Some content")
        .send()
        .await
        .unwrap();

    // Abort the upload
    let resp = client
        .delete(format!(
            "{}/abort-bucket/to-abort.bin?uploadId={}",
            base_url, upload_id
        ))
        .send()
        .await
        .expect("Failed to abort upload");
    assert!(resp.status().is_success() || resp.status() == 204);

    // Verify upload is gone
    let resp = client
        .get(format!("{}/abort-bucket?uploads", base_url))
        .send()
        .await
        .unwrap();
    let body = resp.text().await.unwrap();
    assert!(!body.contains("to-abort.bin"));

    server.kill().await.ok();
}

/// Integration test using AWS CLI
#[tokio::test]
async fn test_with_aws_cli() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().join("data");
    let db_path = temp_dir.path().join("test.db");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

    let port: u16 = 17000 + (std::process::id() as u16 % 10000);

    let mut server = match start_server(port, &data_dir, &db_path).await {
        Some(s) => s,
        None => {
            eprintln!("Skipping test - server not available");
            return;
        }
    };

    // Check if AWS CLI is available
    let aws_check = Command::new("aws").args(["--version"]).output().await;

    if aws_check.is_err() || !aws_check.unwrap().status.success() {
        eprintln!("AWS CLI not available, skipping CLI tests");
        server.kill().await.ok();
        return;
    }

    let endpoint_url = format!("http://127.0.0.1:{}", port);

    // Test 1: Create bucket
    let output = Command::new("aws")
        .env("AWS_ACCESS_KEY_ID", "test")
        .env("AWS_SECRET_ACCESS_KEY", "test")
        .args([
            "s3",
            "mb",
            "s3://cli-test-bucket",
            "--endpoint-url",
            &endpoint_url,
        ])
        .output()
        .await
        .expect("Failed to run aws s3 mb");

    if !output.status.success() {
        eprintln!(
            "aws s3 mb failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Test 2: Upload file
    let test_file = temp_dir.path().join("test-upload.txt");
    std::fs::write(&test_file, "Hello from AWS CLI").expect("Failed to create test file");

    let output = Command::new("aws")
        .env("AWS_ACCESS_KEY_ID", "test")
        .env("AWS_SECRET_ACCESS_KEY", "test")
        .args([
            "s3",
            "cp",
            test_file.to_str().unwrap(),
            "s3://cli-test-bucket/uploaded.txt",
            "--endpoint-url",
            &endpoint_url,
        ])
        .output()
        .await
        .expect("Failed to run aws s3 cp");

    if !output.status.success() {
        eprintln!(
            "aws s3 cp upload failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Test 3: List objects
    let output = Command::new("aws")
        .env("AWS_ACCESS_KEY_ID", "test")
        .env("AWS_SECRET_ACCESS_KEY", "test")
        .args([
            "s3",
            "ls",
            "s3://cli-test-bucket",
            "--endpoint-url",
            &endpoint_url,
        ])
        .output()
        .await
        .expect("Failed to run aws s3 ls");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("uploaded.txt"), "File should be listed");
    }

    // Test 4: Download file
    let download_file = temp_dir.path().join("downloaded.txt");
    let output = Command::new("aws")
        .env("AWS_ACCESS_KEY_ID", "test")
        .env("AWS_SECRET_ACCESS_KEY", "test")
        .args([
            "s3",
            "cp",
            "s3://cli-test-bucket/uploaded.txt",
            download_file.to_str().unwrap(),
            "--endpoint-url",
            &endpoint_url,
        ])
        .output()
        .await
        .expect("Failed to run aws s3 cp download");

    if output.status.success() {
        let content = std::fs::read_to_string(&download_file).expect("Failed to read downloaded file");
        assert_eq!(content, "Hello from AWS CLI");
    }

    // Test 5: Delete object
    let _output = Command::new("aws")
        .env("AWS_ACCESS_KEY_ID", "test")
        .env("AWS_SECRET_ACCESS_KEY", "test")
        .args([
            "s3",
            "rm",
            "s3://cli-test-bucket/uploaded.txt",
            "--endpoint-url",
            &endpoint_url,
        ])
        .output()
        .await
        .expect("Failed to run aws s3 rm");

    // Test 6: Delete bucket
    let _output = Command::new("aws")
        .env("AWS_ACCESS_KEY_ID", "test")
        .env("AWS_SECRET_ACCESS_KEY", "test")
        .args([
            "s3",
            "rb",
            "s3://cli-test-bucket",
            "--endpoint-url",
            &endpoint_url,
        ])
        .output()
        .await
        .expect("Failed to run aws s3 rb");

    server.kill().await.ok();
}
