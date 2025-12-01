//! Integration tests for the AI Coding Agent server
//! 
//! These tests verify the HTTP API works correctly with mock LLM responses.

use std::sync::Arc;
use std::time::Duration;

use reqwest::StatusCode;
use tempfile::TempDir;

/// Wait for server to be ready by polling the health endpoint
async fn wait_for_server(base_url: &str, max_attempts: u32) -> bool {
    let client = reqwest::Client::new();
    for _ in 0..max_attempts {
        if let Ok(response) = client
            .get(format!("{}/api/sessions", base_url))
            .timeout(Duration::from_millis(100))
            .send()
            .await
        {
            if response.status().is_success() {
                return true;
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

/// Helper to create a test server with mock LLM
async fn start_test_server() -> (String, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    
    // Find an available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let port = listener.local_addr().expect("Failed to get address").port();
    drop(listener);
    
    let db_path_str = db_path.to_str().unwrap().to_string();
    let port_clone = port;
    
    // Start server in background
    tokio::spawn(async move {
        use axum::{Router, routing::{get, post}};
        use tower_http::{cors::CorsLayer, trace::TraceLayer};
        
        // Initialize database
        let db = ai_coding_agent_server::db::init_db(&db_path_str)
            .await
            .expect("Failed to init db");
        
        // Create mock LLM client
        let llm_client: Arc<dyn ai_coding_agent_server::llm::LlmClient> = 
            Arc::new(ai_coding_agent_server::llm::MockLlmClient::with_responses(vec![
                "TASK:code_editor:high:Implement test feature".to_string(),
                "Test implementation response".to_string(),
            ]));
        
        let templates = Arc::new(tokio::sync::RwLock::new(
            ai_coding_agent_server::templates::TemplateManager::new()
        ));
        let orchestrator = Arc::new(
            ai_coding_agent_server::orchestrator::Orchestrator::new(llm_client.clone())
        );
        
        let state = Arc::new(ai_coding_agent_server::AppState {
            db,
            templates,
            orchestrator,
            llm_client,
        });
        
        let app = Router::new()
            .route("/api/sessions", post(ai_coding_agent_server::handlers::sessions::create_session))
            .route("/api/sessions", get(ai_coding_agent_server::handlers::sessions::list_sessions))
            .route("/api/sessions/:id", get(ai_coding_agent_server::handlers::sessions::get_session))
            .route("/api/sessions/:id/messages", post(ai_coding_agent_server::handlers::sessions::send_message))
            .route("/api/sessions/:id/messages", get(ai_coding_agent_server::handlers::sessions::get_messages))
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http())
            .with_state(state);
        
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port_clone))
            .await
            .expect("Failed to bind");
        axum::serve(listener, app).await.expect("Server error");
    });
    
    let base_url = format!("http://127.0.0.1:{}", port);
    
    // Wait for server to be ready with retry logic
    assert!(wait_for_server(&base_url, 20).await, "Server failed to start");
    
    (base_url, temp_dir)
}

#[tokio::test]
async fn test_create_session() {
    let (base_url, _temp_dir) = start_test_server().await;
    
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/sessions", base_url))
        .json(&serde_json::json!({ "name": "Test Session" }))
        .send()
        .await
        .expect("Request failed");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(body.get("id").is_some());
    assert_eq!(body.get("name").and_then(|v| v.as_str()), Some("Test Session"));
    assert_eq!(body.get("status").and_then(|v| v.as_str()), Some("active"));
}

#[tokio::test]
async fn test_list_sessions() {
    let (base_url, _temp_dir) = start_test_server().await;
    
    let client = reqwest::Client::new();
    
    // Create a session first
    client
        .post(format!("{}/api/sessions", base_url))
        .json(&serde_json::json!({ "name": "Test Session" }))
        .send()
        .await
        .expect("Request failed");
    
    // List sessions
    let response = client
        .get(format!("{}/api/sessions", base_url))
        .send()
        .await
        .expect("Request failed");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Vec<serde_json::Value> = response.json().await.expect("Failed to parse response");
    assert!(!body.is_empty());
}

#[tokio::test]
async fn test_get_session() {
    let (base_url, _temp_dir) = start_test_server().await;
    
    let client = reqwest::Client::new();
    
    // Create a session
    let create_response = client
        .post(format!("{}/api/sessions", base_url))
        .json(&serde_json::json!({ "name": "Test Session" }))
        .send()
        .await
        .expect("Request failed");
    
    let created: serde_json::Value = create_response.json().await.expect("Failed to parse");
    let session_id = created.get("id").and_then(|v| v.as_str()).expect("No id");
    
    // Get session
    let response = client
        .get(format!("{}/api/sessions/{}", base_url, session_id))
        .send()
        .await
        .expect("Request failed");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse");
    assert_eq!(body.get("id").and_then(|v| v.as_str()), Some(session_id));
}

#[tokio::test]
async fn test_send_message() {
    let (base_url, _temp_dir) = start_test_server().await;
    
    let client = reqwest::Client::new();
    
    // Create a session
    let create_response = client
        .post(format!("{}/api/sessions", base_url))
        .json(&serde_json::json!({ "name": "Test Session" }))
        .send()
        .await
        .expect("Request failed");
    
    let created: serde_json::Value = create_response.json().await.expect("Failed to parse");
    let session_id = created.get("id").and_then(|v| v.as_str()).expect("No id");
    
    // Send a message
    let response = client
        .post(format!("{}/api/sessions/{}/messages", base_url, session_id))
        .json(&serde_json::json!({ "content": "Implement a new feature" }))
        .send()
        .await
        .expect("Request failed");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse");
    assert!(body.get("id").is_some());
    assert_eq!(body.get("role").and_then(|v| v.as_str()), Some("user"));
    assert_eq!(body.get("content").and_then(|v| v.as_str()), Some("Implement a new feature"));
}

#[tokio::test]
async fn test_get_messages() {
    let (base_url, _temp_dir) = start_test_server().await;
    
    let client = reqwest::Client::new();
    
    // Create a session
    let create_response = client
        .post(format!("{}/api/sessions", base_url))
        .json(&serde_json::json!({ "name": "Test Session" }))
        .send()
        .await
        .expect("Request failed");
    
    let created: serde_json::Value = create_response.json().await.expect("Failed to parse");
    let session_id = created.get("id").and_then(|v| v.as_str()).expect("No id");
    
    // Send a message
    client
        .post(format!("{}/api/sessions/{}/messages", base_url, session_id))
        .json(&serde_json::json!({ "content": "Hello" }))
        .send()
        .await
        .expect("Request failed");
    
    // Get messages
    let response = client
        .get(format!("{}/api/sessions/{}/messages", base_url, session_id))
        .send()
        .await
        .expect("Request failed");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Vec<serde_json::Value> = response.json().await.expect("Failed to parse");
    assert!(!body.is_empty());
}

#[tokio::test]
async fn test_session_not_found() {
    let (base_url, _temp_dir) = start_test_server().await;
    
    let client = reqwest::Client::new();
    
    let response = client
        .get(format!("{}/api/sessions/nonexistent-id", base_url))
        .send()
        .await
        .expect("Request failed");
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
