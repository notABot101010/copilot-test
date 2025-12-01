//! AI Coding Agent Server Library
//! 
//! This library provides the core functionality for the AI Coding Agent server.

pub mod db;
pub mod handlers;
pub mod llm;
pub mod models;
pub mod orchestrator;
pub mod subagents;
pub mod templates;
pub mod tools;

use std::sync::Arc;
use sqlx::sqlite::SqlitePool;

use llm::LlmClient;

pub struct AppState {
    pub db: SqlitePool,
    pub templates: Arc<tokio::sync::RwLock<templates::TemplateManager>>,
    pub orchestrator: Arc<orchestrator::Orchestrator>,
    pub llm_client: Arc<dyn LlmClient>,
}
