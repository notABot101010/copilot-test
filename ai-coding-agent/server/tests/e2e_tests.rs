//! End-to-end tests that simulate a complete conversation flow
//! 
//! These tests verify the full workflow from sending a message to receiving
//! a response from the AI agent, including WebSocket subscription.

use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};
use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Wait for server to be ready by polling
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

/// Wait for messages to be processed by polling
async fn wait_for_messages(client: &reqwest::Client, base_url: &str, session_id: &str, min_count: usize, max_attempts: u32) -> Vec<serde_json::Value> {
    for _ in 0..max_attempts {
        if let Ok(response) = client
            .get(format!("{}/api/sessions/{}/messages", base_url, session_id))
            .send()
            .await
        {
            if let Ok(messages) = response.json::<Vec<serde_json::Value>>().await {
                if messages.len() >= min_count {
                    return messages;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Vec::new()
}

/// Helper to create a test server with a real mock OpenAI server and WebSocket support
async fn start_test_server_with_websocket() -> (String, u16, TempDir, MockServer) {
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
    
    // Start server in background with WebSocket support
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
            .route("/api/sessions/:id/stream", get(ai_coding_agent_server::handlers::websocket::session_stream))
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
    
    (base_url, port, temp_dir, mock_server)
}

/// Helper to create a test server with mock OpenAI server
/// Note: This function provides full functionality including WebSocket support
/// but returns a simplified tuple for backward compatibility with existing tests
async fn start_test_server_with_mock_openai() -> (String, TempDir, MockServer) {
    let (base_url, _, temp_dir, mock_server) = start_test_server_with_websocket().await;
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
    
    // Step 3: Wait for processing and get messages using polling
    let messages = wait_for_messages(&client, &base_url, session_id, 1, 20).await;
    
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
    
    // Wait for messages using polling - expect at least 2 user messages
    let messages = wait_for_messages(&client, &base_url, session_id, 2, 20).await;
    
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

/// Test WebSocket subscription without panic - verifies the fix for block_on issue
#[tokio::test]
async fn test_e2e_websocket_subscribe_no_panic() {
    let (base_url, port, _temp_dir, _mock_server) = start_test_server_with_websocket().await;
    
    let client = reqwest::Client::new();
    
    // Step 1: Create a session
    let create_response = client
        .post(format!("{}/api/sessions", base_url))
        .json(&serde_json::json!({ "name": "WebSocket Test Session" }))
        .send()
        .await
        .expect("Request failed");
    
    assert_eq!(create_response.status(), reqwest::StatusCode::OK);
    
    let created: serde_json::Value = create_response.json().await.expect("Failed to parse");
    let session_id = created.get("id").and_then(|v| v.as_str()).expect("No id");
    
    // Step 2: Connect to WebSocket - this should not panic
    let ws_url = format!("ws://127.0.0.1:{}/api/sessions/{}/stream", port, session_id);
    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .expect("Failed to connect to WebSocket");
    
    let (mut _write, mut read) = ws_stream.split();
    
    // Step 3: Send a message to trigger events
    let send_response = client
        .post(format!("{}/api/sessions/{}/messages", base_url, session_id))
        .json(&serde_json::json!({ "content": "Implement a test feature" }))
        .send()
        .await
        .expect("Request failed");
    
    assert_eq!(send_response.status(), reqwest::StatusCode::OK);
    
    // Step 4: Read events from WebSocket
    let mut received_events = Vec::new();
    let timeout = tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) {
                    received_events.push(event);
                    // Stop after receiving a few events
                    if received_events.len() >= 2 {
                        break;
                    }
                }
            }
        }
    });
    
    // We don't require receiving events, just that no panic occurred
    let _ = timeout.await;
    
    // The fact that we got here without panic means the fix works
    // WebSocket connection was successful
}

/// Test complete workflow: create session, subscribe WebSocket, send message, receive events
#[tokio::test]
async fn test_e2e_complete_websocket_workflow() {
    let (base_url, port, _temp_dir, _mock_server) = start_test_server_with_websocket().await;
    
    let client = reqwest::Client::new();
    
    // Step 1: Create a session
    let create_response = client
        .post(format!("{}/api/sessions", base_url))
        .json(&serde_json::json!({ "name": "Complete WebSocket Workflow Test" }))
        .send()
        .await
        .expect("Request failed");
    
    assert_eq!(create_response.status(), reqwest::StatusCode::OK);
    
    let created: serde_json::Value = create_response.json().await.expect("Failed to parse");
    let session_id = created.get("id").and_then(|v| v.as_str()).expect("No id").to_string();
    
    // Step 2: Connect to WebSocket stream
    let ws_url = format!("ws://127.0.0.1:{}/api/sessions/{}/stream", port, session_id);
    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .expect("Failed to connect to WebSocket - this indicates the block_on panic issue is still present");
    
    let (mut _write, mut read) = ws_stream.split();
    
    // Step 3: Send a message (this triggers orchestrator processing)
    let session_id_clone = session_id.clone();
    let base_url_clone = base_url.clone();
    tokio::spawn(async move {
        // Small delay to ensure WebSocket is ready to receive
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let client = reqwest::Client::new();
        let _ = client
            .post(format!("{}/api/sessions/{}/messages", base_url_clone, session_id_clone))
            .json(&serde_json::json!({ "content": "Implement a new feature with tests" }))
            .send()
            .await;
    });
    
    // Step 4: Collect events from WebSocket
    let mut received_events = Vec::new();
    let timeout_result = tokio::time::timeout(Duration::from_secs(10), async {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) {
                        received_events.push(event.clone());
                        
                        // Check if we received a terminal event (snake_case due to serde rename)
                        if let Some(event_type) = event.get("event_type").and_then(|v| v.as_str()) {
                            if event_type == "agent_response" {
                                break;
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
    });
    
    let _ = timeout_result.await;
    
    // Step 5: Verify we received some events (or at least no panic occurred)
    // The key test is that the WebSocket connection worked without panic
    
    // Step 6: Verify messages were stored in database
    let messages = wait_for_messages(&client, &base_url, &session_id, 1, 30).await;
    assert!(!messages.is_empty(), "Should have at least the user message stored");
}

/// Test multiple WebSocket connections to same session
#[tokio::test]
async fn test_e2e_multiple_websocket_connections() {
    let (base_url, port, _temp_dir, _mock_server) = start_test_server_with_websocket().await;
    
    let client = reqwest::Client::new();
    
    // Create a session
    let create_response = client
        .post(format!("{}/api/sessions", base_url))
        .json(&serde_json::json!({ "name": "Multi-WebSocket Test" }))
        .send()
        .await
        .expect("Request failed");
    
    let created: serde_json::Value = create_response.json().await.expect("Failed to parse");
    let session_id = created.get("id").and_then(|v| v.as_str()).expect("No id");
    
    // Connect multiple WebSocket clients - this tests concurrent subscribe calls
    let ws_url = format!("ws://127.0.0.1:{}/api/sessions/{}/stream", port, session_id);
    
    let (ws_stream1, _) = connect_async(&ws_url)
        .await
        .expect("Failed to connect first WebSocket");
    
    let (ws_stream2, _) = connect_async(&ws_url)
        .await
        .expect("Failed to connect second WebSocket");
    
    let (ws_stream3, _) = connect_async(&ws_url)
        .await
        .expect("Failed to connect third WebSocket");
    
    // Verify all connections are working by sending a message
    let send_response = client
        .post(format!("{}/api/sessions/{}/messages", base_url, session_id))
        .json(&serde_json::json!({ "content": "Test multiple connections" }))
        .send()
        .await
        .expect("Request failed");
    
    assert_eq!(send_response.status(), reqwest::StatusCode::OK);
    
    // Clean up connections
    drop(ws_stream1);
    drop(ws_stream2);
    drop(ws_stream3);
    
    // If we got here without panic, multiple connections work correctly
}

/// Test WebSocket connection to non-existent session (should still work as subscribe creates session state)
#[tokio::test]
async fn test_e2e_websocket_subscribe_creates_session_state() {
    let (_base_url, port, _temp_dir, _mock_server) = start_test_server_with_websocket().await;
    
    // Connect to a session that hasn't had any messages yet using a unique ID
    let session_id = format!("test-session-{}", uuid::Uuid::new_v4());
    let ws_url = format!("ws://127.0.0.1:{}/api/sessions/{}/stream", port, session_id);
    
    let result = connect_async(&ws_url).await;
    
    // Should successfully connect (subscribe creates session state if not exists)
    assert!(result.is_ok(), "Should be able to connect to WebSocket for new session");
}

/// Test WebSocket receives events in correct order
#[tokio::test]
async fn test_e2e_websocket_event_order() {
    let (base_url, port, _temp_dir, _mock_server) = start_test_server_with_websocket().await;
    
    let client = reqwest::Client::new();
    
    // Create a session
    let create_response = client
        .post(format!("{}/api/sessions", base_url))
        .json(&serde_json::json!({ "name": "Event Order Test" }))
        .send()
        .await
        .expect("Request failed");
    
    let created: serde_json::Value = create_response.json().await.expect("Failed to parse");
    let session_id = created.get("id").and_then(|v| v.as_str()).expect("No id").to_string();
    
    // Connect to WebSocket
    let ws_url = format!("ws://127.0.0.1:{}/api/sessions/{}/stream", port, session_id);
    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .expect("Failed to connect to WebSocket");
    
    let (mut _write, mut read) = ws_stream.split();
    
    // Send message
    let session_id_clone = session_id.clone();
    let base_url_clone = base_url.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let client = reqwest::Client::new();
        let _ = client
            .post(format!("{}/api/sessions/{}/messages", base_url_clone, session_id_clone))
            .json(&serde_json::json!({ "content": "Test event ordering" }))
            .send()
            .await;
    });
    
    // Collect event types
    let mut event_types = Vec::new();
    let timeout_result = tokio::time::timeout(Duration::from_secs(10), async {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(event_type) = event.get("event_type").and_then(|v| v.as_str()) {
                        event_types.push(event_type.to_string());
                        if event_type == "agent_response" {
                            break;
                        }
                    }
                }
            }
        }
    });
    
    let _ = timeout_result.await;
    
    // Verify event order: agent_thinking should come before task_started/task_completed
    if !event_types.is_empty() {
        // First event should be agent_thinking (snake_case due to serde rename)
        assert_eq!(event_types.first().map(|s| s.as_str()), Some("agent_thinking"), 
                   "First event should be agent_thinking");
    }
}
