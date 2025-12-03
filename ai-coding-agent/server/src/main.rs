use axum::{
    routing::{get, post, put},
    Router,
};
use clap::Parser;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ai_coding_agent_server::{db, handlers, llm, orchestrator, templates};
use handlers::{
    sessions::{
        create_session, get_messages, get_session, list_sessions, send_message, steer_session,
    },
    templates::{list_templates, update_template},
    websocket::session_stream,
};
use llm::{LlmClient, LlmConfig, OpenAiLlmClient};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,

    #[arg(short, long, default_value = "8080")]
    port: u16,

    #[arg(long, default_value = "agent.db")]
    database: String,

    /// OpenAI-compatible API base URL
    #[arg(
        long,
        env = "OPENAI_BASE_URL",
        default_value = "https://api.openai.com/v1"
    )]
    api_base: String,

    /// API key for authentication
    #[arg(long, env = "OPENAI_API_KEY", default_value = "")]
    api_key: String,

    /// Model to use for completions
    #[arg(long, env = "OPENAI_MODEL", default_value = "gpt-4")]
    model: String,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    let db = db::init_db(&args.database).await?;

    let templates = Arc::new(tokio::sync::RwLock::new(templates::TemplateManager::new()));

    // Configure LLM client
    let llm_config = LlmConfig::new()
        .with_api_base(&args.api_base)
        .with_api_key(&args.api_key)
        .with_model(&args.model);

    let llm_client: Arc<dyn LlmClient> = Arc::new(OpenAiLlmClient::new(llm_config));

    let orchestrator = Arc::new(orchestrator::Orchestrator::new(llm_client.clone()));

    let state = Arc::new(ai_coding_agent_server::AppState {
        db,
        templates,
        orchestrator,
        llm_client,
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
    tracing::info!("Using API base: {}", args.api_base);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Simple Result type alias for error handling
type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
