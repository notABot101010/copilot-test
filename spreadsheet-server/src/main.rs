use automerge::transaction::Transactable;
use automerge::ReadDoc;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};

// Channel capacity for broadcasting updates
const BROADCAST_CAPACITY: usize = 100;

// Store for document broadcast channels - one per document for real-time updates
type DocumentChannels = Arc<RwLock<HashMap<String, broadcast::Sender<DocumentUpdate>>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DocumentUpdate {
    /// Base64-encoded Automerge document binary
    document: String,
    /// Client ID that sent this update (to avoid echoing back)
    sender_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SpreadsheetInfo {
    id: String,
    name: String,
    #[serde(rename = "createdAt")]
    created_at: i64,
    #[serde(rename = "updatedAt")]
    updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SyncRequest {
    /// Base64-encoded Automerge document binary
    document: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SyncResponse {
    /// Base64-encoded merged Automerge document binary
    document: String,
    /// Whether the document was updated from the server
    updated: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateRequest {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateResponse {
    id: String,
    name: String,
    #[serde(rename = "createdAt")]
    created_at: i64,
    #[serde(rename = "updatedAt")]
    updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListResponse {
    spreadsheets: Vec<SpreadsheetInfo>,
}

// WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum WsMessage {
    /// Client identifies itself
    #[serde(rename = "identify")]
    Identify { client_id: String },
    /// Client sends document update
    #[serde(rename = "update")]
    Update { document: String },
    /// Server sends document update to clients
    #[serde(rename = "sync")]
    Sync { document: String, sender_id: String },
    /// Server confirms connection
    #[serde(rename = "connected")]
    Connected { document: String },
    /// Error message
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    channels: DocumentChannels,
}

async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS spreadsheets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            document BLOB NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Initialize SQLite database
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:spreadsheet.db?mode=rwc".to_string());
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
        .route(
            "/api/spreadsheets",
            get(list_spreadsheets).post(create_spreadsheet),
        )
        .route(
            "/api/spreadsheets/{id}",
            get(get_spreadsheet).delete(delete_spreadsheet),
        )
        .route("/api/spreadsheets/{id}/sync", post(sync_spreadsheet))
        .route("/ws/spreadsheets/{id}", get(ws_handler))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4001").await.unwrap();
    println!("Spreadsheet server listening on http://0.0.0.0:4001");
    println!("WebSocket endpoint: ws://0.0.0.0:4001/ws/spreadsheets/:id");
    axum::serve(listener, app).await.unwrap();
}

async fn list_spreadsheets(
    State(state): State<AppState>,
) -> Result<Json<ListResponse>, StatusCode> {
    let rows = sqlx::query_as::<_, (String, String, i64, i64)>(
        "SELECT id, name, created_at, updated_at FROM spreadsheets ORDER BY updated_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let spreadsheets: Vec<SpreadsheetInfo> = rows
        .into_iter()
        .map(|(id, name, created_at, updated_at)| SpreadsheetInfo {
            id,
            name,
            created_at,
            updated_at,
        })
        .collect();

    Ok(Json(ListResponse { spreadsheets }))
}

async fn create_spreadsheet(
    State(state): State<AppState>,
    Json(payload): Json<CreateRequest>,
) -> Result<(StatusCode, Json<CreateResponse>), StatusCode> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    // Create initial Automerge document
    let mut doc = automerge::AutoCommit::new();
    doc.put(automerge::ROOT, "id", id.clone()).unwrap();
    doc.put(automerge::ROOT, "name", payload.name.clone())
        .unwrap();
    let cells = doc
        .put_object(automerge::ROOT, "cells", automerge::ObjType::Map)
        .unwrap();
    let _ = cells; // cells is created but empty initially
    doc.put(automerge::ROOT, "createdAt", now).unwrap();
    doc.put(automerge::ROOT, "updatedAt", now).unwrap();

    let binary = doc.save();

    sqlx::query(
        "INSERT INTO spreadsheets (id, name, document, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&binary)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::CREATED,
        Json(CreateResponse {
            id,
            name: payload.name,
            created_at: now,
            updated_at: now,
        }),
    ))
}

async fn get_spreadsheet(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<SyncResponse>, StatusCode> {
    let row: Option<(Vec<u8>,)> = sqlx::query_as("SELECT document FROM spreadsheets WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some((binary,)) => Ok(Json(SyncResponse {
            document: BASE64.encode(&binary),
            updated: false,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn delete_spreadsheet(Path(id): Path<String>, State(state): State<AppState>) -> StatusCode {
    let result = sqlx::query("DELETE FROM spreadsheets WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => {
            // Also clean up the broadcast channel
            let mut channels = state.channels.write().await;
            channels.remove(&id);
            StatusCode::NO_CONTENT
        }
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn sync_spreadsheet(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<SyncRequest>,
) -> Result<Json<SyncResponse>, StatusCode> {
    let client_binary = BASE64
        .decode(&payload.document)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    // Fetch existing document from database
    let row: Option<(Vec<u8>,)> = sqlx::query_as("SELECT document FROM spreadsheets WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // If we have an existing document, merge with the client's version
    let (merged_binary, updated) = if let Some((server_binary,)) = row {
        let mut server_doc = automerge::AutoCommit::load(&server_binary)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let client_doc =
            automerge::AutoCommit::load(&client_binary).map_err(|_| StatusCode::BAD_REQUEST)?;

        // Merge the client's changes into the server document
        server_doc
            .merge(&mut client_doc.clone())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let merged = server_doc.save();
        let updated = merged != client_binary;
        (merged, updated)
    } else {
        // First sync - just store the client's document
        (client_binary, false)
    };

    // Update name from the merged document
    let name = if let Ok(doc) = automerge::AutoCommit::load(&merged_binary) {
        doc.get(automerge::ROOT, "name")
            .ok()
            .flatten()
            .and_then(|(v, _)| v.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| format!("Spreadsheet {}", &id[..8.min(id.len())]))
    } else {
        format!("Spreadsheet {}", &id[..8.min(id.len())])
    };

    // Upsert the document
    sqlx::query(
        r#"
        INSERT INTO spreadsheets (id, name, document, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            document = excluded.document,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&id)
    .bind(&name)
    .bind(&merged_binary)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Broadcast to WebSocket clients
    broadcast_update(&state, &id, &merged_binary, "http-sync").await;

    Ok(Json(SyncResponse {
        document: BASE64.encode(&merged_binary),
        updated,
    }))
}

// WebSocket handler for real-time updates
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, id, state))
}

async fn handle_socket(socket: WebSocket, spreadsheet_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Get or create broadcast channel for this document
    let rx = {
        let mut channels = state.channels.write().await;
        let tx = channels
            .entry(spreadsheet_id.clone())
            .or_insert_with(|| broadcast::channel(BROADCAST_CAPACITY).0);
        tx.subscribe()
    };

    let mut rx = rx;

    // Shared client ID between send and receive tasks
    let client_id: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));

    // Send current document state on connection
    {
        let row: Option<(Vec<u8>,)> =
            sqlx::query_as("SELECT document FROM spreadsheets WHERE id = ?")
                .bind(&spreadsheet_id)
                .fetch_optional(&state.db)
                .await
                .ok()
                .flatten();

        if let Some((binary,)) = row {
            let msg = WsMessage::Connected {
                document: BASE64.encode(&binary),
            };
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = sender.send(Message::Text(json.into())).await;
            }
        }
    }

    // Spawn task to forward broadcast messages to this client
    let client_id_for_broadcast = client_id.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(update) = rx.recv().await {
            // Don't echo back to the sender
            let should_send = {
                let client_id_guard = client_id_for_broadcast.read().await;
                client_id_guard
                    .as_ref()
                    .map_or(true, |id| id != &update.sender_id)
            };

            if should_send {
                let msg = WsMessage::Sync {
                    document: update.document,
                    sender_id: update.sender_id,
                };
                if let Ok(json) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Handle incoming messages
    let state_clone = state.clone();
    let spreadsheet_id_clone2 = spreadsheet_id.clone();
    let client_id_for_recv = client_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        match ws_msg {
                            WsMessage::Identify { client_id: cid } => {
                                // Update the shared client ID
                                let mut client_id_guard = client_id_for_recv.write().await;
                                *client_id_guard = Some(cid);
                            }
                            WsMessage::Update { document } => {
                                // Decode and merge the document
                                if let Ok(client_binary) = BASE64.decode(&document) {
                                    let sender_id = {
                                        let client_id_guard = client_id_for_recv.read().await;
                                        client_id_guard
                                            .clone()
                                            .unwrap_or_else(|| "unknown".to_string())
                                    };

                                    let now = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis()
                                        as i64;

                                    // Fetch existing document from database
                                    let row: Option<(Vec<u8>,)> = sqlx::query_as(
                                        "SELECT document FROM spreadsheets WHERE id = ?",
                                    )
                                    .bind(&spreadsheet_id_clone2)
                                    .fetch_optional(&state_clone.db)
                                    .await
                                    .ok()
                                    .flatten();

                                    // Merge with server document
                                    let merged_binary = if let Some((server_binary,)) = row {
                                        if let Ok(mut server_doc) =
                                            automerge::AutoCommit::load(&server_binary)
                                        {
                                            if let Ok(client_doc) =
                                                automerge::AutoCommit::load(&client_binary)
                                            {
                                                let _ = server_doc.merge(&mut client_doc.clone());
                                                server_doc.save()
                                            } else {
                                                client_binary.clone()
                                            }
                                        } else {
                                            client_binary.clone()
                                        }
                                    } else {
                                        client_binary.clone()
                                    };

                                    // Update name from the merged document
                                    let name = if let Ok(doc) =
                                        automerge::AutoCommit::load(&merged_binary)
                                    {
                                        doc.get(automerge::ROOT, "name")
                                            .ok()
                                            .flatten()
                                            .and_then(|(v, _)| v.to_str().map(|s| s.to_string()))
                                            .unwrap_or_else(|| {
                                                format!(
                                                    "Spreadsheet {}",
                                                    &spreadsheet_id_clone2
                                                        [..8.min(spreadsheet_id_clone2.len())]
                                                )
                                            })
                                    } else {
                                        format!(
                                            "Spreadsheet {}",
                                            &spreadsheet_id_clone2
                                                [..8.min(spreadsheet_id_clone2.len())]
                                        )
                                    };

                                    // Upsert the document to database
                                    let _ = sqlx::query(
                                        r#"
                                        INSERT INTO spreadsheets (id, name, document, created_at, updated_at)
                                        VALUES (?, ?, ?, ?, ?)
                                        ON CONFLICT(id) DO UPDATE SET
                                            name = excluded.name,
                                            document = excluded.document,
                                            updated_at = excluded.updated_at
                                        "#
                                    )
                                    .bind(&spreadsheet_id_clone2)
                                    .bind(&name)
                                    .bind(&merged_binary)
                                    .bind(now)
                                    .bind(now)
                                    .execute(&state_clone.db)
                                    .await;

                                    // Broadcast to other clients
                                    broadcast_update(
                                        &state_clone,
                                        &spreadsheet_id_clone2,
                                        &merged_binary,
                                        &sender_id,
                                    )
                                    .await;
                                }
                            }
                            _ => {}
                        }
                    }
                }
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

    tracing::info!(
        "WebSocket connection closed for spreadsheet {}",
        spreadsheet_id
    );
}

async fn broadcast_update(state: &AppState, spreadsheet_id: &str, binary: &[u8], sender_id: &str) {
    let channels = state.channels.read().await;
    if let Some(tx) = channels.get(spreadsheet_id) {
        let update = DocumentUpdate {
            document: BASE64.encode(binary),
            sender_id: sender_id.to_string(),
        };
        let _ = tx.send(update);
    }
}
