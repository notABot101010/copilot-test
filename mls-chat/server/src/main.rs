use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

mod auth;
mod groups;
mod long_poll;
mod models;

use auth::{hash_password, verify_password};
use long_poll::LongPollManager;
use models::*;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub poll_manager: Arc<RwLock<LongPollManager>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:mls_chat.db?mode=rwc".to_string());

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Run migrations
    sqlx::query(include_str!("../migrations/001_initial.sql"))
        .execute(&db)
        .await
        .ok(); // Ignore errors if tables already exist

    let poll_manager = Arc::new(RwLock::new(LongPollManager::new()));

    let state = AppState { db, poll_manager };

    let app = Router::new()
        // Auth routes
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        // Key package routes
        .route("/api/key-packages", post(upload_key_packages))
        .route("/api/key-packages/:username", get(get_key_package))
        // Group routes
        .route("/api/groups", post(groups::create_group))
        .route("/api/groups", get(groups::list_groups))
        .route("/api/groups/:group_id", get(groups::get_group))
        .route("/api/groups/:group_id/invite", post(groups::invite_member))
        .route("/api/groups/:group_id/join", post(groups::join_group))
        .route("/api/groups/:group_id/message", post(groups::send_message))
        .route("/api/groups/:group_id/messages", get(groups::get_messages))
        // Channel routes
        .route("/api/channels", get(groups::list_channels))
        .route(
            "/api/channels/:group_id/subscribe",
            post(groups::subscribe_channel),
        )
        // Pending data routes
        .route("/api/welcomes", get(get_pending_welcomes))
        .route("/api/welcomes/:welcome_id", delete(delete_welcome))
        // Long polling
        .route("/api/poll", get(poll_updates))
        // User routes
        .route("/api/users", get(list_users))
        .layer(CorsLayer::permissive())
        .with_state(state);

    info!("Starting MLS Chat server on http://0.0.0.0:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let password_hash = hash_password(&req.password)?;

    let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&req.username)
        .bind(&password_hash)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) => {
            let user_id = res.last_insert_rowid();
            info!("User registered: {} (id={})", req.username, user_id);
            Ok(Json(AuthResponse {
                success: true,
                user_id: Some(user_id),
                username: req.username,
                error: None,
            }))
        }
        Err(err) => {
            warn!("Registration failed: {}", err);
            Err(AppError::Conflict("Username already exists".to_string()))
        }
    }
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await?;

    match user {
        Some(user) => {
            if verify_password(&req.password, &user.password_hash)? {
                info!("User logged in: {}", req.username);
                Ok(Json(AuthResponse {
                    success: true,
                    user_id: Some(user.id),
                    username: user.username,
                    error: None,
                }))
            } else {
                Err(AppError::Unauthorized("Invalid credentials".to_string()))
            }
        }
        None => Err(AppError::Unauthorized("Invalid credentials".to_string())),
    }
}

async fn upload_key_packages(
    State(state): State<AppState>,
    Json(req): Json<UploadKeyPackagesRequest>,
) -> Result<Json<GenericResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    for kp in &req.key_packages {
        let key_package_data =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, kp)
                .map_err(|_| AppError::BadRequest("Invalid base64".to_string()))?;

        // Use a hash of the key package as unique identifier
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        key_package_data.hash(&mut hasher);
        let hash = hasher.finish().to_le_bytes();

        sqlx::query(
            "INSERT OR IGNORE INTO key_packages (user_id, key_package_data, key_package_hash) VALUES (?, ?, ?)",
        )
        .bind(user.id)
        .bind(&key_package_data)
        .bind(&hash[..])
        .execute(&state.db)
        .await?;
    }

    info!(
        "Uploaded {} key packages for user {}",
        req.key_packages.len(),
        req.username
    );
    Ok(Json(GenericResponse { success: true }))
}

async fn get_key_package(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<KeyPackageResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let kp = sqlx::query_as::<_, KeyPackageRow>(
        "SELECT id, user_id, key_package_data, key_package_hash, used FROM key_packages WHERE user_id = ? AND used = 0 LIMIT 1",
    )
    .bind(user.id)
    .fetch_optional(&state.db)
    .await?;

    match kp {
        Some(kp) => {
            // Mark as used
            sqlx::query("UPDATE key_packages SET used = 1 WHERE id = ?")
                .bind(kp.id)
                .execute(&state.db)
                .await?;

            let encoded = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &kp.key_package_data,
            );

            Ok(Json(KeyPackageResponse {
                key_package: encoded,
            }))
        }
        None => Err(AppError::NotFound("No key packages available".to_string())),
    }
}

async fn get_pending_welcomes(
    State(state): State<AppState>,
    Query(params): Query<UsernameQuery>,
) -> Result<Json<Vec<PendingWelcome>>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&params.username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let welcomes = sqlx::query_as::<_, PendingWelcomeRow>(
        "SELECT pw.id, pw.user_id, pw.group_id, pw.welcome_data, pw.group_info_data, pw.inviter_id, pw.created_at, g.name as group_name, u.username as inviter_name
         FROM pending_welcomes pw
         JOIN groups g ON g.group_id = pw.group_id
         JOIN users u ON u.id = pw.inviter_id
         WHERE pw.user_id = ?",
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;

    let result: Vec<PendingWelcome> = welcomes
        .into_iter()
        .map(|w| PendingWelcome {
            id: w.id,
            group_id: w.group_id,
            group_name: w.group_name,
            welcome_data: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &w.welcome_data,
            ),
            group_info_data: w
                .group_info_data
                .map(|d| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &d)),
            inviter_name: w.inviter_name,
        })
        .collect();

    Ok(Json(result))
}

async fn delete_welcome(
    State(state): State<AppState>,
    Path(welcome_id): Path<i64>,
) -> Result<Json<GenericResponse>, AppError> {
    sqlx::query("DELETE FROM pending_welcomes WHERE id = ?")
        .bind(welcome_id)
        .execute(&state.db)
        .await?;

    Ok(Json(GenericResponse { success: true }))
}

async fn poll_updates(
    State(state): State<AppState>,
    Query(params): Query<PollParams>,
) -> Result<Json<PollResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&params.username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Set up long-polling waiter
    let mut poll_manager = state.poll_manager.write().await;
    let waiter = poll_manager.add_waiter(&params.username);
    drop(poll_manager);

    // Wait for notification or timeout
    tokio::select! {
        _ = waiter.notified() => {},
        _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {},
    }

    // Check for new welcomes
    let welcomes = sqlx::query_as::<_, PendingWelcomeRow>(
        "SELECT pw.id, pw.user_id, pw.group_id, pw.welcome_data, pw.group_info_data, pw.inviter_id, pw.created_at, g.name as group_name, u.username as inviter_name
         FROM pending_welcomes pw
         JOIN groups g ON g.group_id = pw.group_id
         JOIN users u ON u.id = pw.inviter_id
         WHERE pw.user_id = ?",
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;

    // Check for new messages
    let pending = sqlx::query_as::<_, PendingMessageRow>(
        r#"SELECT pm.id, pm.user_id, pm.group_id, pm.message_id, pm.delivered, pm.created_at,
                  m.message_type, m.message_data, m.sender_id, u.username as sender_name
           FROM pending_messages pm
           JOIN mls_messages m ON m.id = pm.message_id
           LEFT JOIN users u ON u.id = m.sender_id
           WHERE pm.user_id = ? AND pm.delivered = 0"#,
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;

    // Mark messages as delivered
    for msg in &pending {
        sqlx::query("UPDATE pending_messages SET delivered = 1 WHERE id = ?")
            .bind(msg.id)
            .execute(&state.db)
            .await?;
    }

    let welcome_list: Vec<PendingWelcome> = welcomes
        .into_iter()
        .map(|w| PendingWelcome {
            id: w.id,
            group_id: w.group_id,
            group_name: w.group_name,
            welcome_data: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &w.welcome_data,
            ),
            group_info_data: w
                .group_info_data
                .map(|d| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &d)),
            inviter_name: w.inviter_name,
        })
        .collect();

    let message_list: Vec<PendingMessage> = pending
        .into_iter()
        .map(|m| PendingMessage {
            id: m.id,
            group_id: m.group_id,
            message_type: m.message_type,
            message_data: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &m.message_data,
            ),
            sender_name: m.sender_name,
        })
        .collect();

    Ok(Json(PollResponse {
        welcomes: welcome_list,
        messages: message_list,
    }))
}

async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListUsersQuery>,
) -> Result<Json<Vec<UserInfo>>, AppError> {
    let users = sqlx::query_as::<_, UserInfo>(
        "SELECT id, username FROM users WHERE username != ? ORDER BY username",
    )
    .bind(&params.exclude)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(users))
}

#[derive(Debug)]
pub enum AppError {
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
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => {
                warn!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
