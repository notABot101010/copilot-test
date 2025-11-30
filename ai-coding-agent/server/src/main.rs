use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, put},
};
use clap::Parser;
use sqlx::sqlite::SqlitePool;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod handlers;
mod models;
mod orchestrator;
mod subagents;
mod templates;
mod tools;

use handlers::{
    sessions::{create_session, get_session, list_sessions, send_message, get_messages, steer_session},
    templates::{list_templates, update_template},
    websocket::session_stream,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,

    #[arg(short, long, default_value = "8080")]
    port: u16,

    #[arg(long, default_value = "agent.db")]
    database: String,
}

pub struct AppState {
    pub db: SqlitePool,
    pub templates: Arc<tokio::sync::RwLock<templates::TemplateManager>>,
    pub orchestrator: Arc<orchestrator::Orchestrator>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    let db = db::init_db(&args.database).await?;
    
    let templates = Arc::new(tokio::sync::RwLock::new(templates::TemplateManager::new()));
    let orchestrator = Arc::new(orchestrator::Orchestrator::new());

    let state = Arc::new(AppState {
        db,
        templates,
        orchestrator,
    });

    let app = Router::new()
        .route("/api/sessions", post(create_session))
        .route("/api/sessions", get(list_sessions))
        .route("/api/sessions/:id", get(get_session))
        .route("/api/sessions/:id/messages", post(send_message))
        .route("/api/sessions/:id/messages", get(get_messages))
        .route("/api/sessions/:id/steer", post(steer_session))
        .route("/api/sessions/:id/stream", get(session_stream))
        .route("/api/templates", get(list_templates))
        .route("/api/templates/:id", put(update_template))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{}:{}", args.host, args.port);
    tracing::info!("Starting AI Coding Agent server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Import anyhow for Result
mod anyhow {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
}
