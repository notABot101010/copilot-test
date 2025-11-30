use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

const BASE_URL: &str = "http://localhost:4001";

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceResponse {
    id: String,
    name: String,
    created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContactResponse {
    id: String,
    workspace_id: String,
    visitor_id: String,
    name: Option<String>,
    email: Option<String>,
    created_at: i64,
    last_seen_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConversationResponse {
    id: String,
    workspace_id: String,
    contact_id: String,
    contact_name: Option<String>,
    status: String,
    last_message: Option<String>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageResponse {
    id: String,
    conversation_id: String,
    sender_type: String,
    sender_id: String,
    content: String,
    created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnalyticsResponse {
    top_pages: Vec<PageStats>,
    top_countries: Vec<CountryStats>,
    top_browsers: Vec<BrowserStats>,
    total_visitors: i64,
    total_page_views: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct PageStats {
    page_url: String,
    visitors: i64,
    page_views: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CountryStats {
    country: String,
    visitors: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BrowserStats {
    browser: String,
    visitors: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct VisitorInitResponse {
    contact_id: String,
    conversation_id: Option<String>,
    messages: Vec<MessageResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VisitorSendMessageResponse {
    conversation_id: String,
    message: MessageResponse,
}

async fn wait_for_server() {
    let client = Client::new();
    for _ in 0..30 {
        if client.get(format!("{}/api/workspaces", BASE_URL)).send().await.is_ok() {
            return;
        }
        sleep(Duration::from_millis(100)).await;
    }
    panic!("Server did not start in time");
}

#[tokio::test]
async fn test_workspace_crud() {
    wait_for_server().await;
    let client = Client::new();

    // Create workspace
    let res = client
        .post(format!("{}/api/workspaces", BASE_URL))
        .json(&serde_json::json!({ "name": "Test Workspace" }))
        .send()
        .await
        .expect("Failed to create workspace");

    assert_eq!(res.status(), 201);
    let workspace: WorkspaceResponse = res.json().await.expect("Failed to parse response");
    assert_eq!(workspace.name, "Test Workspace");
    assert!(!workspace.id.is_empty());

    // Get workspace
    let res = client
        .get(format!("{}/api/workspaces/{}", BASE_URL, workspace.id))
        .send()
        .await
        .expect("Failed to get workspace");

    assert_eq!(res.status(), 200);
    let fetched: WorkspaceResponse = res.json().await.expect("Failed to parse response");
    assert_eq!(fetched.id, workspace.id);
    assert_eq!(fetched.name, "Test Workspace");

    // List workspaces
    let res = client
        .get(format!("{}/api/workspaces", BASE_URL))
        .send()
        .await
        .expect("Failed to list workspaces");

    assert_eq!(res.status(), 200);
    let workspaces: Vec<WorkspaceResponse> = res.json().await.expect("Failed to parse response");
    assert!(workspaces.iter().any(|w| w.id == workspace.id));
}

#[tokio::test]
async fn test_contact_crud() {
    wait_for_server().await;
    let client = Client::new();

    // Create workspace first
    let res = client
        .post(format!("{}/api/workspaces", BASE_URL))
        .json(&serde_json::json!({ "name": "Contact Test Workspace" }))
        .send()
        .await
        .expect("Failed to create workspace");

    let workspace: WorkspaceResponse = res.json().await.expect("Failed to parse response");

    // Create contact
    let res = client
        .post(format!("{}/api/workspaces/{}/contacts", BASE_URL, workspace.id))
        .json(&serde_json::json!({
            "visitor_id": "visitor_123",
            "name": "John Doe",
            "email": "john@example.com"
        }))
        .send()
        .await
        .expect("Failed to create contact");

    assert_eq!(res.status(), 201);
    let contact: ContactResponse = res.json().await.expect("Failed to parse response");
    assert_eq!(contact.visitor_id, "visitor_123");
    assert_eq!(contact.name, Some("John Doe".to_string()));
    assert_eq!(contact.email, Some("john@example.com".to_string()));

    // Get contact
    let res = client
        .get(format!(
            "{}/api/workspaces/{}/contacts/{}",
            BASE_URL, workspace.id, contact.id
        ))
        .send()
        .await
        .expect("Failed to get contact");

    assert_eq!(res.status(), 200);
    let fetched: ContactResponse = res.json().await.expect("Failed to parse response");
    assert_eq!(fetched.id, contact.id);

    // Update contact
    let res = client
        .patch(format!(
            "{}/api/workspaces/{}/contacts/{}",
            BASE_URL, workspace.id, contact.id
        ))
        .json(&serde_json::json!({ "name": "Jane Doe" }))
        .send()
        .await
        .expect("Failed to update contact");

    assert_eq!(res.status(), 200);
    let updated: ContactResponse = res.json().await.expect("Failed to parse response");
    assert_eq!(updated.name, Some("Jane Doe".to_string()));

    // List contacts
    let res = client
        .get(format!("{}/api/workspaces/{}/contacts", BASE_URL, workspace.id))
        .send()
        .await
        .expect("Failed to list contacts");

    assert_eq!(res.status(), 200);
    let contacts: Vec<ContactResponse> = res.json().await.expect("Failed to parse response");
    assert!(contacts.iter().any(|c| c.id == contact.id));
}

#[tokio::test]
async fn test_conversation_and_messages() {
    wait_for_server().await;
    let client = Client::new();

    // Create workspace
    let res = client
        .post(format!("{}/api/workspaces", BASE_URL))
        .json(&serde_json::json!({ "name": "Message Test Workspace" }))
        .send()
        .await
        .expect("Failed to create workspace");

    let workspace: WorkspaceResponse = res.json().await.expect("Failed to parse response");

    // Create contact
    let res = client
        .post(format!("{}/api/workspaces/{}/contacts", BASE_URL, workspace.id))
        .json(&serde_json::json!({
            "visitor_id": "visitor_456",
            "name": "Test User"
        }))
        .send()
        .await
        .expect("Failed to create contact");

    let contact: ContactResponse = res.json().await.expect("Failed to parse response");

    // Create conversation
    let res = client
        .post(format!(
            "{}/api/workspaces/{}/conversations",
            BASE_URL, workspace.id
        ))
        .json(&serde_json::json!({ "contact_id": contact.id }))
        .send()
        .await
        .expect("Failed to create conversation");

    assert_eq!(res.status(), 201);
    let conversation: ConversationResponse = res.json().await.expect("Failed to parse response");
    assert_eq!(conversation.contact_id, contact.id);
    assert_eq!(conversation.status, "open");

    // Send message
    let res = client
        .post(format!(
            "{}/api/workspaces/{}/conversations/{}/messages",
            BASE_URL, workspace.id, conversation.id
        ))
        .json(&serde_json::json!({
            "sender_type": "agent",
            "sender_id": "agent-1",
            "content": "Hello, how can I help you?"
        }))
        .send()
        .await
        .expect("Failed to send message");

    assert_eq!(res.status(), 201);
    let message: MessageResponse = res.json().await.expect("Failed to parse response");
    assert_eq!(message.content, "Hello, how can I help you?");
    assert_eq!(message.sender_type, "agent");

    // List messages
    let res = client
        .get(format!(
            "{}/api/workspaces/{}/conversations/{}/messages",
            BASE_URL, workspace.id, conversation.id
        ))
        .send()
        .await
        .expect("Failed to list messages");

    assert_eq!(res.status(), 200);
    let messages: Vec<MessageResponse> = res.json().await.expect("Failed to parse response");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Hello, how can I help you?");

    // Update conversation status
    let res = client
        .patch(format!(
            "{}/api/workspaces/{}/conversations/{}",
            BASE_URL, workspace.id, conversation.id
        ))
        .json(&serde_json::json!({ "status": "closed" }))
        .send()
        .await
        .expect("Failed to update conversation");

    assert_eq!(res.status(), 200);
    let updated: ConversationResponse = res.json().await.expect("Failed to parse response");
    assert_eq!(updated.status, "closed");
}

#[tokio::test]
async fn test_visitor_flow() {
    wait_for_server().await;
    let client = Client::new();

    // Create workspace
    let res = client
        .post(format!("{}/api/workspaces", BASE_URL))
        .json(&serde_json::json!({ "name": "Visitor Test Workspace" }))
        .send()
        .await
        .expect("Failed to create workspace");

    let workspace: WorkspaceResponse = res.json().await.expect("Failed to parse response");

    // Visitor init
    let res = client
        .post(format!(
            "{}/api/workspaces/{}/visitor/init",
            BASE_URL, workspace.id
        ))
        .json(&serde_json::json!({ "visitor_id": "v_test_visitor" }))
        .send()
        .await
        .expect("Failed to init visitor");

    assert_eq!(res.status(), 200);
    let init: VisitorInitResponse = res.json().await.expect("Failed to parse response");
    assert!(!init.contact_id.is_empty());
    assert!(init.conversation_id.is_none()); // No conversation yet
    assert!(init.messages.is_empty());

    // Visitor sends message (creates conversation)
    let res = client
        .post(format!(
            "{}/api/workspaces/{}/visitor/message",
            BASE_URL, workspace.id
        ))
        .json(&serde_json::json!({
            "visitor_id": "v_test_visitor",
            "content": "Hello, I need help!"
        }))
        .send()
        .await
        .expect("Failed to send visitor message");

    assert_eq!(res.status(), 201);
    let msg_res: VisitorSendMessageResponse = res.json().await.expect("Failed to parse response");
    assert!(!msg_res.conversation_id.is_empty());
    assert_eq!(msg_res.message.content, "Hello, I need help!");
    assert_eq!(msg_res.message.sender_type, "visitor");

    // Visitor init again - should have conversation now
    let res = client
        .post(format!(
            "{}/api/workspaces/{}/visitor/init",
            BASE_URL, workspace.id
        ))
        .json(&serde_json::json!({ "visitor_id": "v_test_visitor" }))
        .send()
        .await
        .expect("Failed to init visitor");

    assert_eq!(res.status(), 200);
    let init2: VisitorInitResponse = res.json().await.expect("Failed to parse response");
    assert_eq!(init2.contact_id, init.contact_id);
    assert!(init2.conversation_id.is_some());
    assert_eq!(init2.messages.len(), 1);
}

#[tokio::test]
async fn test_analytics() {
    wait_for_server().await;
    let client = Client::new();

    // Create workspace
    let res = client
        .post(format!("{}/api/workspaces", BASE_URL))
        .json(&serde_json::json!({ "name": "Analytics Test Workspace" }))
        .send()
        .await
        .expect("Failed to create workspace");

    let workspace: WorkspaceResponse = res.json().await.expect("Failed to parse response");

    // Track page views
    for page in ["/home", "/about", "/pricing", "/home", "/home"] {
        let res = client
            .post(format!("{}/api/workspaces/{}/track", BASE_URL, workspace.id))
            .header("User-Agent", "Mozilla/5.0 Chrome/120.0")
            .json(&serde_json::json!({
                "page_url": page,
                "page_title": Some(format!("{} Page", page)),
                "referrer": None::<String>
            }))
            .send()
            .await
            .expect("Failed to track page view");

        assert_eq!(res.status(), 201);
    }

    // Get analytics
    let res = client
        .get(format!(
            "{}/api/workspaces/{}/analytics?days=30",
            BASE_URL, workspace.id
        ))
        .send()
        .await
        .expect("Failed to get analytics");

    assert_eq!(res.status(), 200);
    let analytics: AnalyticsResponse = res.json().await.expect("Failed to parse response");
    
    // We should have at least some data
    assert!(analytics.total_page_views >= 5);
    assert!(!analytics.top_pages.is_empty());
    assert!(!analytics.top_browsers.is_empty());
    
    // Chrome should be detected
    assert!(analytics.top_browsers.iter().any(|b| b.browser == "Chrome"));
}

#[tokio::test]
async fn test_contact_conversations() {
    wait_for_server().await;
    let client = Client::new();

    // Create workspace
    let res = client
        .post(format!("{}/api/workspaces", BASE_URL))
        .json(&serde_json::json!({ "name": "Contact Conversations Test" }))
        .send()
        .await
        .expect("Failed to create workspace");

    let workspace: WorkspaceResponse = res.json().await.expect("Failed to parse response");

    // Create contact
    let res = client
        .post(format!("{}/api/workspaces/{}/contacts", BASE_URL, workspace.id))
        .json(&serde_json::json!({ "visitor_id": "conv_test_visitor" }))
        .send()
        .await
        .expect("Failed to create contact");

    let contact: ContactResponse = res.json().await.expect("Failed to parse response");

    // Create multiple conversations
    for _ in 0..3 {
        let _res = client
            .post(format!(
                "{}/api/workspaces/{}/conversations",
                BASE_URL, workspace.id
            ))
            .json(&serde_json::json!({ "contact_id": contact.id }))
            .send()
            .await
            .expect("Failed to create conversation");
    }

    // Get contact conversations
    let res = client
        .get(format!(
            "{}/api/workspaces/{}/contacts/{}/conversations",
            BASE_URL, workspace.id, contact.id
        ))
        .send()
        .await
        .expect("Failed to get contact conversations");

    assert_eq!(res.status(), 200);
    let conversations: Vec<ConversationResponse> = res.json().await.expect("Failed to parse response");
    assert_eq!(conversations.len(), 3);
}
