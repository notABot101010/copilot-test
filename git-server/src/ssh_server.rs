use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use rand_core::OsRng;
use russh::keys::ssh_key::PublicKey;
use russh::server::{Auth, Msg, Server as _, Session};
use russh::{Channel, ChannelId, CryptoVec};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::database::Database;
use crate::git_ops;

/// SSH server for git operations
pub struct GitServer {
    config: Arc<Config>,
    db: Arc<Database>,
    repos_path: PathBuf,
    authorized_keys: Vec<PublicKey>,
}

impl GitServer {
    pub fn new(
        config: Arc<Config>,
        db: Arc<Database>,
        repos_path: PathBuf,
        authorized_keys: Vec<PublicKey>,
    ) -> Self {
        GitServer {
            config,
            db,
            repos_path,
            authorized_keys,
        }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key = russh::keys::PrivateKey::random(&mut OsRng, russh::keys::Algorithm::Ed25519)
            .map_err(|e| format!("Failed to generate server key: {}", e))?;

        let config = russh::server::Config {
            auth_rejection_time: std::time::Duration::from_secs(3),
            auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
            keys: vec![key],
            ..Default::default()
        };

        let config = Arc::new(config);
        let addr = format!("0.0.0.0:{}", self.config.ssh_port);

        info!("Git server listening on {}", addr);

        let mut server = GitServerInstance {
            db: self.db.clone(),
            repos_path: self.repos_path.clone(),
            authorized_keys: Arc::new(self.authorized_keys),
        };

        let socket = TcpListener::bind(&addr).await?;
        server.run_on_socket(config, &socket).await?;

        Ok(())
    }
}

#[derive(Clone)]
struct GitServerInstance {
    db: Arc<Database>,
    repos_path: PathBuf,
    authorized_keys: Arc<Vec<PublicKey>>,
}

impl russh::server::Server for GitServerInstance {
    type Handler = GitSessionHandler;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
        GitSessionHandler {
            db: self.db.clone(),
            repos_path: self.repos_path.clone(),
            authorized_keys: self.authorized_keys.clone(),
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

struct GitSessionHandler {
    db: Arc<Database>,
    repos_path: PathBuf,
    authorized_keys: Arc<Vec<PublicKey>>,
    handlers: Arc<Mutex<HashMap<ChannelId, git_ops::SshProtocolHandler>>>,
}

impl russh::server::Handler for GitSessionHandler {
    type Error = russh::Error;

    async fn auth_none(&mut self, _user: &str) -> Result<Auth, Self::Error> {
        Ok(Auth::Reject {
            proceed_with_methods: Some(russh::MethodSet::from(&[russh::MethodKind::PublicKey][..])),
            partial_success: false,
        })
    }

    async fn auth_publickey(
        &mut self,
        _user: &str,
        public_key: &PublicKey,
    ) -> Result<Auth, Self::Error> {
        // Check if the public key is in our authorized keys list
        for authorized_key in self.authorized_keys.iter() {
            if authorized_key == public_key {
                return Ok(Auth::Accept);
            }
        }

        Ok(Auth::Reject {
            proceed_with_methods: None,
            partial_success: false,
        })
    }

    async fn channel_open_session(
        &mut self,
        _channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let command = String::from_utf8_lossy(data);
        debug!("Received command: {}", command);

        // Parse git command (git-upload-pack 'org/project/repo' or git-receive-pack 'org/project/repo')
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() != 2 {
            session.channel_failure(channel)?;
            return Ok(());
        }

        let git_cmd = parts[0];
        let repo_path_str = parts[1].trim_matches('\'').trim_matches('"');

        // Validate git command
        if git_cmd != "git-upload-pack" && git_cmd != "git-receive-pack" {
            session.channel_failure(channel)?;
            return Ok(());
        }

        // Parse org/project/repo format
        let repo_parts: Vec<&str> = repo_path_str.split('/').collect();
        if repo_parts.len() != 3 {
            warn!("Invalid repository path format (expected org/project/repo): {}", repo_path_str);
            session.channel_failure(channel)?;
            return Ok(());
        }
        
        let org_name = repo_parts[0];
        let project_name = repo_parts[1];
        let repo_name = repo_parts[2];

        // Validate repository name contains no path traversal sequences
        if org_name.contains("..") || org_name.starts_with('/') || org_name.contains('\0') ||
           project_name.contains("..") || project_name.starts_with('/') || project_name.contains('\0') ||
           repo_name.contains("..") || repo_name.starts_with('/') || repo_name.contains('\0') {
            warn!("Invalid repository name (potential path traversal): {}", repo_path_str);
            session.channel_failure(channel)?;
            return Ok(());
        }

        // Look up repository
        let repo = match self.db.get_repository(org_name, project_name, repo_name).await {
            Ok(Some(repo)) => repo,
            Ok(None) => {
                warn!("Repository not found: {}/{}/{}", org_name, project_name, repo_name);
                session.channel_failure(channel)?;
                return Ok(());
            }
            Err(e) => {
                error!("Database error: {}", e);
                session.channel_failure(channel)?;
                return Ok(());
            }
        };

        // Validate that the repository path doesn't contain path traversal
        if repo.path.contains("..") || repo.path.starts_with('/') {
            warn!("Invalid repository path in database: {}", repo.path);
            session.channel_failure(channel)?;
            return Ok(());
        }

        let repo_path = self.repos_path.join(&repo.path);

        // Verify the resolved path is within the repos directory
        let canonical_repos = match self.repos_path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to canonicalize repos path: {}", e);
                session.channel_failure(channel)?;
                return Ok(());
            }
        };
        let canonical_repo = match repo_path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to canonicalize repository path: {}", e);
                session.channel_failure(channel)?;
                return Ok(());
            }
        };
        if !canonical_repo.starts_with(&canonical_repos) {
            warn!("Repository path escape attempt: {:?}", canonical_repo);
            session.channel_failure(channel)?;
            return Ok(());
        }

        // Create git2 protocol handler
        let handler = git_ops::SshProtocolHandler::new(&canonical_repo, git_cmd);
        self.handlers.lock().await.insert(channel, handler);
        session.channel_success(channel)?;

        Ok(())
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let mut handlers = self.handlers.lock().await;

        if let Some(handler) = handlers.get_mut(&channel) {
            handler.write_input(data);
        } else {
            session.channel_failure(channel)?;
        }

        Ok(())
    }

    async fn channel_eof(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let mut handlers = self.handlers.lock().await;

        if let Some(handler) = handlers.remove(&channel) {
            // Process the git protocol and get output
            match handler.finish() {
                Ok(output) => {
                    // Send output in chunks
                    const CHUNK_SIZE: usize = 32768; // 32KB chunks
                    for chunk in output.chunks(CHUNK_SIZE) {
                        session.data(channel, CryptoVec::from(chunk))?;
                    }
                    
                    // Exit with success
                    session.exit_status_request(channel, 0)?;
                }
                Err(e) => {
                    error!("Git protocol error: {}", e);
                    session.exit_status_request(channel, 1)?;
                }
            }
            
            session.close(channel)?;
        }

        Ok(())
    }
}
