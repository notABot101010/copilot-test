use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

// Long-polling state: maps username to pending messages
type PendingMessages = Arc<RwLock<HashMap<String, Vec<EncryptedMessage>>>>;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    pending: PendingMessages,
}

// Database models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct User {
    username: String,
    // Base64-encoded encrypted identity public key
    identity_public_key: String,
    // Base64-encoded salt for PBKDF2
    salt: String,
    // Base64-encoded encrypted identity private key (encrypted with master key)
    encrypted_identity_private_key: String,
    // Base64-encoded IV for identity key encryption
    identity_key_iv: String,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct PreKeyBundle {
    id: String,
    username: String,
    // Base64-encoded signed pre-key public (X25519)
    signed_prekey_public: String,
    // Base64-encoded signature of the signed pre-key (by identity key)
    signed_prekey_signature: String,
    // Base64-encoded encrypted signed pre-key private
    encrypted_signed_prekey_private: String,
    // IV for signed pre-key private encryption
    signed_prekey_iv: String,
    // One-time pre-keys (JSON array of base64-encoded public keys)
    one_time_prekeys: String,
    // Encrypted one-time pre-key privates (JSON array)
    encrypted_one_time_prekey_privates: String,
    // IVs for one-time pre-key encryption (JSON array)
    one_time_prekey_ivs: String,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct StoredMessage {
    id: String,
    // Sealed sender envelope (recipient can't see sender until decryption)
    sealed_sender_envelope: String,
    recipient_username: String,
    created_at: i64,
    delivered: bool,
}

// Request/Response types
#[derive(Debug, Deserialize)]
struct RegisterRequest {
    username: String,
    identity_public_key: String,
    salt: String,
    encrypted_identity_private_key: String,
    identity_key_iv: String,
}

#[derive(Debug, Serialize)]
struct RegisterResponse {
    username: String,
    created_at: i64,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    username: String,
    identity_public_key: String,
    salt: String,
    encrypted_identity_private_key: String,
    identity_key_iv: String,
}

#[derive(Debug, Deserialize)]
struct UploadPreKeyBundleRequest {
    signed_prekey_public: String,
    signed_prekey_signature: String,
    encrypted_signed_prekey_private: String,
    signed_prekey_iv: String,
    one_time_prekeys: Vec<String>,
    encrypted_one_time_prekey_privates: Vec<String>,
    one_time_prekey_ivs: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PreKeyBundleResponse {
    identity_public_key: String,
    signed_prekey_public: String,
    signed_prekey_signature: String,
    one_time_prekey: Option<String>,
}

#[derive(Debug, Serialize)]
struct MyPreKeyBundleResponse {
    encrypted_signed_prekey_private: String,
    signed_prekey_iv: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedMessage {
    id: String,
    sealed_sender_envelope: String,
    created_at: i64,
}

#[derive(Debug, Deserialize)]
struct SendMessageRequest {
    recipient_username: String,
    sealed_sender_envelope: String,
}

#[derive(Debug, Serialize)]
struct SendMessageResponse {
    id: String,
    created_at: i64,
}

#[derive(Debug, Deserialize)]
struct PollMessagesQuery {
    #[serde(default)]
    timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
struct PollMessagesResponse {
    messages: Vec<EncryptedMessage>,
    timestamp: i64,
}

#[derive(Debug, Deserialize)]
struct AckMessagesRequest {
    message_ids: Vec<String>,
}

fn get_current_time() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY,
            identity_public_key TEXT NOT NULL,
            salt TEXT NOT NULL,
            encrypted_identity_private_key TEXT NOT NULL,
            identity_key_iv TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS prekey_bundles (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL,
            signed_prekey_public TEXT NOT NULL,
            signed_prekey_signature TEXT NOT NULL,
            encrypted_signed_prekey_private TEXT NOT NULL,
            signed_prekey_iv TEXT NOT NULL,
            one_time_prekeys TEXT NOT NULL,
            encrypted_one_time_prekey_privates TEXT NOT NULL,
            one_time_prekey_ivs TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (username) REFERENCES users(username)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            sealed_sender_envelope TEXT NOT NULL,
            recipient_username TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            delivered INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (recipient_username) REFERENCES users(username)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_messages_recipient 
        ON messages(recipient_username, delivered)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

// User registration
async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), StatusCode> {
    let now = get_current_time();

    // Check if user already exists
    let existing: Option<User> =
        sqlx::query_as("SELECT * FROM users WHERE username = ?")
            .bind(&payload.username)
            .fetch_optional(&state.db)
            .await
            .map_err(|err| {
                tracing::error!("Failed to check existing user: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    if existing.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    sqlx::query(
        "INSERT INTO users (username, identity_public_key, salt, encrypted_identity_private_key, identity_key_iv, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&payload.username)
    .bind(&payload.identity_public_key)
    .bind(&payload.salt)
    .bind(&payload.encrypted_identity_private_key)
    .bind(&payload.identity_key_iv)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to create user: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            username: payload.username,
            created_at: now,
        }),
    ))
}

// User login (retrieve encrypted key material)
async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let user: User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to get user: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(LoginResponse {
        username: user.username,
        identity_public_key: user.identity_public_key,
        salt: user.salt,
        encrypted_identity_private_key: user.encrypted_identity_private_key,
        identity_key_iv: user.identity_key_iv,
    }))
}

// Upload pre-key bundle
async fn upload_prekey_bundle(
    Path(username): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<UploadPreKeyBundleRequest>,
) -> Result<StatusCode, StatusCode> {
    // Verify user exists
    let _user: User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to get user: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = get_current_time();

    // Serialize arrays to JSON
    let one_time_prekeys = serde_json::to_string(&payload.one_time_prekeys).map_err(|err| {
        tracing::error!("Failed to serialize one_time_prekeys: {:?}", err);
        StatusCode::BAD_REQUEST
    })?;
    let encrypted_one_time_prekey_privates =
        serde_json::to_string(&payload.encrypted_one_time_prekey_privates).map_err(|err| {
            tracing::error!(
                "Failed to serialize encrypted_one_time_prekey_privates: {:?}",
                err
            );
            StatusCode::BAD_REQUEST
        })?;
    let one_time_prekey_ivs =
        serde_json::to_string(&payload.one_time_prekey_ivs).map_err(|err| {
            tracing::error!("Failed to serialize one_time_prekey_ivs: {:?}", err);
            StatusCode::BAD_REQUEST
        })?;

    // Delete existing bundle for this user
    sqlx::query("DELETE FROM prekey_bundles WHERE username = ?")
        .bind(&username)
        .execute(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to delete old prekey bundle: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    sqlx::query(
        r#"
        INSERT INTO prekey_bundles (id, username, signed_prekey_public, signed_prekey_signature, 
            encrypted_signed_prekey_private, signed_prekey_iv, one_time_prekeys, 
            encrypted_one_time_prekey_privates, one_time_prekey_ivs, created_at) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&username)
    .bind(&payload.signed_prekey_public)
    .bind(&payload.signed_prekey_signature)
    .bind(&payload.encrypted_signed_prekey_private)
    .bind(&payload.signed_prekey_iv)
    .bind(&one_time_prekeys)
    .bind(&encrypted_one_time_prekey_privates)
    .bind(&one_time_prekey_ivs)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to upload prekey bundle: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::CREATED)
}

// Fetch pre-key bundle for a user (to initiate key exchange)
async fn get_prekey_bundle(
    Path(username): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<PreKeyBundleResponse>, StatusCode> {
    let user: User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to get user: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let bundle: PreKeyBundle =
        sqlx::query_as("SELECT * FROM prekey_bundles WHERE username = ? ORDER BY created_at DESC LIMIT 1")
            .bind(&username)
            .fetch_optional(&state.db)
            .await
            .map_err(|err| {
                tracing::error!("Failed to get prekey bundle: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::NOT_FOUND)?;

    // Parse one-time prekeys and consume one if available
    let mut one_time_prekeys: Vec<String> =
        serde_json::from_str(&bundle.one_time_prekeys).map_err(|err| {
            tracing::error!("Failed to parse one_time_prekeys: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let one_time_prekey = if !one_time_prekeys.is_empty() {
        let key = one_time_prekeys.remove(0);

        // Also remove from private keys and IVs
        let mut encrypted_privates: Vec<String> =
            serde_json::from_str(&bundle.encrypted_one_time_prekey_privates).map_err(|err| {
                tracing::error!(
                    "Failed to parse encrypted_one_time_prekey_privates: {:?}",
                    err
                );
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        let mut ivs: Vec<String> =
            serde_json::from_str(&bundle.one_time_prekey_ivs).map_err(|err| {
                tracing::error!("Failed to parse one_time_prekey_ivs: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        if !encrypted_privates.is_empty() {
            encrypted_privates.remove(0);
        }
        if !ivs.is_empty() {
            ivs.remove(0);
        }

        // Update bundle
        let new_one_time_prekeys = serde_json::to_string(&one_time_prekeys).map_err(|err| {
            tracing::error!("Failed to serialize one_time_prekeys: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let new_encrypted_privates = serde_json::to_string(&encrypted_privates).map_err(|err| {
            tracing::error!(
                "Failed to serialize encrypted_one_time_prekey_privates: {:?}",
                err
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let new_ivs = serde_json::to_string(&ivs).map_err(|err| {
            tracing::error!("Failed to serialize one_time_prekey_ivs: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        sqlx::query(
            "UPDATE prekey_bundles SET one_time_prekeys = ?, encrypted_one_time_prekey_privates = ?, one_time_prekey_ivs = ? WHERE id = ?",
        )
        .bind(&new_one_time_prekeys)
        .bind(&new_encrypted_privates)
        .bind(&new_ivs)
        .bind(&bundle.id)
        .execute(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update prekey bundle: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Some(key)
    } else {
        None
    };

    Ok(Json(PreKeyBundleResponse {
        identity_public_key: user.identity_public_key,
        signed_prekey_public: bundle.signed_prekey_public,
        signed_prekey_signature: bundle.signed_prekey_signature,
        one_time_prekey,
    }))
}

// Fetch user's own prekey bundle (for logging in and decrypting)
async fn get_my_prekeys(
    Path(username): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<MyPreKeyBundleResponse>, StatusCode> {
    // Verify user exists
    let _user: User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to get user: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let bundle: PreKeyBundle =
        sqlx::query_as("SELECT * FROM prekey_bundles WHERE username = ? ORDER BY created_at DESC LIMIT 1")
            .bind(&username)
            .fetch_optional(&state.db)
            .await
            .map_err(|err| {
                tracing::error!("Failed to get prekey bundle: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(MyPreKeyBundleResponse {
        encrypted_signed_prekey_private: bundle.encrypted_signed_prekey_private,
        signed_prekey_iv: bundle.signed_prekey_iv,
    }))
}

// Send encrypted message (sealed sender)
async fn send_message(
    Path(sender_username): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<SendMessageResponse>), StatusCode> {
    // Verify sender exists
    let _sender: User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&sender_username)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to get sender: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify recipient exists
    let _recipient: User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&payload.recipient_username)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to get recipient: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = get_current_time();

    sqlx::query(
        "INSERT INTO messages (id, sealed_sender_envelope, recipient_username, created_at, delivered) VALUES (?, ?, ?, ?, 0)",
    )
    .bind(&id)
    .bind(&payload.sealed_sender_envelope)
    .bind(&payload.recipient_username)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to store message: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Add to pending messages for long-polling
    {
        let mut pending = state.pending.write().await;
        let messages = pending
            .entry(payload.recipient_username.clone())
            .or_default();
        messages.push(EncryptedMessage {
            id: id.clone(),
            sealed_sender_envelope: payload.sealed_sender_envelope,
            created_at: now,
        });
    }

    Ok((
        StatusCode::CREATED,
        Json(SendMessageResponse {
            id,
            created_at: now,
        }),
    ))
}

// Long-polling for messages
async fn poll_messages(
    Path(username): Path<String>,
    Query(query): Query<PollMessagesQuery>,
    State(state): State<AppState>,
) -> Result<Json<PollMessagesResponse>, StatusCode> {
    // Verify user exists
    let _user: User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to get user: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let timeout_secs = query.timeout_secs.unwrap_or(25);
    let timeout_duration = tokio::time::Duration::from_secs(timeout_secs);
    let start = tokio::time::Instant::now();
    let poll_interval = tokio::time::Duration::from_millis(100);

    // First check for undelivered messages in DB
    let stored_messages: Vec<StoredMessage> = sqlx::query_as(
        "SELECT * FROM messages WHERE recipient_username = ? AND delivered = 0 ORDER BY created_at ASC",
    )
    .bind(&username)
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get messages: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !stored_messages.is_empty() {
        let messages: Vec<EncryptedMessage> = stored_messages
            .into_iter()
            .map(|m| EncryptedMessage {
                id: m.id,
                sealed_sender_envelope: m.sealed_sender_envelope,
                created_at: m.created_at,
            })
            .collect();

        return Ok(Json(PollMessagesResponse {
            messages,
            timestamp: get_current_time(),
        }));
    }

    // Long-poll for new messages
    while start.elapsed() < timeout_duration {
        {
            let mut pending = state.pending.write().await;
            if let Some(messages) = pending.get_mut(&username) {
                if !messages.is_empty() {
                    let result: Vec<EncryptedMessage> = messages.drain(..).collect();
                    return Ok(Json(PollMessagesResponse {
                        messages: result,
                        timestamp: get_current_time(),
                    }));
                }
            }
        }
        tokio::time::sleep(poll_interval).await;
    }

    Ok(Json(PollMessagesResponse {
        messages: vec![],
        timestamp: get_current_time(),
    }))
}

// Acknowledge messages (mark as delivered)
async fn ack_messages(
    Path(username): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<AckMessagesRequest>,
) -> Result<StatusCode, StatusCode> {
    for message_id in &payload.message_ids {
        sqlx::query("UPDATE messages SET delivered = 1 WHERE id = ? AND recipient_username = ?")
            .bind(message_id)
            .bind(&username)
            .execute(&state.db)
            .await
            .map_err(|err| {
                tracing::error!("Failed to ack message: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    Ok(StatusCode::OK)
}

// List users (for finding contacts)
async fn list_users(State(state): State<AppState>) -> Result<Json<Vec<String>>, StatusCode> {
    let users: Vec<(String,)> = sqlx::query_as("SELECT username FROM users ORDER BY username ASC")
        .fetch_all(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to list users: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(users.into_iter().map(|(u,)| u).collect()))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:encrypted_chat.db?mode=rwc".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    init_db(&pool).await.expect("Failed to initialize database");

    let state = AppState {
        db: pool,
        pending: Arc::new(RwLock::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // User management
        .route("/api/register", post(register))
        .route("/api/login", post(login))
        .route("/api/users", get(list_users))
        // Pre-key bundles
        .route("/api/users/{username}/prekeys", post(upload_prekey_bundle))
        .route("/api/users/{username}/prekeys", get(get_prekey_bundle))
        .route("/api/users/{username}/myprekeys", get(get_my_prekeys))
        // Messaging
        .route("/api/users/{username}/messages", post(send_message))
        .route("/api/users/{username}/messages/poll", get(poll_messages))
        .route("/api/users/{username}/messages/ack", post(ack_messages))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4001")
        .await
        .expect("Failed to bind");
    tracing::info!("Encrypted Chat server listening on http://0.0.0.0:4001");
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
