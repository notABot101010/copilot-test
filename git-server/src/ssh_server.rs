use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use rand_core::OsRng;
use russh::keys::ssh_key::PublicKey;
use russh::server::{Auth, Msg, Server as _, Session};
use russh::{Channel, ChannelId, CryptoVec};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::database::Database;

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

        println!("Git server listening on {}", addr);

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
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

struct GitSessionHandler {
    db: Arc<Database>,
    repos_path: PathBuf,
    authorized_keys: Arc<Vec<PublicKey>>,
    processes: Arc<Mutex<HashMap<ChannelId, Child>>>,
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
        println!("Received command: {}", command);

        // Parse git command (git-upload-pack 'repo' or git-receive-pack 'repo')
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() != 2 {
            session.channel_failure(channel)?;
            return Ok(());
        }

        let git_cmd = parts[0];
        let repo_name = parts[1].trim_matches('\'').trim_matches('"');

        // Validate git command
        if git_cmd != "git-upload-pack" && git_cmd != "git-receive-pack" {
            session.channel_failure(channel)?;
            return Ok(());
        }

        // Validate repository name contains no path traversal sequences
        if repo_name.contains("..") || repo_name.starts_with('/') || repo_name.contains('\0') {
            eprintln!("Invalid repository name (potential path traversal): {}", repo_name);
            session.channel_failure(channel)?;
            return Ok(());
        }

        // Look up repository
        let repo = match self.db.get_repository(repo_name).await {
            Ok(Some(repo)) => repo,
            Ok(None) => {
                eprintln!("Repository not found: {}", repo_name);
                session.channel_failure(channel)?;
                return Ok(());
            }
            Err(e) => {
                eprintln!("Database error: {}", e);
                session.channel_failure(channel)?;
                return Ok(());
            }
        };

        // Validate that the repository path doesn't contain path traversal
        if repo.path.contains("..") || repo.path.starts_with('/') {
            eprintln!("Invalid repository path in database: {}", repo.path);
            session.channel_failure(channel)?;
            return Ok(());
        }

        let repo_path = self.repos_path.join(&repo.path);

        // Verify the resolved path is within the repos directory
        let canonical_repos = match self.repos_path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to canonicalize repos path: {}", e);
                session.channel_failure(channel)?;
                return Ok(());
            }
        };
        let canonical_repo = match repo_path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to canonicalize repository path: {}", e);
                session.channel_failure(channel)?;
                return Ok(());
            }
        };
        if !canonical_repo.starts_with(&canonical_repos) {
            eprintln!("Repository path escape attempt: {:?}", canonical_repo);
            session.channel_failure(channel)?;
            return Ok(());
        }

        // Spawn git process
        let child = Command::new(git_cmd)
            .arg(&canonical_repo)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        match child {
            Ok(child) => {
                self.processes.lock().await.insert(channel, child);
                session.channel_success(channel)?;
            }
            Err(e) => {
                eprintln!("Failed to spawn git process: {}", e);
                session.channel_failure(channel)?;
            }
        }

        Ok(())
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let mut processes = self.processes.lock().await;

        if let Some(child) = processes.get_mut(&channel) {
            if let Some(stdin) = child.stdin.as_mut() {
                if let Err(e) = stdin.write_all(data).await {
                    eprintln!("Failed to write to git process stdin: {}", e);
                    session.channel_failure(channel)?;
                }
            }
        }

        Ok(())
    }

    async fn channel_eof(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let mut processes = self.processes.lock().await;

        if let Some(mut child) = processes.remove(&channel) {
            // Close stdin to signal end of input
            drop(child.stdin.take());

            // Read stdout and send to client in chunks to handle large repositories
            if let Some(mut stdout) = child.stdout.take() {
                const CHUNK_SIZE: usize = 32768; // 32KB chunks
                let mut buffer = vec![0u8; CHUNK_SIZE];
                loop {
                    match stdout.read(&mut buffer).await {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            session.data(channel, CryptoVec::from(&buffer[..n]))?;
                        }
                        Err(e) => {
                            eprintln!("Failed to read git process stdout: {}", e);
                            break;
                        }
                    }
                }
            }

            // Wait for process to finish
            let status = child.wait().await;
            let exit_code = match status {
                Ok(s) => s.code().unwrap_or_else(|| {
                    // Process was terminated by signal
                    if s.success() { 0 } else { 1 }
                }) as u32,
                Err(e) => {
                    eprintln!("Failed to wait for git process: {}", e);
                    1
                }
            };

            session.exit_status_request(channel, exit_code)?;
            session.close(channel)?;
        }

        Ok(())
    }
}
