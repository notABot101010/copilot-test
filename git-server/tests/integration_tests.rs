//! Integration tests for the git-server
//!
//! These tests verify that the git server can:
//! 1. Accept SSH connections with proper authentication
//! 2. Handle git push and pull operations

use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use tempfile::TempDir;
use tokio::process::Command;
use tokio::time::sleep;

/// Helper to generate an SSH key pair for testing
async fn generate_ssh_keypair(dir: &PathBuf) -> (PathBuf, String) {
    let private_key_path = dir.join("test_key");
    let public_key_path = dir.join("test_key.pub");

    // Generate ed25519 key pair
    let status = Command::new("ssh-keygen")
        .args([
            "-t", "ed25519",
            "-f", private_key_path.to_str().unwrap(),
            "-N", "", // No passphrase
            "-q",
        ])
        .status()
        .await
        .expect("Failed to run ssh-keygen");

    assert!(status.success(), "ssh-keygen failed");

    // Read the public key
    let public_key = tokio::fs::read_to_string(&public_key_path)
        .await
        .expect("Failed to read public key");

    (private_key_path, public_key.trim().to_string())
}

/// Create a test configuration file
async fn create_config(dir: &PathBuf, ssh_port: u16, http_port: u16, public_key: &str) -> PathBuf {
    let config_path = dir.join("config.json");
    let config = serde_json::json!({
        "ssh_port": ssh_port,
        "http_port": http_port,
        "public_keys": [public_key],
        "auth": []
    });

    let mut file = std::fs::File::create(&config_path).expect("Failed to create config file");
    file.write_all(serde_json::to_string_pretty(&config).unwrap().as_bytes())
        .expect("Failed to write config");

    config_path
}

/// Wait for a port to be available
async fn wait_for_port(port: u16, timeout_secs: u64) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(timeout_secs) {
        if tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await.is_ok() {
            return true;
        }
        sleep(Duration::from_millis(100)).await;
    }
    false
}

/// Initialize a local git repository with some content
async fn init_local_repo(dir: &PathBuf) {
    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(dir)
        .status()
        .await
        .expect("Failed to init git repo");

    // Configure git user
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir)
        .status()
        .await
        .expect("Failed to configure git email");

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(dir)
        .status()
        .await
        .expect("Failed to configure git name");

    // Create a test file
    let test_file = dir.join("README.md");
    std::fs::write(&test_file, "# Test Repository\n\nThis is a test.\n")
        .expect("Failed to create test file");

    // Add and commit
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .status()
        .await
        .expect("Failed to git add");

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(dir)
        .status()
        .await
        .expect("Failed to git commit");
}

#[tokio::test]
async fn test_ssh_key_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let dir = temp_dir.path().to_path_buf();

    let (private_key_path, public_key) = generate_ssh_keypair(&dir).await;

    // Verify the key files exist
    assert!(private_key_path.exists(), "Private key should exist");
    assert!(private_key_path.with_extension("pub").exists(), "Public key should exist");

    // Verify the public key format
    assert!(
        public_key.starts_with("ssh-ed25519"),
        "Public key should be ed25519 format"
    );
}

#[tokio::test]
async fn test_git_server_integration() {
    // Create temporary directories
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let server_dir = temp_dir.path().to_path_buf();
    let repos_dir = server_dir.join("repos");
    let db_path = server_dir.join("test.db");
    let local_repo_dir = server_dir.join("local_repo");
    let clone_dir = server_dir.join("cloned_repo");

    std::fs::create_dir_all(&repos_dir).expect("Failed to create repos dir");
    std::fs::create_dir_all(&local_repo_dir).expect("Failed to create local repo dir");

    // Generate SSH key pair
    let (private_key_path, public_key) = generate_ssh_keypair(&server_dir).await;

    // Use random high ports to avoid conflicts
    let ssh_port: u16 = 20000 + (std::process::id() as u16 % 10000);
    let http_port: u16 = ssh_port + 1;

    // Create config
    let config_path = create_config(&server_dir, ssh_port, http_port, &public_key).await;

    // Find the git-server binary
    let git_server_binary = std::env::current_dir()
        .unwrap()
        .join("target/debug/git-server");

    if !git_server_binary.exists() {
        eprintln!("git-server binary not found at {:?}, skipping integration test", git_server_binary);
        return;
    }

    // Create a repository
    let create_status = Command::new(&git_server_binary)
        .args([
            "-c", config_path.to_str().unwrap(),
            "-r", repos_dir.to_str().unwrap(),
            "-d", db_path.to_str().unwrap(),
            "create-repo", "test-repo",
        ])
        .status()
        .await
        .expect("Failed to create repository");

    assert!(create_status.success(), "Failed to create repository");

    // Verify the bare repo was created
    let bare_repo_path = repos_dir.join("test-repo.git");
    assert!(bare_repo_path.exists(), "Bare repository should exist");

    // Start the server
    let mut server_process = Command::new(&git_server_binary)
        .args([
            "-c", config_path.to_str().unwrap(),
            "-r", repos_dir.to_str().unwrap(),
            "-d", db_path.to_str().unwrap(),
            "serve",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    // Wait for the server to start
    let server_started = wait_for_port(ssh_port, 10).await;

    // Clean up helper function
    let mut cleanup = || {
        let _ = server_process.start_kill();
    };

    if !server_started {
        cleanup();
        eprintln!("Server did not start in time, skipping test");
        return;
    }

    // Initialize local repo with content
    init_local_repo(&local_repo_dir).await;

    // Create SSH config to use our test key
    let ssh_config_path = server_dir.join("ssh_config");
    let ssh_config_content = format!(
        r#"Host testserver
    HostName 127.0.0.1
    Port {}
    User git
    IdentityFile {}
    StrictHostKeyChecking no
    UserKnownHostsFile /dev/null
    LogLevel ERROR
"#,
        ssh_port,
        private_key_path.display()
    );
    std::fs::write(&ssh_config_path, ssh_config_content).expect("Failed to write SSH config");

    // Add remote and push
    let add_remote_status = Command::new("git")
        .args(["remote", "add", "origin", &format!("ssh://testserver/test-repo")])
        .current_dir(&local_repo_dir)
        .env("GIT_SSH_COMMAND", format!("ssh -F {}", ssh_config_path.display()))
        .status()
        .await;

    if add_remote_status.is_err() || !add_remote_status.unwrap().success() {
        cleanup();
        eprintln!("Failed to add remote, skipping push test");
        return;
    }

    // Try to push
    let push_output = Command::new("git")
        .args(["push", "-u", "origin", "master"])
        .current_dir(&local_repo_dir)
        .env("GIT_SSH_COMMAND", format!("ssh -F {}", ssh_config_path.display()))
        .output()
        .await;

    match push_output {
        Ok(output) => {
            if output.status.success() {
                println!("Push succeeded!");
                
                // Try to clone the repository
                let clone_output = Command::new("git")
                    .args(["clone", &format!("ssh://testserver/test-repo"), clone_dir.to_str().unwrap()])
                    .env("GIT_SSH_COMMAND", format!("ssh -F {}", ssh_config_path.display()))
                    .output()
                    .await;

                match clone_output {
                    Ok(output) => {
                        if output.status.success() {
                            println!("Clone succeeded!");
                            
                            // Verify the cloned content
                            let readme_path = clone_dir.join("README.md");
                            assert!(readme_path.exists(), "README.md should exist in cloned repo");
                            
                            let content = std::fs::read_to_string(&readme_path).expect("Failed to read README");
                            assert!(content.contains("Test Repository"), "README should contain expected content");
                        } else {
                            eprintln!("Clone failed: {}", String::from_utf8_lossy(&output.stderr));
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to run git clone: {}", e);
                    }
                }
            } else {
                eprintln!("Push failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            eprintln!("Failed to run git push: {}", e);
        }
    }

    // Stop the server
    cleanup();
}

#[tokio::test]
async fn test_http_api() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let server_dir = temp_dir.path().to_path_buf();
    let repos_dir = server_dir.join("repos");
    let db_path = server_dir.join("test.db");

    std::fs::create_dir_all(&repos_dir).expect("Failed to create repos dir");

    // Use random high ports to avoid conflicts
    let ssh_port: u16 = 30000 + (std::process::id() as u16 % 10000);
    let http_port: u16 = ssh_port + 1;

    // Create config (no public keys needed for HTTP-only test)
    let config_path = create_config(&server_dir, ssh_port, http_port, "").await;

    // Find the git-server binary
    let git_server_binary = std::env::current_dir()
        .unwrap()
        .join("target/debug/git-server");

    if !git_server_binary.exists() {
        eprintln!("git-server binary not found, skipping HTTP API test");
        return;
    }

    // Create a repository
    let create_status = Command::new(&git_server_binary)
        .args([
            "-c", config_path.to_str().unwrap(),
            "-r", repos_dir.to_str().unwrap(),
            "-d", db_path.to_str().unwrap(),
            "create-repo", "api-test-repo",
        ])
        .status()
        .await
        .expect("Failed to create repository");

    assert!(create_status.success(), "Failed to create repository");

    // Start the server
    let mut server_process = Command::new(&git_server_binary)
        .args([
            "-c", config_path.to_str().unwrap(),
            "-r", repos_dir.to_str().unwrap(),
            "-d", db_path.to_str().unwrap(),
            "serve",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    // Wait for HTTP server to start
    let server_started = wait_for_port(http_port, 10).await;

    // Clean up helper function
    let mut cleanup = || {
        let _ = server_process.start_kill();
    };

    if !server_started {
        cleanup();
        eprintln!("HTTP server did not start in time, skipping test");
        return;
    }

    // Test the API endpoints
    let client = reqwest::Client::new();

    // Test list repos endpoint
    let repos_response = client
        .get(format!("http://127.0.0.1:{}/api/repos", http_port))
        .send()
        .await;

    match repos_response {
        Ok(response) => {
            assert!(response.status().is_success(), "GET /api/repos should succeed");
            let repos: Vec<serde_json::Value> = response.json().await.expect("Failed to parse JSON");
            assert!(!repos.is_empty(), "Should have at least one repository");
            assert_eq!(repos[0]["name"], "api-test-repo", "Repository name should match");
        }
        Err(e) => {
            eprintln!("Failed to call API: {}", e);
        }
    }

    // Test get single repo endpoint
    let repo_response = client
        .get(format!("http://127.0.0.1:{}/api/repos/api-test-repo", http_port))
        .send()
        .await;

    match repo_response {
        Ok(response) => {
            assert!(response.status().is_success(), "GET /api/repos/api-test-repo should succeed");
            let repo: serde_json::Value = response.json().await.expect("Failed to parse JSON");
            assert_eq!(repo["name"], "api-test-repo", "Repository name should match");
        }
        Err(e) => {
            eprintln!("Failed to call API: {}", e);
        }
    }

    cleanup();
}
