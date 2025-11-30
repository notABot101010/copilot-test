use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    http::{StatusCode, Method},
    response::{IntoResponse, Response},
    routing::{get, post, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, Row};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{CorsLayer, Any};
use tracing::{info, error};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<Vec<u8>>>>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    id: String,
    title: String,
    doc_type: String,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Deserialize)]
struct CreateDocumentRequest {
    title: String,
    doc_type: String,
}

#[derive(Debug, Deserialize)]
struct UpdateRequest {
    changes: Vec<u8>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let database_url = "sqlite:./offline-office.db?mode=rwc";
    let pool = SqlitePool::connect(database_url).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            doc_type TEXT NOT NULL,
            automerge_data BLOB NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    info!("Database initialized");

    let state = AppState {
        db: pool,
        channels: Arc::new(RwLock::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/documents", get(list_documents))
        .route("/api/documents", post(create_document))
        .route("/api/documents/{id}", get(get_document))
        .route("/api/documents/{id}", delete(delete_document))
        .route("/api/documents/{id}/sync", post(update_document))
        .route("/api/documents/{id}/ws", get(ws_handler))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("Server listening on http://0.0.0.0:8080");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn list_documents(
    State(state): State<AppState>,
) -> Result<Json<Vec<Document>>, AppError> {
    let documents = sqlx::query(
        "SELECT id, title, doc_type, created_at, updated_at FROM documents ORDER BY updated_at DESC"
    )
    .fetch_all(&state.db)
    .await?
    .iter()
    .map(|row| Document {
        id: row.get("id"),
        title: row.get("title"),
        doc_type: row.get("doc_type"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
    .collect();

    Ok(Json(documents))
}

async fn create_document(
    State(state): State<AppState>,
    Json(payload): Json<CreateDocumentRequest>,
) -> Result<Json<Document>, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    let mut automerge_doc = automerge::AutoCommit::new();
    let automerge_data = automerge_doc.save();

    sqlx::query(
        "INSERT INTO documents (id, title, doc_type, automerge_data, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&payload.title)
    .bind(&payload.doc_type)
    .bind(&automerge_data)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await?;

    Ok(Json(Document {
        id,
        title: payload.title,
        doc_type: payload.doc_type,
        created_at: now,
        updated_at: now,
    }))
}

async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let row = sqlx::query("SELECT automerge_data FROM documents WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await?;

    match row {
        Some(row) => {
            let data: Vec<u8> = row.get("automerge_data");
            Ok((StatusCode::OK, data).into_response())
        }
        None => Ok((StatusCode::NOT_FOUND, "Document not found").into_response()),
    }
}

async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM documents WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn update_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateRequest>,
) -> Result<StatusCode, AppError> {
    let row = sqlx::query("SELECT automerge_data FROM documents WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await?;

    let existing_data: Vec<u8> = row
        .ok_or_else(|| AppError::NotFound)?
        .get("automerge_data");

    let mut doc = automerge::AutoCommit::load(&existing_data)?;
    doc.load_incremental(&payload.changes)?;

    let updated_data = doc.save();
    let now = chrono::Utc::now().timestamp();

    sqlx::query("UPDATE documents SET automerge_data = ?, updated_at = ? WHERE id = ?")
        .bind(&updated_data)
        .bind(now)
        .bind(&id)
        .execute(&state.db)
        .await?;

    let channels = state.channels.read().await;
    if let Some(tx) = channels.get(&id) {
        let _ = tx.send(payload.changes);
    }

    Ok(StatusCode::OK)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, id))
}

async fn handle_socket(socket: WebSocket, state: AppState, doc_id: String) {
    let (mut sender, mut receiver) = socket.split();

    let tx = {
        let mut channels = state.channels.write().await;
        channels
            .entry(doc_id.clone())
            .or_insert_with(|| broadcast::channel(100).0)
            .clone()
    };

    let mut rx = tx.subscribe();

    let doc_id_clone = doc_id.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender
                .send(Message::Binary(msg.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let tx_clone = tx.clone();
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Binary(data))) = receiver.next().await {
            let row = sqlx::query("SELECT automerge_data FROM documents WHERE id = ?")
                .bind(&doc_id_clone)
                .fetch_optional(&state_clone.db)
                .await;

            if let Ok(Some(row)) = row {
                let existing_data: Vec<u8> = row.get("automerge_data");

                if let Ok(mut doc) = automerge::AutoCommit::load(&existing_data) {
                    if doc.load_incremental(&data.to_vec()).is_ok() {
                        let updated_data = doc.save();
                        let now = chrono::Utc::now().timestamp();

                        let _ = sqlx::query("UPDATE documents SET automerge_data = ?, updated_at = ? WHERE id = ?")
                            .bind(&updated_data)
                            .bind(now)
                            .bind(&doc_id_clone)
                            .execute(&state_clone.db)
                            .await;

                        let _ = tx_clone.send(data.to_vec());
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };

    let mut channels = state.channels.write().await;
    if let Some(channel) = channels.get(&doc_id) {
        if channel.receiver_count() == 0 {
            channels.remove(&doc_id);
        }
    }
}

#[derive(Debug)]
enum AppError {
    Database(sqlx::Error),
    Automerge(automerge::AutomergeError),
    NotFound,
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err)
    }
}

impl From<automerge::AutomergeError> for AppError {
    fn from(err: automerge::AutomergeError) -> Self {
        AppError::Automerge(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(err) => {
                error!("Database error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
            AppError::Automerge(err) => {
                error!("Automerge error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "CRDT error")
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
        };
        (status, message).into_response()
    }
}

use futures_util::{SinkExt, StreamExt};
