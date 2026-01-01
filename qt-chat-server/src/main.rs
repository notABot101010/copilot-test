use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    rooms: Arc<RwLock<HashMap<String, Room>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Room {
    id: String,
    name: String,
    members: Vec<String>,
    messages: Vec<Message>,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    id: String,
    room_id: String,
    sender: String,
    content: String,
    timestamp: i64,
}

#[derive(Debug, Deserialize)]
struct CreateRoomRequest {
    name: String,
    creator: String,
}

#[derive(Debug, Serialize)]
struct CreateRoomResponse {
    room_id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct JoinRoomRequest {
    username: String,
}

#[derive(Debug, Serialize)]
struct JoinRoomResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Deserialize)]
struct SendMessageRequest {
    sender: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct SendMessageResponse {
    message_id: String,
    timestamp: i64,
}

#[derive(Debug, Serialize)]
struct GetMessagesResponse {
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct GetRoomsResponse {
    rooms: Vec<RoomInfo>,
}

#[derive(Debug, Clone, Serialize)]
struct RoomInfo {
    id: String,
    name: String,
    member_count: usize,
}

async fn create_room(
    State(state): State<AppState>,
    Json(req): Json<CreateRoomRequest>,
) -> Result<Json<CreateRoomResponse>, StatusCode> {
    let room_id = Uuid::new_v4().to_string();
    let room = Room {
        id: room_id.clone(),
        name: req.name.clone(),
        members: vec![req.creator],
        messages: Vec::new(),
        created_at: Utc::now().timestamp(),
    };

    let mut rooms = state.rooms.write().await;
    rooms.insert(room_id.clone(), room);

    info!("Created room: {} ({})", req.name, room_id);

    Ok(Json(CreateRoomResponse {
        room_id,
        name: req.name,
    }))
}

async fn join_room(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
    Json(req): Json<JoinRoomRequest>,
) -> Result<Json<JoinRoomResponse>, StatusCode> {
    let mut rooms = state.rooms.write().await;
    
    if let Some(room) = rooms.get_mut(&room_id) {
        if !room.members.contains(&req.username) {
            room.members.push(req.username.clone());
            info!("User {} joined room {}", req.username, room_id);
            Ok(Json(JoinRoomResponse {
                success: true,
                message: format!("Joined room {}", room.name),
            }))
        } else {
            Ok(Json(JoinRoomResponse {
                success: true,
                message: "Already a member".to_string(),
            }))
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn send_message(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<SendMessageResponse>, StatusCode> {
    let mut rooms = state.rooms.write().await;
    
    if let Some(room) = rooms.get_mut(&room_id) {
        if !room.members.contains(&req.sender) {
            return Err(StatusCode::FORBIDDEN);
        }

        let message_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().timestamp();
        
        let message = Message {
            id: message_id.clone(),
            room_id: room_id.clone(),
            sender: req.sender.clone(),
            content: req.content,
            timestamp,
        };

        room.messages.push(message);
        info!("Message sent in room {} by {}", room_id, req.sender);

        Ok(Json(SendMessageResponse {
            message_id,
            timestamp,
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_messages(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
) -> Result<Json<GetMessagesResponse>, StatusCode> {
    let rooms = state.rooms.read().await;
    
    if let Some(room) = rooms.get(&room_id) {
        Ok(Json(GetMessagesResponse {
            messages: room.messages.clone(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_rooms(
    State(state): State<AppState>,
) -> Result<Json<GetRoomsResponse>, StatusCode> {
    let rooms = state.rooms.read().await;
    
    let room_list: Vec<RoomInfo> = rooms.values()
        .map(|room| RoomInfo {
            id: room.id.clone(),
            name: room.name.clone(),
            member_count: room.members.len(),
        })
        .collect();

    Ok(Json(GetRoomsResponse { rooms: room_list }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        rooms: Arc::new(RwLock::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/rooms", post(create_room))
        .route("/rooms", get(get_rooms))
        .route("/rooms/{id}/join", post(join_room))
        .route("/rooms/{id}/messages", post(send_message))
        .route("/rooms/{id}/messages", get(get_messages))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    info!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
