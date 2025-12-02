//! TVflix - Self-hosted media center server
//!
//! Features:
//! - User authentication (register, login, logout)
//! - Media upload with streaming to filesystem
//! - Thumbnail generation for videos
//! - Media library management (music, photos, videos)
//! - Streaming playback

use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use tracing::info;

use tvflix_server::{database, handlers, storage, AppState};

#[derive(Parser)]
#[command(name = "tvflix-server")]
#[command(about = "Self-hosted media center server")]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Path to the data directory for media files
    #[arg(short, long, default_value = "data")]
    data_path: PathBuf,

    /// Path to the SQLite database
    #[arg(short = 'D', long, default_value = "tvflix.db")]
    database: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    // Create data directory if it doesn't exist
    if !cli.data_path.exists() {
        std::fs::create_dir_all(&cli.data_path)?;
    }

    // Connect to database
    let db_url = format!("sqlite:{}?mode=rwc", cli.database.display());
    let db = database::Database::connect(&db_url).await?;
    db.init().await?;

    let storage = storage::Storage::new(cli.data_path.clone());

    let state = AppState {
        db: Arc::new(db),
        storage: Arc::new(storage),
    };

    info!("Starting TVflix server on port {}", cli.port);
    info!("Data directory: {:?}", cli.data_path);
    info!("Database: {:?}", cli.database);

    let app = handlers::create_router(state);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], cli.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
