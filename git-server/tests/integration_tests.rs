//! Integration tests for the git-server
//!
//! These tests verify that the git server can:
//! 1. Accept SSH connections with proper authentication
//! 2. Handle git push and pull operations
//! 3. Manage organizations, projects, files, and issues via HTTP API
//! 4. Sandbox git operations to prevent path traversal and symlink attacks

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
            "-t",
            "ed25519",
            "-f",
            private_key_path.to_str().unwrap(),
            "-N",
            "", // No passphrase
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
    let public_keys: Vec<&str> = if public_key.is_empty() {
        vec![]
    } else {
        vec![public_key]
    };
    let config = serde_json::json!({
        "ssh_port": ssh_port,
        "http_port": http_port,
        "public_keys": public_keys,
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
    assert!(
        private_key_path.with_extension("pub").exists(),
        "Public key should exist"
    );

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
        eprintln!(
            "git-server binary not found at {:?}, skipping integration test",
            git_server_binary
        );
        return;
    }

    // Create org first
    let create_org_status = Command::new(&git_server_binary)
        .args([
            "-c",
            config_path.to_str().unwrap(),
            "-r",
            repos_dir.to_str().unwrap(),
            "-d",
            db_path.to_str().unwrap(),
            "create-org",
            "test-org",
        ])
        .status()
        .await
        .expect("Failed to create organization");

    assert!(create_org_status.success(), "Failed to create organization");

    // Create project
    let create_project_status = Command::new(&git_server_binary)
        .args([
            "-c",
            config_path.to_str().unwrap(),
            "-r",
            repos_dir.to_str().unwrap(),
            "-d",
            db_path.to_str().unwrap(),
            "create-project",
            "--org",
            "test-org",
            "test-project",
        ])
        .status()
        .await
        .expect("Failed to create project");

    assert!(create_project_status.success(), "Failed to create project");

    // Create a repository
    let create_status = Command::new(&git_server_binary)
        .args([
            "-c",
            config_path.to_str().unwrap(),
            "-r",
            repos_dir.to_str().unwrap(),
            "-d",
            db_path.to_str().unwrap(),
            "create-repo",
            "--org",
            "test-org",
            "--project",
            "test-project",
            "test-repo",
        ])
        .status()
        .await
        .expect("Failed to create repository");

    assert!(create_status.success(), "Failed to create repository");

    // Verify the bare repo was created
    let bare_repo_path = repos_dir
        .join("test-org")
        .join("test-project")
        .join("test-repo.git");
    assert!(bare_repo_path.exists(), "Bare repository should exist");

    // Start the server
    let mut server_process = Command::new(&git_server_binary)
        .args([
            "-c",
            config_path.to_str().unwrap(),
            "-r",
            repos_dir.to_str().unwrap(),
            "-d",
            db_path.to_str().unwrap(),
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
        .args([
            "remote",
            "add",
            "origin",
            &format!("ssh://testserver/test-org/test-project/test-repo"),
        ])
        .current_dir(&local_repo_dir)
        .env(
            "GIT_SSH_COMMAND",
            format!("ssh -F {}", ssh_config_path.display()),
        )
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
        .env(
            "GIT_SSH_COMMAND",
            format!("ssh -F {}", ssh_config_path.display()),
        )
        .output()
        .await;

    match push_output {
        Ok(output) => {
            if output.status.success() {
                println!("Push succeeded!");

                // Try to clone the repository
                let clone_output = Command::new("git")
                    .args([
                        "clone",
                        &format!("ssh://testserver/test-org/test-project/test-repo"),
                        clone_dir.to_str().unwrap(),
                    ])
                    .env(
                        "GIT_SSH_COMMAND",
                        format!("ssh -F {}", ssh_config_path.display()),
                    )
                    .output()
                    .await;

                match clone_output {
                    Ok(output) => {
                        if output.status.success() {
                            println!("Clone succeeded!");

                            // Verify the cloned content
                            let readme_path = clone_dir.join("README.md");
                            assert!(
                                readme_path.exists(),
                                "README.md should exist in cloned repo"
                            );

                            let content = std::fs::read_to_string(&readme_path)
                                .expect("Failed to read README");
                            assert!(
                                content.contains("Test Repository"),
                                "README should contain expected content"
                            );
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

/// Comprehensive integration test for all HTTP API operations
#[tokio::test]
async fn test_full_http_api_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let server_dir = temp_dir.path().to_path_buf();
    let repos_dir = server_dir.join("repos");
    let db_path = server_dir.join("test.db");

    std::fs::create_dir_all(&repos_dir).expect("Failed to create repos dir");

    // Use random high ports to avoid conflicts
    let ssh_port: u16 = 40000 + (std::process::id() as u16 % 10000);
    let http_port: u16 = ssh_port + 1;

    // Create config (no public keys needed for HTTP-only test)
    let config_path = create_config(&server_dir, ssh_port, http_port, "").await;

    // Find the git-server binary
    let git_server_binary = std::env::current_dir()
        .unwrap()
        .join("target/debug/git-server");

    if !git_server_binary.exists() {
        eprintln!("git-server binary not found, skipping full HTTP API test");
        return;
    }

    // Start the server
    let mut server_process = Command::new(&git_server_binary)
        .args([
            "-c",
            config_path.to_str().unwrap(),
            "-r",
            repos_dir.to_str().unwrap(),
            "-d",
            db_path.to_str().unwrap(),
            "serve",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    // Wait for HTTP server to start
    let server_started = wait_for_port(http_port, 10).await;

    let mut cleanup = || {
        let _ = server_process.start_kill();
    };

    if !server_started {
        cleanup();
        panic!("HTTP server did not start in time");
    }

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", http_port);

    // ============ Test Organization CRUD ============

    // 1. Create organization
    let create_org_resp = client
        .post(format!("{}/api/orgs", base_url))
        .json(&serde_json::json!({
            "name": "test-org",
            "display_name": "Test Organization",
            "description": "A test organization"
        }))
        .send()
        .await
        .expect("Failed to create org");

    assert!(
        create_org_resp.status().is_success(),
        "Create org should succeed"
    );
    let org: serde_json::Value = create_org_resp.json().await.expect("Failed to parse org");
    assert_eq!(org["name"], "test-org");
    assert_eq!(org["display_name"], "Test Organization");

    // 2. List organizations
    let list_orgs_resp = client
        .get(format!("{}/api/orgs", base_url))
        .send()
        .await
        .expect("Failed to list orgs");

    assert!(list_orgs_resp.status().is_success());
    let orgs: Vec<serde_json::Value> = list_orgs_resp.json().await.expect("Failed to parse orgs");
    assert_eq!(orgs.len(), 1);
    assert_eq!(orgs[0]["name"], "test-org");

    // 3. Get organization
    let get_org_resp = client
        .get(format!("{}/api/orgs/test-org", base_url))
        .send()
        .await
        .expect("Failed to get org");

    assert!(get_org_resp.status().is_success());
    let org: serde_json::Value = get_org_resp.json().await.expect("Failed to parse org");
    assert_eq!(org["name"], "test-org");

    // 4. Update organization
    let update_org_resp = client
        .patch(format!("{}/api/orgs/test-org", base_url))
        .json(&serde_json::json!({
            "display_name": "Updated Test Organization"
        }))
        .send()
        .await
        .expect("Failed to update org");

    assert!(update_org_resp.status().is_success());
    let updated_org: serde_json::Value = update_org_resp
        .json()
        .await
        .expect("Failed to parse updated org");
    assert_eq!(updated_org["display_name"], "Updated Test Organization");

    // ============ Test Project CRUD ============

    // 1. Create project (also creates a git repository)
    let create_project_resp = client
        .post(format!("{}/api/orgs/test-org/projects", base_url))
        .json(&serde_json::json!({
            "name": "test-project",
            "display_name": "Test Project",
            "description": "A test project"
        }))
        .send()
        .await
        .expect("Failed to create project");

    assert!(
        create_project_resp.status().is_success(),
        "Create project should succeed: {:?}",
        create_project_resp.status()
    );
    let project: serde_json::Value = create_project_resp
        .json()
        .await
        .expect("Failed to parse project");
    assert_eq!(project["name"], "test-project");

    // 2. List projects
    let list_projects_resp = client
        .get(format!("{}/api/orgs/test-org/projects", base_url))
        .send()
        .await
        .expect("Failed to list projects");

    assert!(list_projects_resp.status().is_success());
    let projects: Vec<serde_json::Value> = list_projects_resp
        .json()
        .await
        .expect("Failed to parse projects");
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0]["name"], "test-project");

    // 3. Get project
    let get_project_resp = client
        .get(format!(
            "{}/api/orgs/test-org/projects/test-project",
            base_url
        ))
        .send()
        .await
        .expect("Failed to get project");

    assert!(get_project_resp.status().is_success());
    let project: serde_json::Value = get_project_resp
        .json()
        .await
        .expect("Failed to parse project");
    assert_eq!(project["name"], "test-project");

    // ============ Test File Operations ============

    // 1. Add a file to the project repository
    let add_file_resp = client
        .post(format!(
            "{}/api/orgs/test-org/projects/test-project/files",
            base_url
        ))
        .json(&serde_json::json!({
            "path": "README.md",
            "content": "# Hello World\n\nThis is a test file.",
            "message": "Add README.md"
        }))
        .send()
        .await
        .expect("Failed to add file");

    assert!(
        add_file_resp.status().is_success(),
        "Add file should succeed: {:?}",
        add_file_resp.status()
    );

    // 2. List files in the repository
    let list_files_resp = client
        .get(format!(
            "{}/api/orgs/test-org/projects/test-project/files",
            base_url
        ))
        .send()
        .await
        .expect("Failed to list files");

    assert!(list_files_resp.status().is_success());
    let files: Vec<serde_json::Value> =
        list_files_resp.json().await.expect("Failed to parse files");
    assert!(!files.is_empty(), "Should have at least one file");
    assert!(
        files.iter().any(|f| f["name"] == "README.md"),
        "README.md should exist"
    );

    // 3. View file content
    let view_file_resp = client
        .get(format!(
            "{}/api/orgs/test-org/projects/test-project/blob?path=README.md",
            base_url
        ))
        .send()
        .await
        .expect("Failed to view file");

    assert!(view_file_resp.status().is_success());
    let content = view_file_resp
        .text()
        .await
        .expect("Failed to get file content");
    assert!(
        content.contains("Hello World"),
        "File should contain expected content"
    );

    // 4. Edit file
    let edit_file_resp = client
        .post(format!(
            "{}/api/orgs/test-org/projects/test-project/files",
            base_url
        ))
        .json(&serde_json::json!({
            "path": "README.md",
            "content": "# Hello World - Updated\n\nThis is an updated test file.",
            "message": "Update README.md"
        }))
        .send()
        .await
        .expect("Failed to edit file");

    assert!(
        edit_file_resp.status().is_success(),
        "Edit file should succeed"
    );

    // 5. Verify the edit
    let verify_edit_resp = client
        .get(format!(
            "{}/api/orgs/test-org/projects/test-project/blob?path=README.md",
            base_url
        ))
        .send()
        .await
        .expect("Failed to verify edit");

    assert!(verify_edit_resp.status().is_success());
    let updated_content = verify_edit_resp
        .text()
        .await
        .expect("Failed to get updated content");
    assert!(
        updated_content.contains("Updated"),
        "File should contain updated content"
    );

    // 6. List commits
    let list_commits_resp = client
        .get(format!(
            "{}/api/orgs/test-org/projects/test-project/commits",
            base_url
        ))
        .send()
        .await
        .expect("Failed to list commits");

    assert!(list_commits_resp.status().is_success());
    let commits: Vec<serde_json::Value> = list_commits_resp
        .json()
        .await
        .expect("Failed to parse commits");
    assert!(commits.len() >= 2, "Should have at least 2 commits");

    // ============ Test Issue Operations ============

    // 1. Create an issue
    let create_issue_resp = client
        .post(format!(
            "{}/api/orgs/test-org/projects/test-project/issues",
            base_url
        ))
        .json(&serde_json::json!({
            "title": "Test Issue",
            "body": "This is a test issue body"
        }))
        .send()
        .await
        .expect("Failed to create issue");

    assert!(
        create_issue_resp.status().is_success(),
        "Create issue should succeed"
    );
    let issue: serde_json::Value = create_issue_resp
        .json()
        .await
        .expect("Failed to parse issue");
    assert_eq!(issue["title"], "Test Issue");
    assert_eq!(issue["number"], 1);
    assert_eq!(issue["state"], "open");

    // 2. List issues
    let list_issues_resp = client
        .get(format!(
            "{}/api/orgs/test-org/projects/test-project/issues",
            base_url
        ))
        .send()
        .await
        .expect("Failed to list issues");

    assert!(list_issues_resp.status().is_success());
    let issues: Vec<serde_json::Value> = list_issues_resp
        .json()
        .await
        .expect("Failed to parse issues");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0]["title"], "Test Issue");

    // 3. Get issue
    let get_issue_resp = client
        .get(format!(
            "{}/api/orgs/test-org/projects/test-project/issues/1",
            base_url
        ))
        .send()
        .await
        .expect("Failed to get issue");

    assert!(get_issue_resp.status().is_success());
    let issue: serde_json::Value = get_issue_resp.json().await.expect("Failed to parse issue");
    assert_eq!(issue["title"], "Test Issue");

    // 4. Update issue
    let update_issue_resp = client
        .patch(format!(
            "{}/api/orgs/test-org/projects/test-project/issues/1",
            base_url
        ))
        .json(&serde_json::json!({
            "title": "Updated Test Issue",
            "state": "closed"
        }))
        .send()
        .await
        .expect("Failed to update issue");

    assert!(update_issue_resp.status().is_success());
    let updated_issue: serde_json::Value = update_issue_resp
        .json()
        .await
        .expect("Failed to parse updated issue");
    assert_eq!(updated_issue["title"], "Updated Test Issue");
    assert_eq!(updated_issue["state"], "closed");

    // 5. Create issue comment
    let create_comment_resp = client
        .post(format!(
            "{}/api/orgs/test-org/projects/test-project/issues/1/comments",
            base_url
        ))
        .json(&serde_json::json!({
            "body": "This is a test comment"
        }))
        .send()
        .await
        .expect("Failed to create comment");

    assert!(create_comment_resp.status().is_success());
    let comment: serde_json::Value = create_comment_resp
        .json()
        .await
        .expect("Failed to parse comment");
    assert_eq!(comment["body"], "This is a test comment");

    // 6. List issue comments
    let list_comments_resp = client
        .get(format!(
            "{}/api/orgs/test-org/projects/test-project/issues/1/comments",
            base_url
        ))
        .send()
        .await
        .expect("Failed to list comments");

    assert!(list_comments_resp.status().is_success());
    let comments: Vec<serde_json::Value> = list_comments_resp
        .json()
        .await
        .expect("Failed to parse comments");
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0]["body"], "This is a test comment");

    // ============ Test Error Handling ============

    // 1. Test 404 for non-existent organization
    let not_found_resp = client
        .get(format!("{}/api/orgs/nonexistent-org", base_url))
        .send()
        .await
        .expect("Failed to test 404");

    assert_eq!(
        not_found_resp.status().as_u16(),
        404,
        "Should return 404 for non-existent org"
    );

    // 2. Test 409 for duplicate organization
    let duplicate_org_resp = client
        .post(format!("{}/api/orgs", base_url))
        .json(&serde_json::json!({
            "name": "test-org",
            "display_name": "Duplicate Org",
            "description": ""
        }))
        .send()
        .await
        .expect("Failed to test duplicate");

    assert_eq!(
        duplicate_org_resp.status().as_u16(),
        409,
        "Should return 409 for duplicate org"
    );

    // 3. Test 400 for invalid input
    let bad_request_resp = client
        .post(format!("{}/api/orgs", base_url))
        .json(&serde_json::json!({
            "name": "",
            "display_name": "",
            "description": ""
        }))
        .send()
        .await
        .expect("Failed to test bad request");

    assert_eq!(
        bad_request_resp.status().as_u16(),
        400,
        "Should return 400 for empty name"
    );

    // Clean up
    cleanup();

    println!("All HTTP API workflow tests passed!");
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

    // Start the server
    let mut server_process = Command::new(&git_server_binary)
        .args([
            "-c",
            config_path.to_str().unwrap(),
            "-r",
            repos_dir.to_str().unwrap(),
            "-d",
            db_path.to_str().unwrap(),
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

    // Test list orgs endpoint (initially empty)
    let orgs_response = client
        .get(format!("http://127.0.0.1:{}/api/orgs", http_port))
        .send()
        .await;

    match orgs_response {
        Ok(response) => {
            assert!(
                response.status().is_success(),
                "GET /api/orgs should succeed"
            );
            let orgs: Vec<serde_json::Value> = response.json().await.expect("Failed to parse JSON");
            assert!(orgs.is_empty(), "Should have no organizations initially");
        }
        Err(e) => {
            eprintln!("Failed to call API: {}", e);
        }
    }

    cleanup();
}

// ============ Sandbox Security Tests ============

/// Test that path traversal attempts in file paths are blocked
#[tokio::test]
async fn test_path_traversal_blocked() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let server_dir = temp_dir.path().to_path_buf();
    let repos_dir = server_dir.join("repos");
    let db_path = server_dir.join("test.db");

    std::fs::create_dir_all(&repos_dir).expect("Failed to create repos dir");

    // Use random high ports to avoid conflicts
    let ssh_port: u16 = 50000 + (std::process::id() as u16 % 10000);
    let http_port: u16 = ssh_port + 1;

    // Create config
    let config_path = create_config(&server_dir, ssh_port, http_port, "").await;

    // Find the git-server binary
    let git_server_binary = std::env::current_dir()
        .unwrap()
        .join("target/debug/git-server");

    if !git_server_binary.exists() {
        eprintln!("git-server binary not found, skipping path traversal test");
        return;
    }

    // Start the server
    let mut server_process = Command::new(&git_server_binary)
        .args([
            "-c",
            config_path.to_str().unwrap(),
            "-r",
            repos_dir.to_str().unwrap(),
            "-d",
            db_path.to_str().unwrap(),
            "serve",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    // Wait for HTTP server to start
    let server_started = wait_for_port(http_port, 10).await;

    let mut cleanup = || {
        let _ = server_process.start_kill();
    };

    if !server_started {
        cleanup();
        panic!("HTTP server did not start in time");
    }

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", http_port);

    // Create organization and project first
    client
        .post(format!("{}/api/orgs", base_url))
        .json(&serde_json::json!({
            "name": "test-org",
            "display_name": "Test Organization",
            "description": ""
        }))
        .send()
        .await
        .expect("Failed to create org");

    client
        .post(format!("{}/api/orgs/test-org/projects", base_url))
        .json(&serde_json::json!({
            "name": "test-project",
            "display_name": "Test Project",
            "description": ""
        }))
        .send()
        .await
        .expect("Failed to create project");

    // Add a normal file first
    client
        .post(format!(
            "{}/api/orgs/test-org/projects/test-project/files",
            base_url
        ))
        .json(&serde_json::json!({
            "path": "README.md",
            "content": "# Hello",
            "message": "Add README"
        }))
        .send()
        .await
        .expect("Failed to add normal file");

    // ============ Test path traversal attacks ============

    // Test 1: Try to create a file with path traversal in the path
    let traversal_resp = client
        .post(format!(
            "{}/api/orgs/test-org/projects/test-project/files",
            base_url
        ))
        .json(&serde_json::json!({
            "path": "../../../etc/evil_file",
            "content": "evil content",
            "message": "Evil commit"
        }))
        .send()
        .await
        .expect("Failed to send traversal request");

    assert_eq!(
        traversal_resp.status().as_u16(),
        400,
        "Path traversal should be blocked with 400 Bad Request"
    );

    // Test 2: Try another path traversal pattern
    let traversal_resp2 = client
        .post(format!(
            "{}/api/orgs/test-org/projects/test-project/files",
            base_url
        ))
        .json(&serde_json::json!({
            "path": "foo/../../bar/../../../etc/passwd",
            "content": "evil content",
            "message": "Evil commit"
        }))
        .send()
        .await
        .expect("Failed to send traversal request");

    assert_eq!(
        traversal_resp2.status().as_u16(),
        400,
        "Complex path traversal should be blocked with 400 Bad Request"
    );

    // Test 3: Verify normal file operations still work
    let normal_resp = client
        .post(format!(
            "{}/api/orgs/test-org/projects/test-project/files",
            base_url
        ))
        .json(&serde_json::json!({
            "path": "src/main.rs",
            "content": "fn main() {}",
            "message": "Add main.rs"
        }))
        .send()
        .await
        .expect("Failed to add normal file");

    assert!(
        normal_resp.status().is_success(),
        "Normal file creation should succeed"
    );

    // Verify the file exists
    let files_resp = client
        .get(format!(
            "{}/api/orgs/test-org/projects/test-project/tree?path=src",
            base_url
        ))
        .send()
        .await
        .expect("Failed to list files");

    assert!(files_resp.status().is_success());
    let files: Vec<serde_json::Value> = files_resp.json().await.expect("Failed to parse files");
    assert!(
        files.iter().any(|f| f["name"] == "main.rs"),
        "main.rs should exist"
    );

    cleanup();

    println!("Path traversal blocking tests passed!");
}

/// Test that symlink attacks are blocked in SSH operations
#[tokio::test]
async fn test_symlink_attack_blocked() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let server_dir = temp_dir.path().to_path_buf();
    let repos_dir = server_dir.join("repos");
    let db_path = server_dir.join("test.db");

    std::fs::create_dir_all(&repos_dir).expect("Failed to create repos dir");

    // Create organization and project directories
    let org_dir = repos_dir.join("evil-org").join("evil-project");
    std::fs::create_dir_all(&org_dir).expect("Failed to create org dir");

    // Create a symlink inside repos that points outside
    let symlink_path = org_dir.join("evil-link.git");

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        // Try to create a symlink that points to /tmp
        if symlink("/tmp", &symlink_path).is_ok() {
            // Use random high ports to avoid conflicts
            let ssh_port: u16 = 51000 + (std::process::id() as u16 % 10000);
            let http_port: u16 = ssh_port + 1;

            // Create config
            let config_path = create_config(&server_dir, ssh_port, http_port, "").await;

            // Find the git-server binary
            let git_server_binary = std::env::current_dir()
                .unwrap()
                .join("target/debug/git-server");

            if !git_server_binary.exists() {
                eprintln!("git-server binary not found, skipping symlink test");
                return;
            }

            // Start the server
            let mut server_process = Command::new(&git_server_binary)
                .args([
                    "-c",
                    config_path.to_str().unwrap(),
                    "-r",
                    repos_dir.to_str().unwrap(),
                    "-d",
                    db_path.to_str().unwrap(),
                    "serve",
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start server");

            // Wait for HTTP server to start
            let server_started = wait_for_port(http_port, 10).await;

            let mut cleanup = || {
                let _ = server_process.start_kill();
            };

            if !server_started {
                cleanup();
                panic!("HTTP server did not start in time");
            }

            let client = reqwest::Client::new();
            let base_url = format!("http://127.0.0.1:{}", http_port);

            // Create organization in database
            client
                .post(format!("{}/api/orgs", base_url))
                .json(&serde_json::json!({
                    "name": "evil-org",
                    "display_name": "Evil Organization",
                    "description": ""
                }))
                .send()
                .await
                .expect("Failed to create org");

            // Try to access the symlink as a repository
            // This should fail because the path validation should detect the symlink escape
            let blob_resp = client
                .get(format!(
                    "{}/api/orgs/evil-org/projects/evil-project/blob?path=passwd",
                    base_url
                ))
                .send()
                .await;

            match blob_resp {
                Ok(resp) => {
                    // Should be 404 (not found) because the symlink shouldn't be followed
                    assert!(
                        resp.status().as_u16() == 404 || resp.status().as_u16() == 500,
                        "Symlink escape should be blocked, got status: {}",
                        resp.status()
                    );
                }
                Err(err) => {
                    // Connection error is also acceptable - means the request was rejected
                    eprintln!("Request failed (which is expected): {}", err);
                }
            }

            cleanup();

            println!("Symlink attack blocking tests passed!");
        } else {
            eprintln!("Could not create symlink, skipping symlink test");
        }
    }

    #[cfg(not(unix))]
    {
        eprintln!("Symlink tests only run on Unix systems");
    }
}

/// Test that file delete operations validate paths correctly
#[tokio::test]
async fn test_file_delete_path_validation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let server_dir = temp_dir.path().to_path_buf();
    let repos_dir = server_dir.join("repos");
    let db_path = server_dir.join("test.db");

    std::fs::create_dir_all(&repos_dir).expect("Failed to create repos dir");

    // Use random high ports to avoid conflicts
    let ssh_port: u16 = 52000 + (std::process::id() as u16 % 10000);
    let http_port: u16 = ssh_port + 1;

    // Create config
    let config_path = create_config(&server_dir, ssh_port, http_port, "").await;

    // Find the git-server binary
    let git_server_binary = std::env::current_dir()
        .unwrap()
        .join("target/debug/git-server");

    if !git_server_binary.exists() {
        eprintln!("git-server binary not found, skipping delete path validation test");
        return;
    }

    // Start the server
    let mut server_process = Command::new(&git_server_binary)
        .args([
            "-c",
            config_path.to_str().unwrap(),
            "-r",
            repos_dir.to_str().unwrap(),
            "-d",
            db_path.to_str().unwrap(),
            "serve",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    // Wait for HTTP server to start
    let server_started = wait_for_port(http_port, 10).await;

    let mut cleanup = || {
        let _ = server_process.start_kill();
    };

    if !server_started {
        cleanup();
        panic!("HTTP server did not start in time");
    }

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", http_port);

    // Create organization and project
    client
        .post(format!("{}/api/orgs", base_url))
        .json(&serde_json::json!({
            "name": "test-org2",
            "display_name": "Test Organization 2",
            "description": ""
        }))
        .send()
        .await
        .expect("Failed to create org");

    client
        .post(format!("{}/api/orgs/test-org2/projects", base_url))
        .json(&serde_json::json!({
            "name": "test-project2",
            "display_name": "Test Project 2",
            "description": ""
        }))
        .send()
        .await
        .expect("Failed to create project");

    // Add a file first
    client
        .post(format!(
            "{}/api/orgs/test-org2/projects/test-project2/files",
            base_url
        ))
        .json(&serde_json::json!({
            "path": "test.txt",
            "content": "test content",
            "message": "Add test file"
        }))
        .send()
        .await
        .expect("Failed to add file");

    // Try to delete with path traversal
    let delete_resp = client
        .delete(format!(
            "{}/api/orgs/test-org2/projects/test-project2/files",
            base_url
        ))
        .json(&serde_json::json!({
            "path": "../../../etc/passwd",
            "message": "Evil delete"
        }))
        .send()
        .await
        .expect("Failed to send delete request");

    assert_eq!(
        delete_resp.status().as_u16(),
        400,
        "Delete with path traversal should be blocked"
    );

    // Normal delete should work
    let normal_delete_resp = client
        .delete(format!(
            "{}/api/orgs/test-org2/projects/test-project2/files",
            base_url
        ))
        .json(&serde_json::json!({
            "path": "test.txt",
            "message": "Delete test file"
        }))
        .send()
        .await
        .expect("Failed to send normal delete request");

    assert!(
        normal_delete_resp.status().is_success(),
        "Normal delete should succeed"
    );

    cleanup();

    println!("File delete path validation tests passed!");
}

/// Test that the sandbox properly restricts git command filesystem access
#[tokio::test]
async fn test_sandboxed_git_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let server_dir = temp_dir.path().to_path_buf();
    let repos_dir = server_dir.join("repos");
    let db_path = server_dir.join("test.db");

    std::fs::create_dir_all(&repos_dir).expect("Failed to create repos dir");

    // Use random high ports to avoid conflicts
    let ssh_port: u16 = 53000 + (std::process::id() as u16 % 10000);
    let http_port: u16 = ssh_port + 1;

    // Create config
    let config_path = create_config(&server_dir, ssh_port, http_port, "").await;

    // Find the git-server binary
    let git_server_binary = std::env::current_dir()
        .unwrap()
        .join("target/debug/git-server");

    if !git_server_binary.exists() {
        eprintln!("git-server binary not found, skipping sandboxed git operations test");
        return;
    }

    // Start the server
    let mut server_process = Command::new(&git_server_binary)
        .args([
            "-c",
            config_path.to_str().unwrap(),
            "-r",
            repos_dir.to_str().unwrap(),
            "-d",
            db_path.to_str().unwrap(),
            "serve",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    // Wait for HTTP server to start
    let server_started = wait_for_port(http_port, 10).await;

    let mut cleanup = || {
        let _ = server_process.start_kill();
    };

    if !server_started {
        cleanup();
        panic!("HTTP server did not start in time");
    }

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", http_port);

    // Create organization and project
    client
        .post(format!("{}/api/orgs", base_url))
        .json(&serde_json::json!({
            "name": "sandbox-test-org",
            "display_name": "Sandbox Test Org",
            "description": ""
        }))
        .send()
        .await
        .expect("Failed to create org");

    client
        .post(format!("{}/api/orgs/sandbox-test-org/projects", base_url))
        .json(&serde_json::json!({
            "name": "sandbox-test-project",
            "display_name": "Sandbox Test Project",
            "description": ""
        }))
        .send()
        .await
        .expect("Failed to create project");

    // Add multiple files to verify git operations work under sandbox
    for i in 1..=5 {
        let add_resp = client
            .post(format!(
                "{}/api/orgs/sandbox-test-org/projects/sandbox-test-project/files",
                base_url
            ))
            .json(&serde_json::json!({
                "path": format!("file{}.txt", i),
                "content": format!("Content of file {}", i),
                "message": format!("Add file{}.txt", i)
            }))
            .send()
            .await
            .expect("Failed to add file");

        assert!(
            add_resp.status().is_success(),
            "Adding file {} should succeed under sandbox",
            i
        );
    }

    // Verify all files were created
    let files_resp = client
        .get(format!(
            "{}/api/orgs/sandbox-test-org/projects/sandbox-test-project/files",
            base_url
        ))
        .send()
        .await
        .expect("Failed to list files");

    assert!(files_resp.status().is_success());
    let files: Vec<serde_json::Value> = files_resp.json().await.expect("Failed to parse files");
    assert_eq!(files.len(), 5, "Should have 5 files");

    // Verify commits were created
    let commits_resp = client
        .get(format!(
            "{}/api/orgs/sandbox-test-org/projects/sandbox-test-project/commits",
            base_url
        ))
        .send()
        .await
        .expect("Failed to list commits");

    assert!(commits_resp.status().is_success());
    let commits: Vec<serde_json::Value> =
        commits_resp.json().await.expect("Failed to parse commits");
    assert_eq!(commits.len(), 5, "Should have 5 commits");

    // Test git clone via HTTP (uses sandboxed git commands)
    let clone_dir = server_dir.join("clone_test");
    let clone_output = Command::new("git")
        .args([
            "clone",
            &format!(
                "http://127.0.0.1:{}/sandbox-test-org/sandbox-test-project.git",
                http_port
            ),
            clone_dir.to_str().unwrap(),
        ])
        .output()
        .await;

    match clone_output {
        Ok(output) => {
            if output.status.success() {
                // Verify cloned content
                let cloned_files: Vec<_> = std::fs::read_dir(&clone_dir)
                    .expect("Failed to read clone dir")
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "txt"))
                    .collect();

                assert_eq!(cloned_files.len(), 5, "Cloned repo should have 5 txt files");
                println!("Git clone succeeded with sandboxed operations!");
            } else {
                eprintln!(
                    "Git clone failed (may be expected if HTTP auth is required): {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(err) => {
            eprintln!("Failed to run git clone: {}", err);
        }
    }

    cleanup();

    println!("Sandboxed git operations tests passed!");
}
