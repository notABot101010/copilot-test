use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;

mod api;
mod cache;
mod database;
mod error;
mod filter;
mod index;
mod search;
mod storage;

use api::run_server;
use database::Database;
use storage::S3Storage;

#[derive(Parser)]
#[command(name = "vectordb")]
#[command(about = "A serverless hybrid search vector database with S3 storage")]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value = "8080", env = "PORT")]
    port: u16,

    /// S3 bucket name
    #[arg(long, env = "S3_BUCKET")]
    s3_bucket: String,

    /// S3 endpoint URL (for S3-compatible services)
    #[arg(long, env = "S3_ENDPOINT")]
    s3_endpoint: Option<String>,

    /// S3 region
    #[arg(long, default_value = "us-east-1", env = "S3_REGION")]
    s3_region: String,

    /// Path to SQLite database
    #[arg(short, long, default_value = "vectordb.db", env = "DATABASE_PATH")]
    database: PathBuf,

    /// Path for disk cache
    #[arg(long, default_value = "/tmp/vectordb-cache", env = "CACHE_PATH")]
    cache_path: PathBuf,

    /// Memory cache size in MB
    #[arg(long, default_value = "256", env = "MEMORY_CACHE_MB")]
    memory_cache_mb: usize,

    /// Disk cache size in MB
    #[arg(long, default_value = "1024", env = "DISK_CACHE_MB")]
    disk_cache_mb: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    // Initialize SQLite database
    let db_url = format!("sqlite:{}?mode=rwc", cli.database.display());
    let db = Database::connect(&db_url).await?;
    db.init().await?;

    // Initialize S3 storage with caching
    let storage = S3Storage::new(
        &cli.s3_bucket,
        cli.s3_endpoint.as_deref(),
        &cli.s3_region,
        &cli.cache_path,
        cli.memory_cache_mb,
        cli.disk_cache_mb,
    )
    .await?;

    tracing::info!("VectorDB starting on port {}", cli.port);
    tracing::info!("S3 bucket: {}", cli.s3_bucket);
    tracing::info!("Database: {}", cli.database.display());

    // Run the HTTP server
    run_server(cli.port, Arc::new(db), Arc::new(storage)).await?;

    Ok(())
}
