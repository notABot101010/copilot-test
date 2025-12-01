//! End-to-end tests that simulate a complete conversation flow
//! 
//! These tests verify the full workflow from sending a message to receiving
//! a response from the AI agent.

use std::sync::Arc;
use tempfile::TempDir;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

/// Helper to create a test server with a real mock OpenAI server
async fn start_test_server_with_mock_openai() -> (String, TempDir, MockServer) {
    // Start mock OpenAI server
    let mock_server = MockServer::start().await;
    
    // Mock the chat completion endpoint
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl-test",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "TASK:code_editor:high:Implement the requested feature\nTASK:test_runner:medium:Add tests for the feature"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50,
                "total_tokens": 150
            }
        })))
        .mount(&mock_server)
        .await;
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    
    // Find an available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let port = listener.local_addr().expect("Failed to get address").port();
    drop(listener);
    
    let db_path_str = db_path.to_str().unwrap().to_string();
    let mock_url = mock_server.uri();
    let port_clone = port;
    
    // Start server in background
    tokio::spawn(async move {
        use axum::{Router, routing::{get, post}};
        use tower_http::{cors::CorsLayer, trace::TraceLayer};
        
        // Initialize database
        let db = ai_coding_agent_server::db::init_db(&db_path_str)
            .await
            .expect("Failed to init db");
        
        // Create LLM client pointing to mock server
        let llm_config = ai_coding_agent_server::llm::LlmConfig::new()
            .with_api_base(&mock_url)
            .with_api_key("test-key")
            .with_model("gpt-4");
        
        let llm_client: Arc<dyn ai_coding_agent_server::llm::LlmClient> = 
            Arc::new(ai_coding_agent_server::llm::OpenAiLlmClient::new(llm_config));
        
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
    
    // Wait for server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let base_url = format!("http://127.0.0.1:{}", port);
    (base_url, temp_dir, mock_server)
}

#[tokio::test]
async fn test_e2e_conversation_flow() {
    let (base_url, _temp_dir, _mock_server) = start_test_server_with_mock_openai().await;
    
    let client = reqwest::Client::new();
    
    // Step 1: Create a session
    let create_response = client
        .post(format!("{}/api/sessions", base_url))
        .json(&serde_json::json!({ "name": "E2E Test Session" }))
        .send()
        .await
        .expect("Request failed");
    
    assert_eq!(create_response.status(), reqwest::StatusCode::OK);
    
    let created: serde_json::Value = create_response.json().await.expect("Failed to parse");
    let session_id = created.get("id").and_then(|v| v.as_str()).expect("No id");
    
    // Step 2: Send a message to implement a feature
    let send_response = client
        .post(format!("{}/api/sessions/{}/messages", base_url, session_id))
        .json(&serde_json::json!({ "content": "Implement a user authentication feature" }))
        .send()
        .await
        .expect("Request failed");
    
    assert_eq!(send_response.status(), reqwest::StatusCode::OK);
    
    let message: serde_json::Value = send_response.json().await.expect("Failed to parse");
    assert_eq!(message.get("role").and_then(|v| v.as_str()), Some("user"));
    
    // Step 3: Wait for processing and get messages
    // Note: The orchestrator processes asynchronously, so we need to wait
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    let messages_response = client
        .get(format!("{}/api/sessions/{}/messages", base_url, session_id))
        .send()
        .await
        .expect("Request failed");
    
    assert_eq!(messages_response.status(), reqwest::StatusCode::OK);
    
    let messages: Vec<serde_json::Value> = messages_response.json().await.expect("Failed to parse");
    
    // Should have at least the user message
    assert!(!messages.is_empty());
    assert!(messages.iter().any(|m| m.get("role").and_then(|v| v.as_str()) == Some("user")));
}

#[tokio::test]
async fn test_e2e_multi_turn_conversation() {
    let (base_url, _temp_dir, _mock_server) = start_test_server_with_mock_openai().await;
    
    let client = reqwest::Client::new();
    
    // Create a session
    let create_response = client
        .post(format!("{}/api/sessions", base_url))
        .json(&serde_json::json!({ "name": "Multi-turn Test" }))
        .send()
        .await
        .expect("Request failed");
    
    let created: serde_json::Value = create_response.json().await.expect("Failed to parse");
    let session_id = created.get("id").and_then(|v| v.as_str()).expect("No id");
    
    // Send first message
    client
        .post(format!("{}/api/sessions/{}/messages", base_url, session_id))
        .json(&serde_json::json!({ "content": "Create a new REST API endpoint" }))
        .send()
        .await
        .expect("Request failed");
    
    // Send second message
    client
        .post(format!("{}/api/sessions/{}/messages", base_url, session_id))
        .json(&serde_json::json!({ "content": "Add tests for the endpoint" }))
        .send()
        .await
        .expect("Request failed");
    
    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Get all messages
    let messages_response = client
        .get(format!("{}/api/sessions/{}/messages", base_url, session_id))
        .send()
        .await
        .expect("Request failed");
    
    let messages: Vec<serde_json::Value> = messages_response.json().await.expect("Failed to parse");
    
    // Should have at least 2 user messages
    let user_messages: Vec<_> = messages
        .iter()
        .filter(|m| m.get("role").and_then(|v| v.as_str()) == Some("user"))
        .collect();
    assert!(user_messages.len() >= 2);
}

#[tokio::test]
async fn test_e2e_session_isolation() {
    let (base_url, _temp_dir, _mock_server) = start_test_server_with_mock_openai().await;
    
    let client = reqwest::Client::new();
    
    // Create first session
    let create_response1 = client
        .post(format!("{}/api/sessions", base_url))
        .json(&serde_json::json!({ "name": "Session 1" }))
        .send()
        .await
        .expect("Request failed");
    
    let created1: serde_json::Value = create_response1.json().await.expect("Failed to parse");
    let session_id1 = created1.get("id").and_then(|v| v.as_str()).expect("No id").to_string();
    
    // Create second session
    let create_response2 = client
        .post(format!("{}/api/sessions", base_url))
        .json(&serde_json::json!({ "name": "Session 2" }))
        .send()
        .await
        .expect("Request failed");
    
    let created2: serde_json::Value = create_response2.json().await.expect("Failed to parse");
    let session_id2 = created2.get("id").and_then(|v| v.as_str()).expect("No id").to_string();
    
    // Send message to first session
    client
        .post(format!("{}/api/sessions/{}/messages", base_url, session_id1))
        .json(&serde_json::json!({ "content": "Message for session 1" }))
        .send()
        .await
        .expect("Request failed");
    
    // Send message to second session
    client
        .post(format!("{}/api/sessions/{}/messages", base_url, session_id2))
        .json(&serde_json::json!({ "content": "Message for session 2" }))
        .send()
        .await
        .expect("Request failed");
    
    // Get messages from first session
    let messages_response1 = client
        .get(format!("{}/api/sessions/{}/messages", base_url, session_id1))
        .send()
        .await
        .expect("Request failed");
    
    let messages1: Vec<serde_json::Value> = messages_response1.json().await.expect("Failed to parse");
    
    // Get messages from second session
    let messages_response2 = client
        .get(format!("{}/api/sessions/{}/messages", base_url, session_id2))
        .send()
        .await
        .expect("Request failed");
    
    let messages2: Vec<serde_json::Value> = messages_response2.json().await.expect("Failed to parse");
    
    // Verify sessions are isolated - messages should be different
    let content1 = messages1.first()
        .and_then(|m| m.get("content"))
        .and_then(|v| v.as_str());
    let content2 = messages2.first()
        .and_then(|m| m.get("content"))
        .and_then(|v| v.as_str());
    
    assert!(content1.is_some());
    assert!(content2.is_some());
    assert_ne!(content1, content2);
}
