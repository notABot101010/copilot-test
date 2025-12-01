use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use russh::keys::ssh_key::PublicKey;
use tracing::{error, info, warn};

mod config;
mod database;
mod error;
mod git_ops;
mod http_server;
mod sandbox;
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
    /// Create a new project
    CreateProject {
        /// Organization name
        #[arg(long)]
        org: String,
        /// Name of the project
        name: String,
        /// Display name of the project
        #[arg(long)]
        display_name: Option<String>,
    },
    /// Create a new repository
    CreateRepo {
        /// Organization name
        #[arg(long)]
        org: String,
        /// Project name
        #[arg(long)]
        project: String,
        /// Name of the repository
        name: String,
    },
    /// Start the git server
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

    // Load configuration
    let config = if cli.config.exists() {
        Config::load(&cli.config)?
    } else {
        warn!(
            "Config file not found at {:?}, using defaults",
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
        Commands::CreateProject { org, name, display_name } => {
            create_project(&db, &org, &name, display_name.as_deref(), &cli.repos_path).await?;
        }
        Commands::CreateRepo { org, project, name } => {
            create_repository(&db, &org, &project, &name, &cli.repos_path).await?;
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

    info!("Created organization '{}' at {:?}", name, org_path);

    Ok(())
}

async fn create_project(
    db: &Database,
    org: &str,
    name: &str,
    display_name: Option<&str>,
    repos_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if organization exists
    if db.get_organization(org).await?.is_none() {
        return Err(format!("Organization '{}' does not exist", org).into());
    }

    // Check if project already exists
    if db.get_project(org, name).await?.is_some() {
        return Err(format!("Project '{}/{}' already exists", org, name).into());
    }

    // Create project directory
    let project_path = repos_path.join(org).join(name);
    std::fs::create_dir_all(&project_path)?;

    // Store in database
    let dn = display_name.unwrap_or(name);
    db.create_project(org, name, dn, "").await?;

    info!("Created project '{}/{}' at {:?}", org, name, project_path);

    Ok(())
}

async fn create_repository(
    db: &Database,
    org: &str,
    project: &str,
    name: &str,
    repos_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if project exists
    if db.get_project(org, project).await?.is_none() {
        return Err(format!("Project '{}/{}' does not exist", org, project).into());
    }

    // Check if repository already exists
    if db.get_repository(org, project, name).await?.is_some() {
        return Err(format!("Repository '{}/{}/{}' already exists", org, project, name).into());
    }

    // Create repository path
    let repo_path = repos_path.join(org).join(project).join(format!("{}.git", name));

    // Ensure project directory exists
    std::fs::create_dir_all(repos_path.join(org).join(project))?;

    // Initialize bare git repository with main as default branch using git2
    git_ops::init_bare_repo(&repo_path, "main")
        .map_err(|e| format!("Failed to initialize bare git repository: {}", e))?;

    // Store in database
    let relative_path = format!("{}/{}/{}.git", org, project, name);
    db.create_repository(org, project, name, &relative_path).await?;

    info!("Created repository '{}/{}/{}' at {:?}", org, project, name, repo_path);

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
                warn!("Failed to parse public key: {} - {}", key_str, e);
            }
        }
    }

    if authorized_keys.is_empty() && !config.public_keys.is_empty() {
        return Err("No valid public keys found in configuration".into());
    }

    if authorized_keys.is_empty() {
        warn!("No authorized keys configured, authentication will fail for all users");
    } else {
        info!("Loaded {} authorized public key(s)", authorized_keys.len());
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
            error!("HTTP server error: {}", e);
        }
    });

    // Run SSH server (this blocks)
    let ssh_result = ssh_server.run().await;

    // Cancel HTTP server if SSH server stops
    http_handle.abort();

    ssh_result.map_err(|e| -> Box<dyn std::error::Error> { e })?;

    Ok(())
}
