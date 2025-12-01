use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::models::{PromptTemplate, UpdateTemplateRequest};
use crate::AppState;

pub async fn list_templates(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PromptTemplate>>, StatusCode> {
    let templates = state.templates.read().await;
    Ok(Json(templates.list()))
}

pub async fn update_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTemplateRequest>,
) -> Result<Json<PromptTemplate>, StatusCode> {
    let mut templates = state.templates.write().await;
    
    let template = templates.update(&id, &req.system_prompt)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(template))
}
