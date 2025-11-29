use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode, Request},
    middleware::{self, Next},
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use tokio::process::Command;
use tower_http::services::ServeDir;

use crate::config::Config;
use crate::database::Database;

/// Shared state for the HTTP server
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<Database>,
    pub repos_path: PathBuf,
}

/// Repository info for API responses
#[derive(Debug, Serialize)]
pub struct RepoInfo {
    pub name: String,
    pub path: String,
}

/// Commit info for API responses
#[derive(Debug, Serialize)]
pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

/// File entry for API responses
#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub entry_type: String, // "file" or "dir"
    pub size: Option<u64>,
}

/// Query parameters for tree endpoint
#[derive(Debug, Deserialize)]
pub struct TreeQuery {
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
    pub path: Option<String>,
}

/// Create the HTTP router
pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        .route("/repos", get(list_repos))
        .route("/repos/:name", get(get_repo))
        .route("/repos/:name/files", get(list_files))
        .route("/repos/:name/commits", get(list_commits))
        .route("/repos/:name/tree", get(get_tree))
        .route("/repos/:name/blob", get(get_blob_root))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Try to find static directory - check multiple locations
    let static_dir = find_static_dir();

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(ServeDir::new(static_dir).fallback(get(serve_index)))
        .with_state(state);

    app
}

/// Find the static directory by checking multiple locations
fn find_static_dir() -> PathBuf {
    // Try current dir/static
    if let Ok(cwd) = std::env::current_dir() {
        let dir = cwd.join("static");
        if dir.exists() {
            return dir;
        }
        // Try current dir/git-server/static
        let dir = cwd.join("git-server").join("static");
        if dir.exists() {
            return dir;
        }
    }
    
    // Try relative to executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let dir = exe_dir.join("static");
            if dir.exists() {
                return dir;
            }
        }
    }
    
    // Fall back to "static" relative path
    PathBuf::from("static")
}

/// Basic auth middleware
async fn auth_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // If no auth is configured, allow all requests
    if state.config.auth.is_empty() {
        return next.run(request).await;
    }

    // Get the Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(auth) if auth.starts_with("Basic ") => {
            let encoded = &auth[6..];
            if let Ok(decoded) = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                encoded,
            ) {
                if let Ok(credentials) = String::from_utf8(decoded) {
                    if let Some((user, password)) = credentials.split_once(':') {
                        // Hash the password
                        let mut hasher = Sha512::new();
                        hasher.update(password.as_bytes());
                        let hash = hex::encode(hasher.finalize());

                        // Check credentials
                        for cred in &state.config.auth {
                            if cred.user == user && cred.password_hash == hash {
                                return next.run(request).await;
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    // Return 401 Unauthorized
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::WWW_AUTHENTICATE, "Basic realm=\"Git Server\"")
        .body("Unauthorized".into())
        .unwrap()
}

/// Serve index.html for SPA routing
async fn serve_index() -> impl IntoResponse {
    Html(include_str!("../static/index.html"))
}

/// List all repositories
async fn list_repos(State(state): State<AppState>) -> Result<Json<Vec<RepoInfo>>, (StatusCode, String)> {
    let repos = state
        .db
        .list_repositories()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let repo_infos: Vec<RepoInfo> = repos
        .into_iter()
        .map(|r| RepoInfo {
            name: r.name,
            path: r.path,
        })
        .collect();

    Ok(Json(repo_infos))
}

/// Get a single repository
async fn get_repo(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<RepoInfo>, (StatusCode, String)> {
    let repo = state
        .db
        .get_repository(&name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Repository not found".to_string()))?;

    Ok(Json(RepoInfo {
        name: repo.name,
        path: repo.path,
    }))
}

/// List files in a repository (root of default branch)
async fn list_files(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Vec<FileEntry>>, (StatusCode, String)> {
    let repo = state
        .db
        .get_repository(&name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Repository not found".to_string()))?;

    let repo_path = state.repos_path.join(&repo.path);
    let files = list_git_files(&repo_path, "HEAD", "").await?;

    Ok(Json(files))
}

/// List commits in a repository
async fn list_commits(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Vec<CommitInfo>>, (StatusCode, String)> {
    let repo = state
        .db
        .get_repository(&name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Repository not found".to_string()))?;

    let repo_path = state.repos_path.join(&repo.path);
    let commits = list_git_commits(&repo_path).await?;

    Ok(Json(commits))
}

/// Get tree at a specific ref and path
async fn get_tree(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(query): Query<TreeQuery>,
) -> Result<Json<Vec<FileEntry>>, (StatusCode, String)> {
    let repo = state
        .db
        .get_repository(&name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Repository not found".to_string()))?;

    let repo_path = state.repos_path.join(&repo.path);
    let git_ref = query.git_ref.as_deref().unwrap_or("HEAD");
    let path = query.path.as_deref().unwrap_or("");
    let files = list_git_files(&repo_path, git_ref, path).await?;

    Ok(Json(files))
}

/// Get tree at a specific path
#[allow(dead_code)]
async fn get_tree_path(
    State(state): State<AppState>,
    Path((name, path)): Path<(String, String)>,
    Query(query): Query<TreeQuery>,
) -> Result<Json<Vec<FileEntry>>, (StatusCode, String)> {
    let repo = state
        .db
        .get_repository(&name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Repository not found".to_string()))?;

    let repo_path = state.repos_path.join(&repo.path);
    let git_ref = query.git_ref.as_deref().unwrap_or("HEAD");
    let files = list_git_files(&repo_path, git_ref, &path).await?;

    Ok(Json(files))
}

/// Get blob content (for root path, uses query param)
async fn get_blob_root(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(query): Query<TreeQuery>,
) -> Result<String, (StatusCode, String)> {
    let repo = state
        .db
        .get_repository(&name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Repository not found".to_string()))?;

    let repo_path = state.repos_path.join(&repo.path);
    let git_ref = query.git_ref.as_deref().unwrap_or("HEAD");
    let path = query.path.as_deref().ok_or((StatusCode::BAD_REQUEST, "path query parameter required".to_string()))?;
    let content = get_git_blob(&repo_path, git_ref, path).await?;

    Ok(content)
}

/// Get blob content (legacy, with path in URL)
#[allow(dead_code)]
async fn get_blob(
    State(state): State<AppState>,
    Path((name, path)): Path<(String, String)>,
    Query(query): Query<TreeQuery>,
) -> Result<String, (StatusCode, String)> {
    let repo = state
        .db
        .get_repository(&name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Repository not found".to_string()))?;

    let repo_path = state.repos_path.join(&repo.path);
    let git_ref = query.git_ref.as_deref().unwrap_or("HEAD");
    let content = get_git_blob(&repo_path, git_ref, &path).await?;

    Ok(content)
}

/// List files in a git repository at a specific ref and path
async fn list_git_files(
    repo_path: &PathBuf,
    git_ref: &str,
    path: &str,
) -> Result<Vec<FileEntry>, (StatusCode, String)> {
    // Validate ref to prevent command injection
    if !is_safe_git_ref(git_ref) {
        return Err((StatusCode::BAD_REQUEST, "Invalid git ref".to_string()));
    }
    
    // Validate path to prevent path traversal
    if path.contains("..") {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_string()));
    }

    let tree_path = if path.is_empty() {
        git_ref.to_string()
    } else {
        format!("{}:{}", git_ref, path)
    };

    let output = Command::new("git")
        .args(["ls-tree", "-l", &tree_path])
        .current_dir(repo_path)
        .output()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Not a valid object") || stderr.contains("fatal:") {
            return Ok(vec![]);
        }
        return Err((StatusCode::INTERNAL_SERVER_ERROR, stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();

    for line in stdout.lines() {
        // Format: <mode> <type> <hash> <size>\t<name>
        // Example: 100644 blob abc123 1234    file.txt
        if let Some((meta, name)) = line.split_once('\t') {
            let parts: Vec<&str> = meta.split_whitespace().collect();
            if parts.len() >= 4 {
                let entry_type = match parts[1] {
                    "blob" => "file",
                    "tree" => "dir",
                    _ => continue,
                };
                let size = if entry_type == "file" {
                    parts[3].parse().ok()
                } else {
                    None
                };
                let entry_path = if path.is_empty() {
                    name.to_string()
                } else {
                    format!("{}/{}", path, name)
                };
                entries.push(FileEntry {
                    name: name.to_string(),
                    path: entry_path,
                    entry_type: entry_type.to_string(),
                    size,
                });
            }
        }
    }

    Ok(entries)
}

/// List commits in a git repository
async fn list_git_commits(repo_path: &PathBuf) -> Result<Vec<CommitInfo>, (StatusCode, String)> {
    let output = Command::new("git")
        .args([
            "log",
            "--format=%H%n%h%n%an%n%ai%n%s%n---",
            "-n",
            "50",
        ])
        .current_dir(repo_path)
        .output()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("does not have any commits") {
            return Ok(vec![]);
        }
        return Err((StatusCode::INTERNAL_SERVER_ERROR, stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();

    for commit_block in stdout.split("---\n") {
        let lines: Vec<&str> = commit_block.lines().collect();
        if lines.len() >= 5 {
            commits.push(CommitInfo {
                hash: lines[0].to_string(),
                short_hash: lines[1].to_string(),
                author: lines[2].to_string(),
                date: lines[3].to_string(),
                message: lines[4].to_string(),
            });
        }
    }

    Ok(commits)
}

/// Get blob content from git
async fn get_git_blob(
    repo_path: &PathBuf,
    git_ref: &str,
    path: &str,
) -> Result<String, (StatusCode, String)> {
    // Validate ref to prevent command injection
    if !is_safe_git_ref(git_ref) {
        return Err((StatusCode::BAD_REQUEST, "Invalid git ref".to_string()));
    }
    
    // Validate path to prevent path traversal
    if path.contains("..") {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_string()));
    }

    let blob_path = format!("{}:{}", git_ref, path);

    let output = Command::new("git")
        .args(["show", &blob_path])
        .current_dir(repo_path)
        .output()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err((StatusCode::NOT_FOUND, stderr.to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Validate that a git ref is safe (no special characters that could be used for injection)
fn is_safe_git_ref(git_ref: &str) -> bool {
    // Allow alphanumeric, dash, underscore, dot, forward slash, and caret/tilde for refs like HEAD~1
    git_ref.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/' || c == '^' || c == '~'
    }) && !git_ref.is_empty() && git_ref.len() < 256
}

/// Run the HTTP server
pub async fn run_http_server(
    config: Arc<Config>,
    db: Arc<Database>,
    repos_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = AppState {
        config: config.clone(),
        db,
        repos_path,
    };

    let app = create_router(state);
    let addr = format!("0.0.0.0:{}", config.http_port);

    println!("HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
