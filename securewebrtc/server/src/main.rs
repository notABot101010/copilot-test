use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, RwLock};
use tower_http::cors::CorsLayer;
use tracing::info;

type Rooms = Arc<RwLock<HashMap<String, Room>>>;
type Tx = mpsc::UnboundedSender<Message>;

#[derive(Default)]
struct Room {
    initiator: Option<Tx>,
    joiner: Option<Tx>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum SignalingMessage {
    #[serde(rename = "offer")]
    Offer { sdp: String },
    #[serde(rename = "answer")]
    Answer { sdp: String },
    #[serde(rename = "ice-candidate")]
    IceCandidate { candidate: String },
    #[serde(rename = "e2ee-key")]
    E2EEKey {
        #[serde(rename = "publicKey")]
        public_key: String,
    },
    #[serde(rename = "peer-joined")]
    PeerJoined,
    #[serde(rename = "peer-left")]
    PeerLeft,
    #[serde(rename = "room-full")]
    RoomFull,
    #[serde(rename = "waiting")]
    Waiting,
}

#[derive(Clone)]
struct AppState {
    rooms: Rooms,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let state = AppState {
        rooms: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/ws/:room_id", get(ws_handler))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{}", port);
    info!("Starting signaling server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, room_id, state))
}

async fn handle_socket(socket: WebSocket, room_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let is_initiator: bool;

    {
        let mut rooms = state.rooms.write().await;
        let room = rooms.entry(room_id.clone()).or_insert_with(Room::default);

        if room.initiator.is_none() {
            room.initiator = Some(tx.clone());
            is_initiator = true;
            info!("Initiator joined room: {}", room_id);
        } else if room.joiner.is_none() {
            room.joiner = Some(tx.clone());
            is_initiator = false;
            info!("Joiner joined room: {}", room_id);

            if let Some(initiator_tx) = &room.initiator {
                let msg = SignalingMessage::PeerJoined;
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = initiator_tx.send(Message::Text(json.into()));
                }
            }
        } else {
            info!("Room {} is full, rejecting connection", room_id);
            let msg = SignalingMessage::RoomFull;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = sender.send(Message::Text(json.into())).await;
            }
            return;
        }
    }

    if is_initiator {
        let msg = SignalingMessage::Waiting;
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = tx.send(Message::Text(json.into()));
        }
    }

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let room_id_clone = room_id.clone();
    let state_clone = state.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                let rooms = state_clone.rooms.read().await;
                if let Some(room) = rooms.get(&room_id_clone) {
                    let peer_tx = if is_initiator {
                        room.joiner.as_ref()
                    } else {
                        room.initiator.as_ref()
                    };

                    if let Some(peer) = peer_tx {
                        let _ = peer.send(Message::Text(text));
                    }
                }
            } else if let Message::Close(_) = msg {
                break;
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }

    {
        let mut rooms = state.rooms.write().await;
        if let Some(room) = rooms.get_mut(&room_id) {
            let peer_tx = if is_initiator {
                room.initiator = None;
                room.joiner.as_ref()
            } else {
                room.joiner = None;
                room.initiator.as_ref()
            };

            if let Some(peer) = peer_tx {
                let msg = SignalingMessage::PeerLeft;
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = peer.send(Message::Text(json.into()));
                }
            }

            if room.initiator.is_none() && room.joiner.is_none() {
                rooms.remove(&room_id);
                info!("Room {} removed (empty)", room_id);
            }
        }
    }

    info!("Client disconnected from room: {}", room_id);
}
