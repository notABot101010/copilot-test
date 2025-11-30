use std::sync::Arc;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};

use crate::AppState;

pub async fn session_stream(
    ws: WebSocketUpgrade,
    Path(session_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, session_id, state))
}

async fn handle_socket(socket: WebSocket, session_id: String, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to session events
    let mut rx = state.orchestrator.subscribe(&session_id);

    // Task to send events to client
    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let msg = serde_json::to_string(&event).unwrap_or_default();
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Task to receive client messages (for steering)
    let orchestrator = state.orchestrator.clone();
    let sid = session_id.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // Parse steering command
                if let Ok(steer_req) = serde_json::from_str::<crate::models::SteerRequest>(&text) {
                    let _ = orchestrator.steer(&sid, steer_req.command).await;
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    // Send a closing event
    let _ = state.orchestrator.unsubscribe(&session_id);
    tracing::info!("WebSocket closed for session {}", session_id);
}
