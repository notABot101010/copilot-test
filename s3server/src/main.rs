//! S3-compatible object storage server
//!
//! Features:
//! - Bucket operations: create, list, get, delete
//! - Object operations: put, get, head, list, delete
//! - Multipart uploads
//! - Conditional writes with If-None-Match
//! - Streaming uploads and downloads
//! - AWS Signature V4 authentication
//! - SQLite metadata storage

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use tracing::info;

mod auth;
mod database;
mod http_server;
mod storage;
mod xml_responses;

use auth::Authenticator;
use database::Database;
use http_server::{run_server, AppState};
use storage::Storage;

#[derive(Parser)]
#[command(name = "s3server")]
#[command(about = "An S3-compatible object storage server")]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value = "9000")]
    port: u16,

    /// Path to the data directory
    #[arg(short, long, default_value = "data")]
    data_path: PathBuf,

    /// Path to the SQLite database
    #[arg(short = 'D', long, default_value = "s3server.db")]
    database: PathBuf,

    /// AWS region for signature verification
    #[arg(short, long, default_value = "us-east-1")]
    region: String,

    /// Require authentication (default: false for development)
    #[arg(long, default_value = "false")]
    require_auth: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Create a new user with access keys
    CreateUser {
        /// User name
        name: String,
        /// Access key ID (generated if not provided)
        #[arg(long)]
        access_key: Option<String>,
        /// Secret access key (generated if not provided)
        #[arg(long)]
        secret_key: Option<String>,
    },
    /// List all users
    ListUsers,
    /// Start the server (default)
    Serve,
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
    let db = Database::connect(&db_url).await?;
    db.init().await?;

    match cli.command.clone().unwrap_or(Commands::Serve) {
        Commands::CreateUser {
            name,
            access_key,
            secret_key,
        } => {
            create_user(&db, &name, access_key, secret_key).await?;
        }
        Commands::ListUsers => {
            list_users(&db).await?;
        }
        Commands::Serve => {
            serve(&cli, db).await?;
        }
    }

    Ok(())
}

async fn create_user(
    db: &Database,
    name: &str,
    access_key: Option<String>,
    secret_key: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let access_key_id = access_key.unwrap_or_else(|| generate_access_key());
    let secret_access_key = secret_key.unwrap_or_else(|| generate_secret_key());

    let user = db
        .create_user(&access_key_id, &secret_access_key, name)
        .await?;

    info!("Created user: {}", user.name);
    println!("Access Key ID: {}", access_key_id);
    println!("Secret Access Key: {}", secret_access_key);
    println!("\nKeep these credentials safe - the secret key cannot be retrieved later!");

    Ok(())
}

async fn list_users(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let users = db.list_users().await?;

    if users.is_empty() {
        println!("No users found.");
        return Ok(());
    }

    println!("{:<20} {:<24} {}", "Name", "Access Key ID", "Created At");
    println!("{}", "-".repeat(60));
    for user in users {
        println!("{:<20} {:<24} {}", user.name, user.access_key_id, user.created_at);
    }

    Ok(())
}

async fn serve(cli: &Cli, db: Database) -> Result<(), Box<dyn std::error::Error>> {
    let db = Arc::new(db);
    let storage = Storage::new(cli.data_path.clone());
    let authenticator = Arc::new(Authenticator::new(db.clone(), &cli.region));

    let state = AppState {
        db,
        storage,
        authenticator,
        require_auth: cli.require_auth,
    };

    info!("Starting S3 server on port {}", cli.port);
    info!("Data directory: {:?}", cli.data_path);
    info!("Database: {:?}", cli.database);
    info!("Authentication required: {}", cli.require_auth);

    run_server(state, cli.port).await?;

    Ok(())
}

/// Generate a random access key ID (20 characters, uppercase alphanumeric)
fn generate_access_key() -> String {
    use aws_lc_rs::rand::SystemRandom;
    use aws_lc_rs::rand::SecureRandom;

    let rng = SystemRandom::new();
    let mut bytes = [0u8; 15];
    rng.fill(&mut bytes).expect("Failed to generate random bytes");

    let key = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes);
    key.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(20)
        .collect::<String>()
        .to_uppercase()
}

/// Generate a random secret access key (40 characters)
fn generate_secret_key() -> String {
    use aws_lc_rs::rand::SystemRandom;
    use aws_lc_rs::rand::SecureRandom;

    let rng = SystemRandom::new();
    let mut bytes = [0u8; 30];
    rng.fill(&mut bytes).expect("Failed to generate random bytes");

    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
}
