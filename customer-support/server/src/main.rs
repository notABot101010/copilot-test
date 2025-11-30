use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, Path, Query, State,
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};

const BROADCAST_CAPACITY: usize = 100;

type WorkspaceChannels = Arc<RwLock<HashMap<String, broadcast::Sender<WsEvent>>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum WsEvent {
    #[serde(rename = "new_message")]
    NewMessage { conversation_id: String, message: MessageResponse },
    #[serde(rename = "new_conversation")]
    NewConversation { conversation: ConversationResponse },
    #[serde(rename = "conversation_updated")]
    ConversationUpdated { conversation: ConversationResponse },
}

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    channels: WorkspaceChannels,
}

// Database models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct Workspace {
    id: String,
    name: String,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct Contact {
    id: String,
    workspace_id: String,
    visitor_id: String,
    name: Option<String>,
    email: Option<String>,
    created_at: i64,
    last_seen_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct Conversation {
    id: String,
    workspace_id: String,
    contact_id: String,
    status: String,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct DbMessage {
    id: String,
    conversation_id: String,
    sender_type: String,
    sender_id: String,
    content: String,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct PageView {
    id: String,
    workspace_id: String,
    anonymous_id: String,
    page_url: String,
    page_title: Option<String>,
    referrer: Option<String>,
    browser: String,
    country: Option<String>,
    created_at: i64,
}

// Request/Response types
#[derive(Debug, Deserialize)]
struct CreateWorkspaceRequest {
    name: String,
}

#[derive(Debug, Serialize)]
struct WorkspaceResponse {
    id: String,
    name: String,
    created_at: i64,
}

#[derive(Debug, Deserialize)]
struct CreateContactRequest {
    visitor_id: String,
    name: Option<String>,
    email: Option<String>,
}

#[derive(Debug, Serialize)]
struct ContactResponse {
    id: String,
    workspace_id: String,
    visitor_id: String,
    name: Option<String>,
    email: Option<String>,
    created_at: i64,
    last_seen_at: i64,
}

#[derive(Debug, Deserialize)]
struct UpdateContactRequest {
    name: Option<String>,
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateConversationRequest {
    contact_id: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
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

#[derive(Debug, Deserialize)]
struct SendMessageRequest {
    sender_type: String,
    sender_id: String,
    content: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
struct MessageResponse {
    id: String,
    conversation_id: String,
    sender_type: String,
    sender_id: String,
    content: String,
    created_at: i64,
}

#[derive(Debug, Deserialize)]
struct TrackPageViewRequest {
    page_url: String,
    page_title: Option<String>,
    referrer: Option<String>,
}

#[derive(Debug, Serialize)]
struct AnalyticsResponse {
    top_pages: Vec<PageStats>,
    top_countries: Vec<CountryStats>,
    top_browsers: Vec<BrowserStats>,
    total_visitors: i64,
    total_page_views: i64,
}

#[derive(Debug, Serialize)]
struct PageStats {
    page_url: String,
    visitors: i64,
    page_views: i64,
}

#[derive(Debug, Serialize)]
struct CountryStats {
    country: String,
    visitors: i64,
}

#[derive(Debug, Serialize)]
struct BrowserStats {
    browser: String,
    visitors: i64,
}

#[derive(Debug, Deserialize)]
struct AnalyticsQuery {
    days: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct VisitorInitRequest {
    visitor_id: String,
}

#[derive(Debug, Serialize)]
struct VisitorInitResponse {
    contact_id: String,
    conversation_id: Option<String>,
    messages: Vec<MessageResponse>,
}

async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS workspaces (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS contacts (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL,
            visitor_id TEXT NOT NULL,
            name TEXT,
            email TEXT,
            created_at INTEGER NOT NULL,
            last_seen_at INTEGER NOT NULL,
            FOREIGN KEY (workspace_id) REFERENCES workspaces(id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_contacts_workspace_visitor 
        ON contacts(workspace_id, visitor_id)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL,
            contact_id TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'open',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (workspace_id) REFERENCES workspaces(id),
            FOREIGN KEY (contact_id) REFERENCES contacts(id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            sender_type TEXT NOT NULL,
            sender_id TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (conversation_id) REFERENCES conversations(id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS page_views (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL,
            anonymous_id TEXT NOT NULL,
            page_url TEXT NOT NULL,
            page_title TEXT,
            referrer TEXT,
            browser TEXT NOT NULL,
            country TEXT,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (workspace_id) REFERENCES workspaces(id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_page_views_workspace_created 
        ON page_views(workspace_id, created_at)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

fn get_current_time() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn extract_browser(user_agent: &str) -> String {
    let ua = user_agent.to_lowercase();
    if ua.contains("firefox") {
        "Firefox".to_string()
    } else if ua.contains("edg/") || ua.contains("edge") {
        "Edge".to_string()
    } else if ua.contains("chrome") && !ua.contains("chromium") {
        "Chrome".to_string()
    } else if ua.contains("safari") && !ua.contains("chrome") {
        "Safari".to_string()
    } else if ua.contains("opera") || ua.contains("opr/") {
        "Opera".to_string()
    } else {
        "Other".to_string()
    }
}

fn hash_for_anonymous_id(ip: &str, user_agent: &str) -> String {
    let input = format!("{}||{}", ip, user_agent);
    let hash = blake3::hash(input.as_bytes());
    hash.to_hex().to_string()
}

fn extract_ip(addr: &SocketAddr, headers: &HeaderMap) -> String {
    // Check for X-Forwarded-For header first (for proxies)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            if let Some(first_ip) = value.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }
    // Check X-Real-IP header
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            return value.to_string();
        }
    }
    // Fallback to connection IP
    addr.ip().to_string()
}

// Workspace endpoints
async fn create_workspace(
    State(state): State<AppState>,
    Json(payload): Json<CreateWorkspaceRequest>,
) -> Result<(StatusCode, Json<WorkspaceResponse>), StatusCode> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = get_current_time();

    sqlx::query("INSERT INTO workspaces (id, name, created_at) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&payload.name)
        .bind(now)
        .execute(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to create workspace: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((
        StatusCode::CREATED,
        Json(WorkspaceResponse {
            id,
            name: payload.name,
            created_at: now,
        }),
    ))
}

async fn list_workspaces(
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkspaceResponse>>, StatusCode> {
    let workspaces = sqlx::query_as::<_, Workspace>(
        "SELECT id, name, created_at FROM workspaces ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to list workspaces: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(
        workspaces
            .into_iter()
            .map(|w| WorkspaceResponse {
                id: w.id,
                name: w.name,
                created_at: w.created_at,
            })
            .collect(),
    ))
}

async fn get_workspace(
    Path(workspace_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<WorkspaceResponse>, StatusCode> {
    let workspace = sqlx::query_as::<_, Workspace>(
        "SELECT id, name, created_at FROM workspaces WHERE id = ?",
    )
    .bind(&workspace_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get workspace: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(WorkspaceResponse {
        id: workspace.id,
        name: workspace.name,
        created_at: workspace.created_at,
    }))
}

// Contact endpoints
async fn create_contact(
    Path(workspace_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<CreateContactRequest>,
) -> Result<(StatusCode, Json<ContactResponse>), StatusCode> {
    // Check if contact already exists for this visitor
    let existing = sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts WHERE workspace_id = ? AND visitor_id = ?",
    )
    .bind(&workspace_id)
    .bind(&payload.visitor_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to check existing contact: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some(contact) = existing {
        // Update last_seen_at
        let now = get_current_time();
        sqlx::query("UPDATE contacts SET last_seen_at = ? WHERE id = ?")
            .bind(now)
            .bind(&contact.id)
            .execute(&state.db)
            .await
            .map_err(|err| {
                tracing::error!("Failed to update contact: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        return Ok((
            StatusCode::OK,
            Json(ContactResponse {
                id: contact.id,
                workspace_id: contact.workspace_id,
                visitor_id: contact.visitor_id,
                name: contact.name,
                email: contact.email,
                created_at: contact.created_at,
                last_seen_at: now,
            }),
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = get_current_time();

    sqlx::query(
        "INSERT INTO contacts (id, workspace_id, visitor_id, name, email, created_at, last_seen_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&workspace_id)
    .bind(&payload.visitor_id)
    .bind(&payload.name)
    .bind(&payload.email)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to create contact: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((
        StatusCode::CREATED,
        Json(ContactResponse {
            id,
            workspace_id,
            visitor_id: payload.visitor_id,
            name: payload.name,
            email: payload.email,
            created_at: now,
            last_seen_at: now,
        }),
    ))
}

async fn list_contacts(
    Path(workspace_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ContactResponse>>, StatusCode> {
    let contacts = sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts WHERE workspace_id = ? ORDER BY last_seen_at DESC",
    )
    .bind(&workspace_id)
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to list contacts: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(
        contacts
            .into_iter()
            .map(|c| ContactResponse {
                id: c.id,
                workspace_id: c.workspace_id,
                visitor_id: c.visitor_id,
                name: c.name,
                email: c.email,
                created_at: c.created_at,
                last_seen_at: c.last_seen_at,
            })
            .collect(),
    ))
}

async fn get_contact(
    Path((workspace_id, contact_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ContactResponse>, StatusCode> {
    let contact = sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts WHERE id = ? AND workspace_id = ?",
    )
    .bind(&contact_id)
    .bind(&workspace_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get contact: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ContactResponse {
        id: contact.id,
        workspace_id: contact.workspace_id,
        visitor_id: contact.visitor_id,
        name: contact.name,
        email: contact.email,
        created_at: contact.created_at,
        last_seen_at: contact.last_seen_at,
    }))
}

async fn update_contact(
    Path((workspace_id, contact_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateContactRequest>,
) -> Result<Json<ContactResponse>, StatusCode> {
    let contact = sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts WHERE id = ? AND workspace_id = ?",
    )
    .bind(&contact_id)
    .bind(&workspace_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get contact: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    let name = payload.name.or(contact.name.clone());
    let email = payload.email.or(contact.email.clone());
    let now = get_current_time();

    sqlx::query("UPDATE contacts SET name = ?, email = ?, last_seen_at = ? WHERE id = ?")
        .bind(&name)
        .bind(&email)
        .bind(now)
        .bind(&contact_id)
        .execute(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update contact: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ContactResponse {
        id: contact.id,
        workspace_id: contact.workspace_id,
        visitor_id: contact.visitor_id,
        name,
        email,
        created_at: contact.created_at,
        last_seen_at: now,
    }))
}

// Get contact conversations with messages
async fn get_contact_conversations(
    Path((workspace_id, contact_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ConversationResponse>>, StatusCode> {
    let conversations = sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations WHERE workspace_id = ? AND contact_id = ? ORDER BY updated_at DESC",
    )
    .bind(&workspace_id)
    .bind(&contact_id)
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to list contact conversations: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut result = Vec::new();
    for conv in conversations {
        let last_message = sqlx::query_as::<_, DbMessage>(
            "SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at DESC LIMIT 1",
        )
        .bind(&conv.id)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to get last message: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let contact = sqlx::query_as::<_, Contact>("SELECT * FROM contacts WHERE id = ?")
            .bind(&conv.contact_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|err| {
                tracing::error!("Failed to get contact: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        result.push(ConversationResponse {
            id: conv.id,
            workspace_id: conv.workspace_id,
            contact_id: conv.contact_id,
            contact_name: contact.and_then(|c| c.name),
            status: conv.status,
            last_message: last_message.map(|m| m.content),
            created_at: conv.created_at,
            updated_at: conv.updated_at,
        });
    }

    Ok(Json(result))
}

// Conversation endpoints
async fn create_conversation(
    Path(workspace_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<CreateConversationRequest>,
) -> Result<(StatusCode, Json<ConversationResponse>), StatusCode> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = get_current_time();

    sqlx::query(
        "INSERT INTO conversations (id, workspace_id, contact_id, status, created_at, updated_at) VALUES (?, ?, ?, 'open', ?, ?)",
    )
    .bind(&id)
    .bind(&workspace_id)
    .bind(&payload.contact_id)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to create conversation: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let contact = sqlx::query_as::<_, Contact>("SELECT * FROM contacts WHERE id = ?")
        .bind(&payload.contact_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to get contact: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response = ConversationResponse {
        id: id.clone(),
        workspace_id: workspace_id.clone(),
        contact_id: payload.contact_id,
        contact_name: contact.and_then(|c| c.name),
        status: "open".to_string(),
        last_message: None,
        created_at: now,
        updated_at: now,
    };

    // Broadcast new conversation event
    broadcast_event(&state, &workspace_id, WsEvent::NewConversation {
        conversation: response.clone(),
    }).await;

    Ok((StatusCode::CREATED, Json(response)))
}

async fn list_conversations(
    Path(workspace_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ConversationResponse>>, StatusCode> {
    let conversations = sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations WHERE workspace_id = ? ORDER BY updated_at DESC",
    )
    .bind(&workspace_id)
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to list conversations: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut result = Vec::new();
    for conv in conversations {
        let last_message = sqlx::query_as::<_, DbMessage>(
            "SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at DESC LIMIT 1",
        )
        .bind(&conv.id)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to get last message: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let contact = sqlx::query_as::<_, Contact>("SELECT * FROM contacts WHERE id = ?")
            .bind(&conv.contact_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|err| {
                tracing::error!("Failed to get contact: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        result.push(ConversationResponse {
            id: conv.id,
            workspace_id: conv.workspace_id,
            contact_id: conv.contact_id,
            contact_name: contact.and_then(|c| c.name),
            status: conv.status,
            last_message: last_message.map(|m| m.content),
            created_at: conv.created_at,
            updated_at: conv.updated_at,
        });
    }

    Ok(Json(result))
}

async fn get_conversation(
    Path((workspace_id, conversation_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ConversationResponse>, StatusCode> {
    let conversation = sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations WHERE id = ? AND workspace_id = ?",
    )
    .bind(&conversation_id)
    .bind(&workspace_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get conversation: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    let last_message = sqlx::query_as::<_, DbMessage>(
        "SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&conversation_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get last message: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let contact = sqlx::query_as::<_, Contact>("SELECT * FROM contacts WHERE id = ?")
        .bind(&conversation.contact_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to get contact: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ConversationResponse {
        id: conversation.id,
        workspace_id: conversation.workspace_id,
        contact_id: conversation.contact_id,
        contact_name: contact.and_then(|c| c.name),
        status: conversation.status,
        last_message: last_message.map(|m| m.content),
        created_at: conversation.created_at,
        updated_at: conversation.updated_at,
    }))
}

#[derive(Debug, Deserialize)]
struct UpdateConversationRequest {
    status: String,
}

async fn update_conversation(
    Path((workspace_id, conversation_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateConversationRequest>,
) -> Result<Json<ConversationResponse>, StatusCode> {
    let now = get_current_time();

    sqlx::query("UPDATE conversations SET status = ?, updated_at = ? WHERE id = ? AND workspace_id = ?")
        .bind(&payload.status)
        .bind(now)
        .bind(&conversation_id)
        .bind(&workspace_id)
        .execute(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update conversation: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    get_conversation(Path((workspace_id, conversation_id)), State(state)).await
}

// Message endpoints
async fn send_message(
    Path((workspace_id, conversation_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), StatusCode> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = get_current_time();

    // Verify conversation exists
    let conversation = sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations WHERE id = ? AND workspace_id = ?",
    )
    .bind(&conversation_id)
    .bind(&workspace_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get conversation: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    sqlx::query(
        "INSERT INTO messages (id, conversation_id, sender_type, sender_id, content, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&conversation_id)
    .bind(&payload.sender_type)
    .bind(&payload.sender_id)
    .bind(&payload.content)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to send message: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Update conversation timestamp
    sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&conversation_id)
        .execute(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update conversation: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let message = MessageResponse {
        id: id.clone(),
        conversation_id: conversation_id.clone(),
        sender_type: payload.sender_type,
        sender_id: payload.sender_id,
        content: payload.content,
        created_at: now,
    };

    // Broadcast message event
    broadcast_event(&state, &conversation.workspace_id, WsEvent::NewMessage {
        conversation_id: conversation_id.clone(),
        message: message.clone(),
    }).await;

    Ok((StatusCode::CREATED, Json(message)))
}

async fn list_messages(
    Path((workspace_id, conversation_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<Vec<MessageResponse>>, StatusCode> {
    // Verify conversation exists
    sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations WHERE id = ? AND workspace_id = ?",
    )
    .bind(&conversation_id)
    .bind(&workspace_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get conversation: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    let messages = sqlx::query_as::<_, DbMessage>(
        "SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at ASC",
    )
    .bind(&conversation_id)
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to list messages: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(
        messages
            .into_iter()
            .map(|m| MessageResponse {
                id: m.id,
                conversation_id: m.conversation_id,
                sender_type: m.sender_type,
                sender_id: m.sender_id,
                content: m.content,
                created_at: m.created_at,
            })
            .collect(),
    ))
}

// Analytics endpoints
async fn track_page_view(
    Path(workspace_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<TrackPageViewRequest>,
) -> Result<StatusCode, StatusCode> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = get_current_time();

    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown");
    
    let ip = extract_ip(&addr, &headers);
    let anonymous_id = hash_for_anonymous_id(&ip, user_agent);
    let browser = extract_browser(user_agent);

    // Country detection would require GeoIP - for now we'll leave it as None
    let country: Option<String> = None;

    sqlx::query(
        "INSERT INTO page_views (id, workspace_id, anonymous_id, page_url, page_title, referrer, browser, country, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&workspace_id)
    .bind(&anonymous_id)
    .bind(&payload.page_url)
    .bind(&payload.page_title)
    .bind(&payload.referrer)
    .bind(&browser)
    .bind(&country)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to track page view: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::CREATED)
}

async fn get_analytics(
    Path(workspace_id): Path<String>,
    Query(query): Query<AnalyticsQuery>,
    State(state): State<AppState>,
) -> Result<Json<AnalyticsResponse>, StatusCode> {
    let days = query.days.unwrap_or(30);
    let now = get_current_time();
    let since = now - (days * 24 * 60 * 60 * 1000);

    // Total visitors (unique anonymous_ids)
    let total_visitors: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT anonymous_id) FROM page_views WHERE workspace_id = ? AND created_at >= ?",
    )
    .bind(&workspace_id)
    .bind(since)
    .fetch_one(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get total visitors: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Total page views
    let total_page_views: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM page_views WHERE workspace_id = ? AND created_at >= ?",
    )
    .bind(&workspace_id)
    .bind(since)
    .fetch_one(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get total page views: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Top pages
    let top_pages: Vec<(String, i64, i64)> = sqlx::query_as(
        r#"
        SELECT page_url, COUNT(DISTINCT anonymous_id) as visitors, COUNT(*) as page_views
        FROM page_views 
        WHERE workspace_id = ? AND created_at >= ?
        GROUP BY page_url
        ORDER BY visitors DESC
        LIMIT 10
        "#,
    )
    .bind(&workspace_id)
    .bind(since)
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get top pages: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Top countries
    let top_countries: Vec<(Option<String>, i64)> = sqlx::query_as(
        r#"
        SELECT country, COUNT(DISTINCT anonymous_id) as visitors
        FROM page_views 
        WHERE workspace_id = ? AND created_at >= ?
        GROUP BY country
        ORDER BY visitors DESC
        LIMIT 10
        "#,
    )
    .bind(&workspace_id)
    .bind(since)
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get top countries: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Top browsers
    let top_browsers: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT browser, COUNT(DISTINCT anonymous_id) as visitors
        FROM page_views 
        WHERE workspace_id = ? AND created_at >= ?
        GROUP BY browser
        ORDER BY visitors DESC
        LIMIT 10
        "#,
    )
    .bind(&workspace_id)
    .bind(since)
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get top browsers: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(AnalyticsResponse {
        top_pages: top_pages
            .into_iter()
            .map(|(page_url, visitors, page_views)| PageStats {
                page_url,
                visitors,
                page_views,
            })
            .collect(),
        top_countries: top_countries
            .into_iter()
            .map(|(country, visitors)| CountryStats {
                country: country.unwrap_or_else(|| "Unknown".to_string()),
                visitors,
            })
            .collect(),
        top_browsers: top_browsers
            .into_iter()
            .map(|(browser, visitors)| BrowserStats { browser, visitors })
            .collect(),
        total_visitors: total_visitors.0,
        total_page_views: total_page_views.0,
    }))
}

// Visitor init endpoint (for SDK)
async fn visitor_init(
    Path(workspace_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<VisitorInitRequest>,
) -> Result<Json<VisitorInitResponse>, StatusCode> {
    // Create or get contact
    let contact = sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts WHERE workspace_id = ? AND visitor_id = ?",
    )
    .bind(&workspace_id)
    .bind(&payload.visitor_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get contact: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let now = get_current_time();
    let contact_id = if let Some(c) = contact {
        sqlx::query("UPDATE contacts SET last_seen_at = ? WHERE id = ?")
            .bind(now)
            .bind(&c.id)
            .execute(&state.db)
            .await
            .map_err(|err| {
                tracing::error!("Failed to update contact: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        c.id
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO contacts (id, workspace_id, visitor_id, created_at, last_seen_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&workspace_id)
        .bind(&payload.visitor_id)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to create contact: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        id
    };

    // Get or create conversation
    let conversation = sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations WHERE workspace_id = ? AND contact_id = ? AND status = 'open' ORDER BY updated_at DESC LIMIT 1",
    )
    .bind(&workspace_id)
    .bind(&contact_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get conversation: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (conversation_id, messages) = if let Some(conv) = conversation {
        let messages = sqlx::query_as::<_, DbMessage>(
            "SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at ASC",
        )
        .bind(&conv.id)
        .fetch_all(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to get messages: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        (
            Some(conv.id),
            messages
                .into_iter()
                .map(|m| MessageResponse {
                    id: m.id,
                    conversation_id: m.conversation_id,
                    sender_type: m.sender_type,
                    sender_id: m.sender_id,
                    content: m.content,
                    created_at: m.created_at,
                })
                .collect(),
        )
    } else {
        (None, vec![])
    };

    Ok(Json(VisitorInitResponse {
        contact_id,
        conversation_id,
        messages,
    }))
}

// Visitor send message (creates conversation if needed)
#[derive(Debug, Deserialize)]
struct VisitorSendMessageRequest {
    visitor_id: String,
    content: String,
    conversation_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct VisitorSendMessageResponse {
    conversation_id: String,
    message: MessageResponse,
}

async fn visitor_send_message(
    Path(workspace_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<VisitorSendMessageRequest>,
) -> Result<(StatusCode, Json<VisitorSendMessageResponse>), StatusCode> {
    let now = get_current_time();

    // Get contact
    let contact = sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts WHERE workspace_id = ? AND visitor_id = ?",
    )
    .bind(&workspace_id)
    .bind(&payload.visitor_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get contact: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Get or create conversation
    let conversation_id = if let Some(conv_id) = payload.conversation_id {
        conv_id
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO conversations (id, workspace_id, contact_id, status, created_at, updated_at) VALUES (?, ?, ?, 'open', ?, ?)",
        )
        .bind(&id)
        .bind(&workspace_id)
        .bind(&contact.id)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to create conversation: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Broadcast new conversation
        let conv_response = ConversationResponse {
            id: id.clone(),
            workspace_id: workspace_id.clone(),
            contact_id: contact.id.clone(),
            contact_name: contact.name.clone(),
            status: "open".to_string(),
            last_message: None,
            created_at: now,
            updated_at: now,
        };
        broadcast_event(&state, &workspace_id, WsEvent::NewConversation {
            conversation: conv_response,
        }).await;

        id
    };

    // Create message
    let message_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, sender_type, sender_id, content, created_at) VALUES (?, ?, 'visitor', ?, ?, ?)",
    )
    .bind(&message_id)
    .bind(&conversation_id)
    .bind(&contact.id)
    .bind(&payload.content)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to create message: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Update conversation timestamp
    sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&conversation_id)
        .execute(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update conversation: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let message = MessageResponse {
        id: message_id,
        conversation_id: conversation_id.clone(),
        sender_type: "visitor".to_string(),
        sender_id: contact.id,
        content: payload.content,
        created_at: now,
    };

    // Broadcast message
    broadcast_event(&state, &workspace_id, WsEvent::NewMessage {
        conversation_id: conversation_id.clone(),
        message: message.clone(),
    }).await;

    Ok((
        StatusCode::CREATED,
        Json(VisitorSendMessageResponse {
            conversation_id,
            message,
        }),
    ))
}

// WebSocket handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(workspace_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, workspace_id, state))
}

async fn handle_socket(socket: WebSocket, workspace_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Get or create broadcast channel for this workspace
    let rx = {
        let mut channels = state.channels.write().await;
        let tx = channels
            .entry(workspace_id.clone())
            .or_insert_with(|| broadcast::channel(BROADCAST_CAPACITY).0);
        tx.subscribe()
    };

    let mut rx = rx;

    // Spawn task to forward broadcast messages to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&event) {
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages (for future use - e.g., typing indicators)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    tracing::info!("WebSocket connection closed for workspace {}", workspace_id);
}

async fn broadcast_event(state: &AppState, workspace_id: &str, event: WsEvent) {
    let channels = state.channels.read().await;
    if let Some(tx) = channels.get(workspace_id) {
        let _ = tx.send(event);
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:customer_support.db?mode=rwc".to_string());
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    init_db(&pool).await.expect("Failed to initialize database");

    let state = AppState {
        db: pool,
        channels: Arc::new(RwLock::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Workspaces
        .route("/api/workspaces", get(list_workspaces).post(create_workspace))
        .route("/api/workspaces/{workspace_id}", get(get_workspace))
        // Contacts
        .route("/api/workspaces/{workspace_id}/contacts", get(list_contacts).post(create_contact))
        .route("/api/workspaces/{workspace_id}/contacts/{contact_id}", get(get_contact).patch(update_contact))
        .route("/api/workspaces/{workspace_id}/contacts/{contact_id}/conversations", get(get_contact_conversations))
        // Conversations
        .route("/api/workspaces/{workspace_id}/conversations", get(list_conversations).post(create_conversation))
        .route("/api/workspaces/{workspace_id}/conversations/{conversation_id}", get(get_conversation).patch(update_conversation))
        .route("/api/workspaces/{workspace_id}/conversations/{conversation_id}/messages", get(list_messages).post(send_message))
        // Analytics
        .route("/api/workspaces/{workspace_id}/analytics", get(get_analytics))
        .route("/api/workspaces/{workspace_id}/track", post(track_page_view))
        // Visitor (SDK) endpoints
        .route("/api/workspaces/{workspace_id}/visitor/init", post(visitor_init))
        .route("/api/workspaces/{workspace_id}/visitor/message", post(visitor_send_message))
        // WebSocket
        .route("/ws/workspaces/{workspace_id}", get(ws_handler))
        .layer(cors)
        .with_state(state)
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4001").await.expect("Failed to bind");
    tracing::info!("Customer Support server listening on http://0.0.0.0:4001");
    axum::serve(listener, app).await.expect("Failed to start server");
}
