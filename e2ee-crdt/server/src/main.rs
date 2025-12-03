use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, FromRow};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Document {
    id: String,
    encrypted_data: String,
    created_at: i64,
}

#[derive(Debug, Clone, FromRow)]
struct DbDocument {
    id: String,
    encrypted_data: String,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DocumentOperation {
    document_id: String,
    encrypted_operation: String,
    timestamp: i64,
}

#[derive(Debug, Clone, FromRow)]
struct DbOperation {
    id: i64,
    document_id: String,
    encrypted_operation: String,
    timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateDocumentRequest {
    encrypted_data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateDocumentResponse {
    id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpdateDocumentRequest {
    encrypted_data: String,
    encrypted_operation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum WsMessage {
    #[serde(rename = "operation")]
    Operation {
        document_id: String,
        encrypted_operation: String,
        timestamp: i64,
    },
    #[serde(rename = "subscribe")]
    Subscribe { document_id: String },
    #[serde(rename = "document_created")]
    DocumentCreated {
        id: String,
        encrypted_data: String,
        created_at: i64,
    },
}

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    broadcast_tx: broadcast::Sender<WsMessage>,
    subscribers: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "e2ee_crdt_server=debug,tower_http=debug".into()),
        )
        .init();

    let db = SqlitePool::connect("sqlite:crdt.db?mode=rwc").await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            encrypted_data TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(&db)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS operations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            document_id TEXT NOT NULL,
            encrypted_operation TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            FOREIGN KEY (document_id) REFERENCES documents(id)
        )
        "#,
    )
    .execute(&db)
    .await?;

    let (broadcast_tx, _) = broadcast::channel(100);

    let state = AppState {
        db,
        broadcast_tx,
        subscribers: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/api/documents", get(list_documents).post(create_document))
        .route("/api/documents/:id", get(get_document).put(update_document))
        .route("/api/documents/:id/operations", get(get_operations))
        .route("/ws", get(ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;

    info!("Server listening on http://0.0.0.0:3001");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn list_documents(State(state): State<AppState>) -> Result<Json<Vec<Document>>, AppError> {
    let docs = sqlx::query_as::<_, DbDocument>(
        "SELECT id, encrypted_data, created_at FROM documents ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    let documents = docs
        .into_iter()
        .map(|d| Document {
            id: d.id,
            encrypted_data: d.encrypted_data,
            created_at: d.created_at,
        })
        .collect();

    Ok(Json(documents))
}

async fn create_document(
    State(state): State<AppState>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<CreateDocumentResponse>, AppError> {
    let id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp();

    sqlx::query("INSERT INTO documents (id, encrypted_data, created_at) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&req.encrypted_data)
        .bind(timestamp)
        .execute(&state.db)
        .await?;

    info!("Created document: {}", id);

    // Broadcast document creation to all connected clients
    let _ = state.broadcast_tx.send(WsMessage::DocumentCreated {
        id: id.clone(),
        encrypted_data: req.encrypted_data.clone(),
        created_at: timestamp,
    });

    Ok(Json(CreateDocumentResponse { id }))
}

async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Document>, AppError> {
    let doc = sqlx::query_as::<_, DbDocument>(
        "SELECT id, encrypted_data, created_at FROM documents WHERE id = ?",
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(Document {
        id: doc.id,
        encrypted_data: doc.encrypted_data,
        created_at: doc.created_at,
    }))
}

async fn update_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDocumentRequest>,
) -> Result<StatusCode, AppError> {
    let timestamp = chrono::Utc::now().timestamp();

    info!("Updating document: {}", id);

    sqlx::query("UPDATE documents SET encrypted_data = ? WHERE id = ?")
        .bind(&req.encrypted_data)
        .bind(&id)
        .execute(&state.db)
        .await?;

    sqlx::query(
        "INSERT INTO operations (document_id, encrypted_operation, timestamp) VALUES (?, ?, ?)",
    )
    .bind(&id)
    .bind(&req.encrypted_operation)
    .bind(timestamp)
    .execute(&state.db)
    .await?;

    let msg = WsMessage::Operation {
        document_id: id.clone(),
        encrypted_operation: req.encrypted_operation.clone(),
        timestamp,
    };

    match state.broadcast_tx.send(msg) {
        Ok(receiver_count) => {
            info!(
                "Broadcasted operation for document {} to {} receivers",
                id, receiver_count
            );
        }
        Err(err) => {
            error!("Failed to broadcast operation for document {}: {}", id, err);
        }
    }

    Ok(StatusCode::OK)
}

async fn get_operations(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<DocumentOperation>>, AppError> {
    let ops = sqlx::query_as::<_, DbOperation>(
        "SELECT id, document_id, encrypted_operation, timestamp FROM operations WHERE document_id = ? ORDER BY timestamp ASC"
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await?;

    let operations = ops
        .into_iter()
        .map(|op| DocumentOperation {
            document_id: op.document_id,
            encrypted_operation: op.encrypted_operation,
            timestamp: op.timestamp,
        })
        .collect();

    Ok(Json(operations))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    info!("New WebSocket connection established");
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.broadcast_tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let msg_type = match &msg {
                WsMessage::Operation { document_id, .. } => format!("Operation({})", document_id),
                WsMessage::DocumentCreated { id, .. } => format!("DocumentCreated({})", id),
                WsMessage::Subscribe { document_id } => format!("Subscribe({})", document_id),
            };
            info!("Sending WebSocket message: {}", msg_type);

            let json = match serde_json::to_string(&msg) {
                Ok(j) => j,
                Err(err) => {
                    error!("Failed to serialize message: {}", err);
                    continue;
                }
            };

            if sender.send(Message::Text(json)).await.is_err() {
                info!("WebSocket client disconnected");
                break;
            }
        }
    });

    let broadcast_tx = state.broadcast_tx.clone();
    let db = state.db.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            info!("Received WebSocket message from client");
            if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                match msg {
                    WsMessage::Operation {
                        document_id,
                        encrypted_operation,
                        timestamp,
                    } => {
                        info!(
                            "Received operation via WebSocket for document: {}",
                            document_id
                        );
                        if let Err(err) = sqlx::query(
                            "INSERT INTO operations (document_id, encrypted_operation, timestamp) VALUES (?, ?, ?)"
                        )
                        .bind(&document_id)
                        .bind(&encrypted_operation)
                        .bind(timestamp)
                        .execute(&db)
                        .await {
                            error!("Failed to insert operation: {}", err);
                        }

                        match broadcast_tx.send(WsMessage::Operation {
                            document_id: document_id.clone(),
                            encrypted_operation,
                            timestamp,
                        }) {
                            Ok(count) => info!(
                                "Broadcasted WebSocket operation for {} to {} receivers",
                                document_id, count
                            ),
                            Err(err) => error!("Failed to broadcast WebSocket operation: {}", err),
                        }
                    }
                    WsMessage::Subscribe { document_id } => {
                        info!("Client subscribed to document: {}", document_id);
                    }
                    WsMessage::DocumentCreated { .. } => {
                        // DocumentCreated messages are only sent by the server, not received from clients
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("Application error: {}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal error: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
