use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::models::{
    CreateSessionRequest, Message, SendMessageRequest, Session, SteerRequest,
};
use crate::AppState;

pub async fn create_session(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<Session>, StatusCode> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let name = req.name.unwrap_or_else(|| format!("Session {}", &id[..8]));

    let session = Session {
        id: id.clone(),
        name,
        created_at: now,
        updated_at: now,
        status: "active".to_string(),
    };

    sqlx::query(
        r#"
        INSERT INTO sessions (id, name, created_at, updated_at, status)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(&session.id)
    .bind(&session.name)
    .bind(&session.created_at.to_rfc3339())
    .bind(&session.updated_at.to_rfc3339())
    .bind(&session.status)
    .execute(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to create session: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(session))
}

pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Session>>, StatusCode> {
    let sessions: Vec<Session> = sqlx::query_as(
        r#"
        SELECT id, name, created_at, updated_at, status
        FROM sessions
        ORDER BY updated_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to list sessions: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(sessions))
}

pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Session>, StatusCode> {
    let session: Session = sqlx::query_as(
        r#"
        SELECT id, name, created_at, updated_at, status
        FROM sessions
        WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get session: {:?}", err);
        StatusCode::NOT_FOUND
    })?;

    Ok(Json(session))
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<Message>, StatusCode> {
    // Verify session exists
    let _: Session = sqlx::query_as(
        "SELECT id, name, created_at, updated_at, status FROM sessions WHERE id = ?",
    )
    .bind(&session_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let message = Message {
        id: id.clone(),
        session_id: session_id.clone(),
        role: "user".to_string(),
        content: req.content.clone(),
        created_at: now,
        metadata: None,
    };

    sqlx::query(
        r#"
        INSERT INTO messages (id, session_id, role, content, created_at, metadata)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&message.id)
    .bind(&message.session_id)
    .bind(&message.role)
    .bind(&message.content)
    .bind(&message.created_at.to_rfc3339())
    .bind(&message.metadata)
    .execute(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to insert message: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Update session timestamp
    sqlx::query("UPDATE sessions SET updated_at = ? WHERE id = ?")
        .bind(&now.to_rfc3339())
        .bind(&session_id)
        .execute(&state.db)
        .await
        .ok();

    // Process message asynchronously with orchestrator
    let orchestrator = state.orchestrator.clone();
    let db = state.db.clone();
    let templates = state.templates.clone();
    let sid = session_id.clone();
    let content = req.content.clone();
    
    tokio::spawn(async move {
        if let Err(err) = orchestrator.process_message(&db, &templates, &sid, &content).await {
            tracing::error!("Orchestrator error: {:?}", err);
        }
    });

    Ok(Json(message))
}

pub async fn get_messages(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Result<Json<Vec<Message>>, StatusCode> {
    let messages: Vec<Message> = sqlx::query_as(
        r#"
        SELECT id, session_id, role, content, created_at, metadata
        FROM messages
        WHERE session_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(&session_id)
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get messages: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(messages))
}

pub async fn steer_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(req): Json<SteerRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Verify session exists
    let _: Session = sqlx::query_as(
        "SELECT id, name, created_at, updated_at, status FROM sessions WHERE id = ?",
    )
    .bind(&session_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    // Send steering command to orchestrator
    state.orchestrator.steer(&session_id, req.command.clone()).await
        .map_err(|err| {
            tracing::error!("Failed to steer session: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "status": "ok",
        "command": req.command
    })))
}
