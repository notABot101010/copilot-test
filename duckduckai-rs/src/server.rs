use anyhow::{Context, Result, anyhow};
use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response, sse::{Event, Sse}},
    routing::post,
    Router,
};
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use crate::DuckDuckGoClient;

#[derive(Debug, Clone)]
pub struct ServerState {
    api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<ChatCompletionMessage>,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionChoice {
    pub index: u32,
    pub message: ChatCompletionResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionResponseMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatCompletionChunkChoice>,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionChunkChoice {
    pub index: u32,
    pub delta: ChatCompletionChunkDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionChunkDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

fn extract_api_key(headers: &HeaderMap) -> Result<String> {
    let auth_header = headers
        .get("authorization")
        .ok_or_else(|| anyhow!("Missing Authorization header"))?
        .to_str()
        .context("Invalid Authorization header")?;

    if !auth_header.starts_with("Bearer ") {
        return Err(anyhow!("Authorization header must start with 'Bearer '"));
    }

    Ok(auth_header[7..].to_string())
}

async fn handle_chat_completion(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Response, (StatusCode, String)> {
    // Validate API key
    let api_key = extract_api_key(&headers).map_err(|err| {
        (
            StatusCode::UNAUTHORIZED,
            format!("Authentication failed: {}", err),
        )
    })?;

    if api_key != state.api_key {
        return Err((StatusCode::UNAUTHORIZED, "Invalid API key".to_string()));
    }

    // Extract the last user message
    let user_message = request
        .messages
        .iter()
        .rev()
        .find(|msg| msg.role == "user")
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                "No user message found".to_string(),
            )
        })?;

    let model = request
        .model
        .as_deref()
        .unwrap_or("gpt-4o-mini")
        .to_string();

    if request.stream {
        // Streaming response
        let stream = create_streaming_response(user_message.content.clone(), model);
        Ok(Sse::new(stream).into_response())
    } else {
        // Non-streaming response
        let response = create_non_streaming_response(user_message.content.clone(), model)
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

        Ok(Json(response).into_response())
    }
}

async fn create_non_streaming_response(
    message: String,
    model: String,
) -> Result<ChatCompletionResponse> {
    let client = Arc::new(Mutex::new(DuckDuckGoClient::new()?));
    let client_clone = Arc::clone(&client);

    let content = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async {
            let mut client = client_clone.lock().unwrap();
            client.chat(&message, Some(&model)).await
        })
    })
    .await??;

    let id = format!("chatcmpl-{}", Uuid::new_v4());
    let created = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(ChatCompletionResponse {
        id,
        object: "chat.completion".to_string(),
        created,
        model,
        choices: vec![ChatCompletionChoice {
            index: 0,
            message: ChatCompletionResponseMessage {
                role: "assistant".to_string(),
                content,
            },
            finish_reason: Some("stop".to_string()),
        }],
    })
}

fn create_streaming_response(
    message: String,
    model: String,
) -> impl Stream<Item = Result<Event, anyhow::Error>> {
    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let id = format!("chatcmpl-{}", Uuid::new_v4());
        let created = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Send initial chunk with role
        let initial_chunk = ChatCompletionChunk {
            id: id.clone(),
            object: "chat.completion.chunk".to_string(),
            created,
            model: model.clone(),
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta: ChatCompletionChunkDelta {
                    role: Some("assistant".to_string()),
                    content: None,
                },
                finish_reason: None,
            }],
        };

        if let Ok(json) = serde_json::to_string(&initial_chunk) {
            let _ = tx.send(Ok(Event::default().data(json))).await;
        }

        // Create client and stream response
        let client_result = DuckDuckGoClient::new();
        if let Ok(mut client) = client_result {
            let tx_clone = tx.clone();
            let result = client
                .chat_stream(&message, Some(&model), move |chunk| {
                    let chunk_data = ChatCompletionChunk {
                        id: id.clone(),
                        object: "chat.completion.chunk".to_string(),
                        created,
                        model: model.clone(),
                        choices: vec![ChatCompletionChunkChoice {
                            index: 0,
                            delta: ChatCompletionChunkDelta {
                                role: None,
                                content: Some(chunk.to_string()),
                            },
                            finish_reason: None,
                        }],
                    };

                    if let Ok(json) = serde_json::to_string(&chunk_data) {
                        let tx_clone2 = tx_clone.clone();
                        tokio::spawn(async move {
                            let _ = tx_clone2.send(Ok(Event::default().data(json))).await;
                        });
                    }
                })
                .await;

            if result.is_err() {
                tracing::error!("Error during streaming: {:?}", result);
            }
        }

        // Send final chunk with finish_reason
        let final_chunk = ChatCompletionChunk {
            id: id.clone(),
            object: "chat.completion.chunk".to_string(),
            created,
            model: model.clone(),
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta: ChatCompletionChunkDelta {
                    role: None,
                    content: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
        };

        if let Ok(json) = serde_json::to_string(&final_chunk) {
            let _ = tx.send(Ok(Event::default().data(json))).await;
        }

        // Send [DONE] message
        let _ = tx.send(Ok(Event::default().data("[DONE]"))).await;
    });

    async_stream::stream! {
        while let Some(result) = rx.recv().await {
            yield result;
        }
    }
}

pub async fn run_server(host: &str, port: u16, api_key: String) -> Result<()> {
    let state = Arc::new(ServerState { api_key });

    let app = Router::new()
        .route("/v1/chat/completions", post(handle_chat_completion))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    tracing::info!("Starting OpenAI-compatible server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("Failed to bind to address")?;

    axum::serve(listener, app)
        .await
        .context("Server error")?;

    Ok(())
}
