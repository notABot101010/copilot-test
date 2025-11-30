use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

mod auth;
mod long_poll;
mod models;

use auth::{hash_password, verify_password};
use long_poll::LongPollManager;
use models::*;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    poll_manager: Arc<RwLock<LongPollManager>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:chat.db".to_string());
    let db = SqlitePool::connect(&db_url).await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let poll_manager = Arc::new(RwLock::new(LongPollManager::new()));

    let state = AppState {
        db,
        poll_manager,
    };

    let app = Router::new()
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/messages/send", post(send_message))
        .route("/api/messages/poll", get(poll_messages))
        .route("/api/messages/:username", get(get_messages))
        .route("/api/users/:username/keys", get(get_user_keys))
        .route("/api/prekeys", post(upload_prekeys))
        .route("/api/prekeys/:username", get(get_prekey))
        .route("/api/ratchet", post(save_ratchet_state))
        .route("/api/ratchet/:peer", get(get_ratchet_state))
        .layer(CorsLayer::permissive())
        .with_state(state);

    info!("Starting server on http://0.0.0.0:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    let password_hash = hash_password(&req.password)?;

    let result = sqlx::query(
        r#"
        INSERT INTO users (username, password_hash, encrypted_identity_key, identity_public_key, prekey_signature)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(&req.username)
    .bind(&password_hash)
    .bind(&req.encrypted_identity_key)
    .bind(&req.identity_public_key)
    .bind(&req.prekey_signature)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Ok(Json(RegisterResponse {
            success: true,
            username: req.username,
        })),
        Err(err) => {
            warn!("Registration failed: {}", err);
            Err(AppError::Conflict("Username already exists".to_string()))
        }
    }
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, encrypted_identity_key, identity_public_key, prekey_signature FROM users WHERE username = ?"
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await?;

    match user {
        Some(user) => {
            if verify_password(&req.password, &user.password_hash)? {
                Ok(Json(LoginResponse {
                    success: true,
                    encrypted_identity_key: user.encrypted_identity_key,
                    identity_public_key: user.identity_public_key,
                }))
            } else {
                Err(AppError::Unauthorized("Invalid credentials".to_string()))
            }
        }
        None => Err(AppError::Unauthorized("Invalid credentials".to_string())),
    }
}

async fn send_message(
    State(state): State<AppState>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<SendMessageResponse>, AppError> {
    let result = sqlx::query(
        r#"
        INSERT INTO messages (from_user, to_user, encrypted_content, ephemeral_public_key,
                             sender_identity_key, sender_signature, message_number, previous_chain_length)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&req.from_user)
    .bind(&req.to_user)
    .bind(&req.encrypted_content)
    .bind(&req.ephemeral_public_key)
    .bind(&req.sender_identity_key)
    .bind(&req.sender_signature)
    .bind(req.message_number)
    .bind(req.previous_chain_length)
    .execute(&state.db)
    .await?;

    let message_id = result.last_insert_rowid();

    let mut poll_manager = state.poll_manager.write().await;
    poll_manager.notify(&req.to_user);

    Ok(Json(SendMessageResponse {
        success: true,
        message_id,
    }))
}

async fn poll_messages(
    State(state): State<AppState>,
    Query(params): Query<PollParams>,
) -> Result<Json<Vec<Message>>, AppError> {
    let mut poll_manager = state.poll_manager.write().await;
    let waiter = poll_manager.add_waiter(&params.username);
    drop(poll_manager);

    tokio::select! {
        _ = waiter.notified() => {},
        _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {},
    }

    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT id, from_user, to_user, encrypted_content, ephemeral_public_key,
               sender_identity_key, sender_signature, message_number, previous_chain_length, created_at
        FROM messages
        WHERE to_user = ? AND delivered_at IS NULL
        ORDER BY created_at ASC
        "#,
    )
    .bind(&params.username)
    .fetch_all(&state.db)
    .await?;

    if !messages.is_empty() {
        let message_ids: Vec<i64> = messages.iter().map(|m| m.id).collect();
        for id in message_ids {
            sqlx::query("UPDATE messages SET delivered_at = CURRENT_TIMESTAMP WHERE id = ?")
                .bind(id)
                .execute(&state.db)
                .await?;
        }
    }

    Ok(Json(messages))
}

async fn get_messages(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(params): Query<GetMessagesParams>,
) -> Result<Json<Vec<Message>>, AppError> {
    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT id, from_user, to_user, encrypted_content, ephemeral_public_key,
               sender_identity_key, sender_signature, message_number, previous_chain_length, created_at
        FROM messages
        WHERE (from_user = ? AND to_user = ?) OR (from_user = ? AND to_user = ?)
        ORDER BY created_at ASC
        "#,
    )
    .bind(&params.current_user)
    .bind(&username)
    .bind(&username)
    .bind(&params.current_user)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(messages))
}

async fn get_user_keys(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<UserKeysResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, encrypted_identity_key, identity_public_key, prekey_signature FROM users WHERE username = ?"
    )
    .bind(&username)
    .fetch_optional(&state.db)
    .await?;

    match user {
        Some(user) => Ok(Json(UserKeysResponse {
            identity_public_key: user.identity_public_key,
            prekey_signature: user.prekey_signature,
        })),
        None => Err(AppError::NotFound("User not found".to_string())),
    }
}

async fn upload_prekeys(
    State(state): State<AppState>,
    Json(req): Json<UploadPrekeysRequest>,
) -> Result<Json<GenericResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, encrypted_identity_key, identity_public_key, prekey_signature FROM users WHERE username = ?"
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    for prekey in req.prekeys {
        sqlx::query(
            "INSERT INTO prekeys (user_id, public_key, key_id) VALUES (?, ?, ?)"
        )
        .bind(user.id)
        .bind(&prekey.public_key)
        .bind(prekey.key_id)
        .execute(&state.db)
        .await?;
    }

    Ok(Json(GenericResponse { success: true }))
}

async fn get_prekey(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<PrekeyResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, encrypted_identity_key, identity_public_key, prekey_signature FROM users WHERE username = ?"
    )
    .bind(&username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let prekey = sqlx::query_as::<_, Prekey>(
        "SELECT id, user_id, public_key, key_id, used FROM prekeys WHERE user_id = ? AND used = 0 LIMIT 1"
    )
    .bind(user.id)
    .fetch_optional(&state.db)
    .await?;

    match prekey {
        Some(prekey) => {
            sqlx::query("UPDATE prekeys SET used = 1 WHERE id = ?")
                .bind(prekey.id)
                .execute(&state.db)
                .await?;

            Ok(Json(PrekeyResponse {
                public_key: prekey.public_key,
                key_id: prekey.key_id,
            }))
        }
        None => Err(AppError::NotFound("No prekeys available".to_string())),
    }
}

async fn save_ratchet_state(
    State(state): State<AppState>,
    Json(req): Json<SaveRatchetStateRequest>,
) -> Result<Json<GenericResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, encrypted_identity_key, identity_public_key, prekey_signature FROM users WHERE username = ?"
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO ratchet_states
        (user_id, peer_username, root_key, chain_key_send, chain_key_receive,
         sending_chain_length, receiving_chain_length, previous_sending_chain_length,
         public_key_send, public_key_receive)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(user_id, peer_username) DO UPDATE SET
            root_key = excluded.root_key,
            chain_key_send = excluded.chain_key_send,
            chain_key_receive = excluded.chain_key_receive,
            sending_chain_length = excluded.sending_chain_length,
            receiving_chain_length = excluded.receiving_chain_length,
            previous_sending_chain_length = excluded.previous_sending_chain_length,
            public_key_send = excluded.public_key_send,
            public_key_receive = excluded.public_key_receive,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(user.id)
    .bind(&req.peer_username)
    .bind(&req.root_key)
    .bind(&req.chain_key_send)
    .bind(&req.chain_key_receive)
    .bind(req.sending_chain_length)
    .bind(req.receiving_chain_length)
    .bind(req.previous_sending_chain_length)
    .bind(&req.public_key_send)
    .bind(&req.public_key_receive)
    .execute(&state.db)
    .await?;

    Ok(Json(GenericResponse { success: true }))
}

async fn get_ratchet_state(
    State(state): State<AppState>,
    Path(peer): Path<String>,
    Query(params): Query<GetRatchetParams>,
) -> Result<Json<Option<RatchetState>>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, encrypted_identity_key, identity_public_key, prekey_signature FROM users WHERE username = ?"
    )
    .bind(&params.username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let ratchet = sqlx::query_as::<_, RatchetState>(
        r#"
        SELECT user_id, peer_username, root_key, chain_key_send, chain_key_receive,
               sending_chain_length, receiving_chain_length, previous_sending_chain_length,
               public_key_send, public_key_receive
        FROM ratchet_states
        WHERE user_id = ? AND peer_username = ?
        "#,
    )
    .bind(user.id)
    .bind(&peer)
    .fetch_optional(&state.db)
    .await?;

    Ok(Json(ratchet))
}

#[derive(Debug)]
enum AppError {
    Database(sqlx::Error),
    Unauthorized(String),
    NotFound(String),
    Conflict(String),
    BadRequest(String),
    Internal(String),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err)
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(err: argon2::password_hash::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(err) => {
                warn!("Database error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => {
                warn!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
