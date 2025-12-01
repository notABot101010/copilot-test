use reqwest::Client;
use serde_json::json;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

struct ServerProcess {
    process: Child,
}

impl ServerProcess {
    fn new() -> Self {
        // Clean up old test database
        let _ = std::fs::remove_file("test_integration.db");
        
        let process = Command::new("cargo")
            .args(["run"])
            .env("DATABASE_URL", "sqlite:test_integration.db?mode=rwc")
            .env("PORT", "4002")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start server");
        
        // Wait for server to start
        std::thread::sleep(Duration::from_secs(5));
        
        ServerProcess { process }
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = std::fs::remove_file("test_integration.db");
    }
}

#[tokio::test]
async fn test_user_registration() {
    let _server = ServerProcess::new();
    let client = Client::new();
    
    let response = client
        .post("http://localhost:4002/api/register")
        .json(&json!({
            "username": "testuser1",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 201);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["username"], "testuser1");
}

#[tokio::test]
async fn test_user_login() {
    let _server = ServerProcess::new();
    let client = Client::new();
    
    // Register first
    client
        .post("http://localhost:4002/api/register")
        .json(&json!({
            "username": "loginuser",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register");
    
    // Login
    let response = client
        .post("http://localhost:4002/api/login")
        .json(&json!({
            "username": "loginuser",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to login");
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["username"], "loginuser");
    assert!(body["token"].as_str().is_some());
}

#[tokio::test]
async fn test_invalid_login() {
    let _server = ServerProcess::new();
    let client = Client::new();
    
    let response = client
        .post("http://localhost:4002/api/login")
        .json(&json!({
            "username": "nonexistent",
            "password": "wrongpassword"
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_create_match() {
    let _server = ServerProcess::new();
    let client = Client::new();
    
    // Register two users
    client
        .post("http://localhost:4002/api/register")
        .json(&json!({
            "username": "white_player",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register user 1");
    
    client
        .post("http://localhost:4002/api/register")
        .json(&json!({
            "username": "black_player",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register user 2");
    
    // Login as white_player
    let login_response = client
        .post("http://localhost:4002/api/login")
        .json(&json!({
            "username": "white_player",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to login");
    
    let login_body: serde_json::Value = login_response.json().await.expect("Failed to parse login");
    let token = login_body["token"].as_str().expect("No token");
    
    // Create match
    let response = client
        .post("http://localhost:4002/api/matches")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "opponent_username": "black_player"
        }))
        .send()
        .await
        .expect("Failed to create match");
    
    assert_eq!(response.status(), 201);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["white_player"], "white_player");
    assert_eq!(body["black_player"], "black_player");
    assert_eq!(body["status"], "active");
}

#[tokio::test]
async fn test_list_matches() {
    let _server = ServerProcess::new();
    let client = Client::new();
    
    // Register two users
    client
        .post("http://localhost:4002/api/register")
        .json(&json!({
            "username": "list_user1",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register");
    
    client
        .post("http://localhost:4002/api/register")
        .json(&json!({
            "username": "list_user2",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register");
    
    // Login
    let login_response = client
        .post("http://localhost:4002/api/login")
        .json(&json!({
            "username": "list_user1",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to login");
    
    let login_body: serde_json::Value = login_response.json().await.expect("Failed to parse login");
    let token = login_body["token"].as_str().expect("No token");
    
    // Create a match
    client
        .post("http://localhost:4002/api/matches")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "opponent_username": "list_user2"
        }))
        .send()
        .await
        .expect("Failed to create match");
    
    // List matches
    let response = client
        .get("http://localhost:4002/api/matches")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to list matches");
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(body["matches"].as_array().is_some());
    assert!(!body["matches"].as_array().expect("Not array").is_empty());
}

#[tokio::test]
async fn test_make_move() {
    let _server = ServerProcess::new();
    let client = Client::new();
    
    // Register two users
    client
        .post("http://localhost:4002/api/register")
        .json(&json!({
            "username": "move_white",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register");
    
    client
        .post("http://localhost:4002/api/register")
        .json(&json!({
            "username": "move_black",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register");
    
    // Login as white
    let login_response = client
        .post("http://localhost:4002/api/login")
        .json(&json!({
            "username": "move_white",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to login");
    
    let login_body: serde_json::Value = login_response.json().await.expect("Failed to parse login");
    let token = login_body["token"].as_str().expect("No token");
    
    // Create match
    let match_response = client
        .post("http://localhost:4002/api/matches")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "opponent_username": "move_black"
        }))
        .send()
        .await
        .expect("Failed to create match");
    
    let match_body: serde_json::Value = match_response.json().await.expect("Failed to parse match");
    let match_id = match_body["id"].as_str().expect("No match id");
    
    // Make a move (e2 to e4)
    let move_response = client
        .post(&format!("http://localhost:4002/api/matches/{}/move", match_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "from": "e2",
            "to": "e4"
        }))
        .send()
        .await
        .expect("Failed to make move");
    
    assert_eq!(move_response.status(), 200);
    
    let move_body: serde_json::Value = move_response.json().await.expect("Failed to parse response");
    assert_eq!(move_body["success"], true);
}

#[tokio::test]
async fn test_invalid_move() {
    let _server = ServerProcess::new();
    let client = Client::new();
    
    // Register two users
    client
        .post("http://localhost:4002/api/register")
        .json(&json!({
            "username": "invalid_white",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register");
    
    client
        .post("http://localhost:4002/api/register")
        .json(&json!({
            "username": "invalid_black",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register");
    
    // Login as white
    let login_response = client
        .post("http://localhost:4002/api/login")
        .json(&json!({
            "username": "invalid_white",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to login");
    
    let login_body: serde_json::Value = login_response.json().await.expect("Failed to parse login");
    let token = login_body["token"].as_str().expect("No token");
    
    // Create match
    let match_response = client
        .post("http://localhost:4002/api/matches")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "opponent_username": "invalid_black"
        }))
        .send()
        .await
        .expect("Failed to create match");
    
    let match_body: serde_json::Value = match_response.json().await.expect("Failed to parse match");
    let match_id = match_body["id"].as_str().expect("No match id");
    
    // Try an invalid move (e2 to e5 - too far for pawn opening)
    let move_response = client
        .post(&format!("http://localhost:4002/api/matches/{}/move", match_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "from": "e2",
            "to": "e5"
        }))
        .send()
        .await
        .expect("Failed to make move");
    
    assert_eq!(move_response.status(), 200);
    
    let move_body: serde_json::Value = move_response.json().await.expect("Failed to parse response");
    assert_eq!(move_body["success"], false);
}

#[tokio::test]
async fn test_get_match_details() {
    let _server = ServerProcess::new();
    let client = Client::new();
    
    // Register two users
    client
        .post("http://localhost:4002/api/register")
        .json(&json!({
            "username": "detail_white",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register");
    
    client
        .post("http://localhost:4002/api/register")
        .json(&json!({
            "username": "detail_black",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register");
    
    // Login
    let login_response = client
        .post("http://localhost:4002/api/login")
        .json(&json!({
            "username": "detail_white",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to login");
    
    let login_body: serde_json::Value = login_response.json().await.expect("Failed to parse login");
    let token = login_body["token"].as_str().expect("No token");
    
    // Create match
    let match_response = client
        .post("http://localhost:4002/api/matches")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "opponent_username": "detail_black"
        }))
        .send()
        .await
        .expect("Failed to create match");
    
    let match_body: serde_json::Value = match_response.json().await.expect("Failed to parse match");
    let match_id = match_body["id"].as_str().expect("No match id");
    
    // Get match details
    let detail_response = client
        .get(&format!("http://localhost:4002/api/matches/{}", match_id))
        .send()
        .await
        .expect("Failed to get match");
    
    assert_eq!(detail_response.status(), 200);
    
    let detail_body: serde_json::Value = detail_response.json().await.expect("Failed to parse response");
    assert_eq!(detail_body["white_player"], "detail_white");
    assert_eq!(detail_body["black_player"], "detail_black");
    assert!(detail_body["document"].as_str().is_some());
}
