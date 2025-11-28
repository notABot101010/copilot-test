use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use russh::keys::ssh_key::PublicKey;

mod config;
mod database;
mod ssh_server;

use config::Config;
use database::Database;
use ssh_server::GitServer;

#[derive(Parser)]
#[command(name = "git-server")]
#[command(about = "A Git server over SSH implemented in Rust")]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,

    /// Path to the repositories directory
    #[arg(short, long, default_value = "repos")]
    repos_path: PathBuf,

    /// Path to the SQLite database
    #[arg(short, long, default_value = "git-server.db")]
    database: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new repository
    CreateRepo {
        /// Name of the repository
        name: String,
    },
    /// Start the git server
    Serve,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Load configuration
    let config = if cli.config.exists() {
        Config::load(&cli.config)?
    } else {
        eprintln!(
            "Warning: Config file not found at {:?}, using defaults",
            cli.config
        );
        Config::default_config()
    };

    // Create repos directory if it doesn't exist
    if !cli.repos_path.exists() {
        std::fs::create_dir_all(&cli.repos_path)?;
    }

    // Connect to database
    let db_url = format!("sqlite:{}?mode=rwc", cli.database.display());
    let db = Database::connect(&db_url).await?;
    db.init().await?;

    match cli.command {
        Commands::CreateRepo { name } => {
            create_repository(&db, &name, &cli.repos_path).await?;
        }
        Commands::Serve => {
            serve(&config, &db, &cli.repos_path).await?;
        }
    }

    Ok(())
}

async fn create_repository(
    db: &Database,
    name: &str,
    repos_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if repository already exists
    if db.get_repository(name).await?.is_some() {
        return Err(format!("Repository '{}' already exists", name).into());
    }

    // Create repository path
    let repo_path = repos_path.join(format!("{}.git", name));

    // Initialize bare git repository
    database::init_bare_repo(&repo_path).await?;

    // Store in database
    let relative_path = format!("{}.git", name);
    db.create_repository(name, &relative_path).await?;

    println!("Created repository '{}' at {:?}", name, repo_path);

    Ok(())
}

async fn serve(
    config: &Config,
    db: &Database,
    repos_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse authorized public keys from config
    let mut authorized_keys: Vec<PublicKey> = Vec::new();
    for key_str in &config.public_keys {
        match key_str.parse::<PublicKey>() {
            Ok(key) => authorized_keys.push(key),
            Err(e) => {
                eprintln!("Warning: Failed to parse public key: {} - {}", key_str, e);
            }
        }
    }

    if authorized_keys.is_empty() && !config.public_keys.is_empty() {
        return Err("No valid public keys found in configuration".into());
    }

    if authorized_keys.is_empty() {
        eprintln!("Warning: No authorized keys configured, authentication will fail for all users");
    } else {
        println!("Loaded {} authorized public key(s)", authorized_keys.len());
    }

    let server = GitServer::new(
        Arc::new(config.clone()),
        Arc::new(db.clone()),
        repos_path.clone(),
        authorized_keys,
    );

    server.run().await.map_err(|e| e.to_string())?;

    Ok(())
}
