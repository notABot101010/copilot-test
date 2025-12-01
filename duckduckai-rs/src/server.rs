use anyhow::{Context, Result, anyhow};
use axum::{
    Router,
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::{
        IntoResponse, Response,
        sse::{Event, Sse},
    },
    routing::post,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use crate::{DEFAULT_MODEL, DuckDuckGoClient};

#[derive(Debug)]
pub struct ServerState {
    api_key: String,
    client: Arc<Mutex<DuckDuckGoClient>>,
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

    // Validate that we have at least one message
    if request.messages.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No messages provided".to_string()));
    }

    // Convert OpenAI-format messages to DuckDuckGo ChatMessage format
    let messages: Vec<crate::ChatMessage> = request
        .messages
        .iter()
        .map(|msg| crate::ChatMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        })
        .collect();

    let model = request
        .model
        .as_deref()
        .unwrap_or(DEFAULT_MODEL)
        .to_string();

    if request.stream {
        // Streaming response
        let stream = create_streaming_response(state.client.clone(), messages, model);
        Ok(Sse::new(stream).into_response())
    } else {
        // Non-streaming response
        let response = create_non_streaming_response(state.client.clone(), messages, model)
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

        Ok(Json(response).into_response())
    }
}

async fn create_non_streaming_response(
    client: Arc<Mutex<DuckDuckGoClient>>,
    messages: Vec<crate::ChatMessage>,
    model: String,
) -> Result<ChatCompletionResponse> {
    let model_for_response = model.clone();

    let content = {
        let mut client = client.lock().await;
        client.chat_with_messages(messages, Some(&model)).await?
    };

    let id = format!("chatcmpl-{}", Uuid::new_v4());
    let created = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(ChatCompletionResponse {
        id,
        object: "chat.completion".to_string(),
        created,
        model: model_for_response,
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
    client: Arc<Mutex<DuckDuckGoClient>>,
    messages: Vec<crate::ChatMessage>,
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

        // Clone values needed for the closure before borrowing
        let id_for_closure = id.clone();
        let model_for_closure = model.clone();
        let tx_clone = tx.clone();

        // Use the shared client
        {
            let mut client = client.lock().await;
            let result = client
                .chat_stream_with_messages(messages, Some(&model), move |chunk| {
                    let chunk_data = ChatCompletionChunk {
                        id: id_for_closure.clone(),
                        object: "chat.completion.chunk".to_string(),
                        created,
                        model: model_for_closure.clone(),
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
    // Create a single DuckDuckGoClient to be shared across all requests
    let client = DuckDuckGoClient::new()?;

    let state = Arc::new(ServerState {
        api_key,
        client: Arc::new(Mutex::new(client)),
    });

    let app = Router::new()
        .route("/v1/chat/completions", post(handle_chat_completion))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    tracing::info!("Starting OpenAI-compatible server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("Failed to bind to address")?;

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}
