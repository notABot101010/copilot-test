use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use russh::keys::ssh_key::PublicKey;

mod config;
mod database;
mod http_server;
mod ssh_server;

use config::Config;
use database::Database;
use http_server::run_http_server;
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
    /// Create a new organization
    CreateOrg {
        /// Name of the organization
        name: String,
        /// Display name of the organization
        #[arg(long)]
        display_name: Option<String>,
    },
    /// Create a new repository
    CreateRepo {
        /// Organization name
        #[arg(long)]
        org: String,
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
        Commands::CreateOrg { name, display_name } => {
            create_organization(&db, &name, display_name.as_deref(), &cli.repos_path).await?;
        }
        Commands::CreateRepo { org, name } => {
            create_repository(&db, &org, &name, &cli.repos_path).await?;
        }
        Commands::Serve => {
            serve(&config, &db, &cli.repos_path).await?;
        }
    }

    Ok(())
}

async fn create_organization(
    db: &Database,
    name: &str,
    display_name: Option<&str>,
    repos_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if organization already exists
    if db.get_organization(name).await?.is_some() {
        return Err(format!("Organization '{}' already exists", name).into());
    }

    // Create organization directory
    let org_path = repos_path.join(name);
    std::fs::create_dir_all(&org_path)?;

    // Store in database
    let dn = display_name.unwrap_or(name);
    db.create_organization(name, dn, "").await?;

    println!("Created organization '{}' at {:?}", name, org_path);

    Ok(())
}

async fn create_repository(
    db: &Database,
    org: &str,
    name: &str,
    repos_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if organization exists
    if db.get_organization(org).await?.is_none() {
        return Err(format!("Organization '{}' does not exist", org).into());
    }

    // Check if repository already exists
    if db.get_repository(org, name).await?.is_some() {
        return Err(format!("Repository '{}/{}' already exists", org, name).into());
    }

    // Create repository path
    let repo_path = repos_path.join(org).join(format!("{}.git", name));

    // Ensure org directory exists
    std::fs::create_dir_all(repos_path.join(org))?;

    // Initialize bare git repository with main as default branch
    let status = tokio::process::Command::new("git")
        .args(["init", "--bare", "--initial-branch=main"])
        .arg(&repo_path)
        .status()
        .await?;

    if !status.success() {
        return Err("Failed to initialize bare git repository".into());
    }

    // Store in database
    let relative_path = format!("{}/{}.git", org, name);
    db.create_repository(org, name, &relative_path).await?;

    println!("Created repository '{}/{}' at {:?}", org, name, repo_path);

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

    let config = Arc::new(config.clone());
    let db = Arc::new(db.clone());

    // Start SSH server
    let ssh_server = GitServer::new(
        config.clone(),
        db.clone(),
        repos_path.clone(),
        authorized_keys,
    );

    // Start HTTP server
    let http_config = config.clone();
    let http_db = db.clone();
    let http_repos_path = repos_path.clone();
    let http_handle = tokio::spawn(async move {
        if let Err(e) = run_http_server(http_config, http_db, http_repos_path).await {
            eprintln!("HTTP server error: {}", e);
        }
    });

    // Run SSH server (this blocks)
    let ssh_result = ssh_server.run().await;

    // Cancel HTTP server if SSH server stops
    http_handle.abort();

    ssh_result.map_err(|e| -> Box<dyn std::error::Error> { e })?;

    Ok(())
}
