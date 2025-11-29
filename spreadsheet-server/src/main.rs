use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use automerge::transaction::Transactable;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tower_http::cors::{Any, CorsLayer};

// In-memory store for spreadsheet documents
type SpreadsheetStore = Arc<RwLock<HashMap<String, Vec<u8>>>>;

#[derive(Debug, Serialize, Deserialize)]
struct SpreadsheetInfo {
    id: String,
    name: String,
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
}

#[derive(Debug, Serialize, Deserialize)]
struct ListResponse {
    spreadsheets: Vec<SpreadsheetInfo>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let store: SpreadsheetStore = Arc::new(RwLock::new(HashMap::new()));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/spreadsheets", get(list_spreadsheets).post(create_spreadsheet))
        .route("/api/spreadsheets/{id}", get(get_spreadsheet).delete(delete_spreadsheet))
        .route("/api/spreadsheets/{id}/sync", post(sync_spreadsheet))
        .layer(cors)
        .with_state(store);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("Spreadsheet server listening on http://0.0.0.0:3001");
    axum::serve(listener, app).await.unwrap();
}

async fn list_spreadsheets(
    State(store): State<SpreadsheetStore>,
) -> Json<ListResponse> {
    let store = store.read().unwrap();
    let spreadsheets: Vec<SpreadsheetInfo> = store
        .keys()
        .map(|id| SpreadsheetInfo {
            id: id.clone(),
            name: format!("Spreadsheet {}", &id[..8]),
        })
        .collect();
    
    Json(ListResponse { spreadsheets })
}

async fn create_spreadsheet(
    State(store): State<SpreadsheetStore>,
    Json(payload): Json<CreateRequest>,
) -> (StatusCode, Json<CreateResponse>) {
    let id = uuid::Uuid::new_v4().to_string();
    
    // Create initial Automerge document
    let mut doc = automerge::AutoCommit::new();
    doc.put(automerge::ROOT, "id", id.clone()).unwrap();
    doc.put(automerge::ROOT, "name", payload.name.clone()).unwrap();
    let cells = doc.put_object(automerge::ROOT, "cells", automerge::ObjType::Map).unwrap();
    let _ = cells; // cells is created but empty initially
    
    let binary = doc.save();
    
    let mut store = store.write().unwrap();
    store.insert(id.clone(), binary);
    
    (
        StatusCode::CREATED,
        Json(CreateResponse {
            id,
            name: payload.name,
        }),
    )
}

async fn get_spreadsheet(
    Path(id): Path<String>,
    State(store): State<SpreadsheetStore>,
) -> Result<Json<SyncResponse>, StatusCode> {
    let store = store.read().unwrap();
    
    match store.get(&id) {
        Some(binary) => Ok(Json(SyncResponse {
            document: BASE64.encode(binary),
            updated: false,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn delete_spreadsheet(
    Path(id): Path<String>,
    State(store): State<SpreadsheetStore>,
) -> StatusCode {
    let mut store = store.write().unwrap();
    
    match store.remove(&id) {
        Some(_) => StatusCode::NO_CONTENT,
        None => StatusCode::NOT_FOUND,
    }
}

async fn sync_spreadsheet(
    Path(id): Path<String>,
    State(store): State<SpreadsheetStore>,
    Json(payload): Json<SyncRequest>,
) -> Result<Json<SyncResponse>, StatusCode> {
    let client_binary = BASE64.decode(&payload.document).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let mut store = store.write().unwrap();
    
    // If we have an existing document, merge with the client's version
    let (merged_binary, updated) = if let Some(server_binary) = store.get(&id) {
        let mut server_doc = automerge::AutoCommit::load(server_binary)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let client_doc = automerge::AutoCommit::load(&client_binary)
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        
        // Merge the client's changes into the server document
        server_doc.merge(&mut client_doc.clone()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        let merged = server_doc.save();
        let updated = merged != client_binary;
        (merged, updated)
    } else {
        // First sync - just store the client's document
        (client_binary, false)
    };
    
    store.insert(id, merged_binary.clone());
    
    Ok(Json(SyncResponse {
        document: BASE64.encode(&merged_binary),
        updated,
    }))
}
