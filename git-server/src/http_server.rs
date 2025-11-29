use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, Method, StatusCode, Request},
    middleware::{self, Next},
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use tokio::process::Command;
use tokio::fs;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::info;

use crate::config::Config;
use crate::database::Database;
use crate::error::AppError;

/// Shared state for the HTTP server
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<Database>,
    pub repos_path: PathBuf,
}

/// Organization info for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationInfo {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub created_at: String,
}

/// Repository info for API responses
#[derive(Debug, Serialize)]
pub struct RepoInfo {
    pub name: String,
    pub org_name: String,
    pub project_name: String,
    pub path: String,
    pub forked_from: Option<String>,
}

/// Project info for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: i64,
    pub name: String,
    pub org_name: String,
    pub display_name: String,
    pub description: String,
    pub created_at: String,
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

/// Request body for creating a repository
#[derive(Debug, Deserialize)]
pub struct CreateRepoRequest {
    pub name: String,
}

/// Request body for creating an organization
#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
}

/// Request body for updating an organization
#[derive(Debug, Deserialize)]
pub struct UpdateOrgRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
}

/// Request body for creating a project
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
}

/// Request body for updating a project
#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
}

/// Request body for updating a file
#[derive(Debug, Deserialize)]
pub struct UpdateFileRequest {
    pub path: String,
    pub content: String,
    pub message: String,
}

/// Request body for deleting a file
#[derive(Debug, Deserialize)]
pub struct DeleteFileRequest {
    pub path: String,
    pub message: String,
}

/// Issue info for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct IssueInfo {
    pub id: i64,
    pub repo_name: String,
    pub number: i64,
    pub title: String,
    pub body: String,
    pub state: String,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Issue comment for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct IssueCommentInfo {
    pub id: i64,
    pub issue_id: i64,
    pub body: String,
    pub author: String,
    pub created_at: String,
}

/// Pull request info for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct PullRequestInfo {
    pub id: i64,
    pub repo_name: String,
    pub number: i64,
    pub title: String,
    pub body: String,
    pub state: String,
    pub source_repo: String,
    pub source_branch: String,
    pub target_branch: String,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Pull request comment for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct PullRequestCommentInfo {
    pub id: i64,
    pub pr_id: i64,
    pub body: String,
    pub author: String,
    pub created_at: String,
}

/// File diff for pull request files
#[derive(Debug, Serialize)]
pub struct FileDiff {
    pub path: String,
    pub status: String,
    pub additions: i32,
    pub deletions: i32,
    pub diff: String,
}

/// Request body for creating an issue
#[derive(Debug, Deserialize)]
pub struct CreateIssueRequest {
    pub title: String,
    pub body: String,
}

/// Request body for updating an issue
#[derive(Debug, Deserialize)]
pub struct UpdateIssueRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub state: Option<String>,
}

/// Request body for creating a comment
#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub body: String,
}

/// Request body for creating a pull request
#[derive(Debug, Deserialize)]
pub struct CreatePullRequestRequest {
    pub title: String,
    pub body: String,
    pub source_repo: String,
    pub source_branch: String,
    pub target_branch: String,
}

/// Request body for updating a pull request
#[derive(Debug, Deserialize)]
pub struct UpdatePullRequestRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub state: Option<String>,
}

/// Create the HTTP router
pub fn create_router(state: AppState) -> Router {
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    let api_routes = Router::new()
        // Organization routes
        .route("/orgs", get(list_orgs).post(create_org))
        .route("/orgs/:org", get(get_org).patch(update_org))
        // Project routes (a project now directly contains a git repository with same name)
        .route("/orgs/:org/projects", get(list_projects).post(create_project))
        .route("/orgs/:org/projects/:project", get(get_project).patch(update_project))
        // Project git operations (project = repository, using project name as repo name)
        .route("/orgs/:org/projects/:project/files", get(project_list_files).post(project_update_file).delete(project_delete_file))
        .route("/orgs/:org/projects/:project/commits", get(project_list_commits))
        .route("/orgs/:org/projects/:project/tree", get(project_get_tree))
        .route("/orgs/:org/projects/:project/blob", get(project_get_blob))
        .route("/orgs/:org/projects/:project/branches", get(project_list_branches))
        .route("/orgs/:org/projects/:project/fork", axum::routing::post(project_fork))
        // Project issue routes
        .route("/orgs/:org/projects/:project/issues", get(project_list_issues).post(project_create_issue))
        .route("/orgs/:org/projects/:project/issues/:number", get(project_get_issue).patch(project_update_issue))
        .route("/orgs/:org/projects/:project/issues/:number/comments", get(project_list_issue_comments).post(project_create_issue_comment))
        // Project pull request routes
        .route("/orgs/:org/projects/:project/pulls", get(project_list_pull_requests).post(project_create_pull_request))
        .route("/orgs/:org/projects/:project/pulls/:number", get(project_get_pull_request).patch(project_update_pull_request))
        .route("/orgs/:org/projects/:project/pulls/:number/comments", get(project_list_pr_comments).post(project_create_pr_comment))
        .route("/orgs/:org/projects/:project/pulls/:number/commits", get(project_get_pr_commits))
        .route("/orgs/:org/projects/:project/pulls/:number/files", get(project_get_pr_files))
        // Legacy repository routes (kept for backward compatibility)
        .route("/orgs/:org/projects/:project/repos", get(list_repos).post(create_repo))
        .route("/orgs/:org/projects/:project/repos/:name", get(get_repo))
        .route("/orgs/:org/projects/:project/repos/:name/files", get(list_files).post(update_file).delete(delete_file))
        .route("/orgs/:org/projects/:project/repos/:name/commits", get(list_commits))
        .route("/orgs/:org/projects/:project/repos/:name/tree", get(get_tree))
        .route("/orgs/:org/projects/:project/repos/:name/blob", get(get_blob_root))
        .route("/orgs/:org/projects/:project/repos/:name/branches", get(list_branches))
        .route("/orgs/:org/projects/:project/repos/:name/fork", axum::routing::post(fork_repo))
        .route("/orgs/:org/projects/:project/repos/:name/issues", get(list_issues).post(create_issue))
        .route("/orgs/:org/projects/:project/repos/:name/issues/:number", get(get_issue).patch(update_issue))
        .route("/orgs/:org/projects/:project/repos/:name/issues/:number/comments", get(list_issue_comments).post(create_issue_comment))
        .route("/orgs/:org/projects/:project/repos/:name/pulls", get(list_pull_requests).post(create_pull_request))
        .route("/orgs/:org/projects/:project/repos/:name/pulls/:number", get(get_pull_request).patch(update_pull_request))
        .route("/orgs/:org/projects/:project/repos/:name/pulls/:number/comments", get(list_pr_comments).post(create_pr_comment))
        .route("/orgs/:org/projects/:project/repos/:name/pulls/:number/commits", get(get_pr_commits))
        .route("/orgs/:org/projects/:project/repos/:name/pulls/:number/files", get(get_pr_files))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Git HTTP Smart Protocol routes - now just /:org/:project.git (project = repo)
    let git_routes = Router::new()
        .route("/:org/:project.git/info/refs", get(project_git_info_refs))
        .route("/:org/:project.git/git-upload-pack", axum::routing::post(project_git_upload_pack))
        .route("/:org/:project.git/git-receive-pack", axum::routing::post(project_git_receive_pack))
        // Legacy: keep old routes for backward compatibility
        .route("/:org/:project/:name.git/info/refs", get(git_info_refs))
        .route("/:org/:project/:name.git/git-upload-pack", axum::routing::post(git_upload_pack))
        .route("/:org/:project/:name.git/git-receive-pack", axum::routing::post(git_receive_pack))
        .with_state(state.clone());

    // Try to find static directory - check multiple locations
    let static_dir = find_static_dir();

    let app = Router::new()
        .nest("/api", api_routes)
        .merge(git_routes)
        .fallback_service(ServeDir::new(static_dir).fallback(get(serve_index)))
        .layer(cors)
        .with_state(state);

    app
}

/// Find the static directory by checking multiple locations
fn find_static_dir() -> PathBuf {
    // Try current dir/static/dist (Vite build output)
    if let Ok(cwd) = std::env::current_dir() {
        let dir = cwd.join("static").join("dist");
        if dir.exists() {
            return dir;
        }
        // Try current dir/git-server/static/dist
        let dir = cwd.join("git-server").join("static").join("dist");
        if dir.exists() {
            return dir;
        }
        // Fallback to static (for old structure)
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
            let dir = exe_dir.join("static").join("dist");
            if dir.exists() {
                return dir;
            }
            let dir = exe_dir.join("static");
            if dir.exists() {
                return dir;
            }
        }
    }

    // Fall back to "static/dist" relative path
    PathBuf::from("static/dist")
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
    let static_dir = find_static_dir();
    let index_path = static_dir.join("index.html");

    match fs::read_to_string(&index_path).await {
        Ok(content) => Html(content).into_response(),
        Err(_) => {
            // Fallback to embedded old index.html
            AppError::NotFound("route not found".to_string()).into_response()
        }
    }
}

// ============ Organization Handlers ============

/// List all organizations
async fn list_orgs(State(state): State<AppState>) -> Result<Json<Vec<OrganizationInfo>>, AppError> {
    let orgs = state.db.list_organizations().await?;
    Ok(Json(orgs))
}

/// Create a new organization
async fn create_org(
    State(state): State<AppState>,
    Json(body): Json<CreateOrgRequest>,
) -> Result<Json<OrganizationInfo>, AppError> {
    let name = body.name.trim();
    if name.is_empty() {
        return Err(AppError::bad_request("Organization name is required"));
    }

    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(AppError::bad_request("Organization name contains invalid characters"));
    }

    // Check if already exists
    let existing = state.db.get_organization(name).await?;

    if existing.is_some() {
        return Err(AppError::conflict("Organization already exists"));
    }

    // Create organization directory
    let org_path = state.repos_path.join(name);
    fs::create_dir_all(&org_path).await?;

    let display_name = if body.display_name.trim().is_empty() {
        name.to_string()
    } else {
        body.display_name.trim().to_string()
    };

    let org = state.db
        .create_organization(name, &display_name, &body.description)
        .await?;

    Ok(Json(org))
}

/// Get a single organization
async fn get_org(
    State(state): State<AppState>,
    Path(org): Path<String>,
) -> Result<Json<OrganizationInfo>, AppError> {
    let org = state
        .db
        .get_organization(&org)
        .await?
        .ok_or_else(|| AppError::not_found("Organization not found"))?;

    Ok(Json(org))
}

/// Update an organization
async fn update_org(
    State(state): State<AppState>,
    Path(org_name): Path<String>,
    Json(body): Json<UpdateOrgRequest>,
) -> Result<Json<OrganizationInfo>, AppError> {
    let org = state.db
        .update_organization(&org_name, body.display_name.as_deref(), body.description.as_deref())
        .await?
        .ok_or_else(|| AppError::not_found("Organization not found"))?;

    Ok(Json(org))
}

// ============ Project Handlers ============

/// List all projects in an organization
async fn list_projects(
    State(state): State<AppState>,
    Path(org): Path<String>,
) -> Result<Json<Vec<ProjectInfo>>, AppError> {
    // Verify org exists
    state.db
        .get_organization(&org)
        .await?
        .ok_or_else(|| AppError::not_found("Organization not found"))?;

    let projects = state.db.list_projects(&org).await?;

    Ok(Json(projects))
}

/// Create a new project (also creates a git repository for it)
async fn create_project(
    State(state): State<AppState>,
    Path(org): Path<String>,
    Json(body): Json<CreateProjectRequest>,
) -> Result<Json<ProjectInfo>, AppError> {
    // Verify org exists
    state.db
        .get_organization(&org)
        .await?
        .ok_or_else(|| AppError::not_found("Organization not found"))?;

    let name = body.name.trim();
    if name.is_empty() {
        return Err(AppError::bad_request("Project name is required"));
    }

    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(AppError::bad_request("Project name contains invalid characters"));
    }

    // Check if already exists
    let existing = state.db.get_project(&org, name).await?;

    if existing.is_some() {
        return Err(AppError::conflict("Project already exists"));
    }

    // Create project directory
    let project_path = state.repos_path.join(&org).join(name);
    fs::create_dir_all(&project_path).await?;

    let display_name = if body.display_name.trim().is_empty() {
        name.to_string()
    } else {
        body.display_name.trim().to_string()
    };

    let project = state.db
        .create_project(&org, name, &display_name, &body.description)
        .await?;

    // Create the git repository for this project (project = 1 repo)
    // The repo name is the same as the project name
    let repo_dir_name = format!("{}.git", name);
    let repo_path = state.repos_path.join(&org).join(name).join(&repo_dir_name);

    // Initialize bare git repository with main as default branch
    let output = Command::new("git")
        .args(["init", "--bare", "--initial-branch=main"])
        .arg(&repo_path)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(format!("Failed to create repository: {}", stderr)));
    }

    // Add to database - store relative path from repos root
    let relative_path = format!("{}/{}/{}", org, name, repo_dir_name);
    state.db
        .create_repository(&org, name, name, &relative_path)
        .await?;

    Ok(Json(project))
}

/// Get a single project
async fn get_project(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
) -> Result<Json<ProjectInfo>, AppError> {
    let project = state
        .db
        .get_project(&org, &project)
        .await?
        .ok_or_else(|| AppError::not_found("Project not found"))?;

    Ok(Json(project))
}

/// Update a project
async fn update_project(
    State(state): State<AppState>,
    Path((org, project_name)): Path<(String, String)>,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectInfo>, AppError> {
    let project = state.db
        .update_project(&org, &project_name, body.display_name.as_deref(), body.description.as_deref())
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    Ok(Json(project))
}

// ============ Repository Handlers ============

/// List all repositories in a project
async fn list_repos(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
) -> Result<Json<Vec<RepoInfo>>, AppError> {
    // Verify project exists
    state.db
        .get_project(&org, &project)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repos = state
        .db
        .list_repositories(&org, &project)
        .await
        ?;

    let repo_infos: Vec<RepoInfo> = repos
        .into_iter()
        .map(|r| RepoInfo {
            name: r.name,
            org_name: r.org_name,
            project_name: r.project_name.unwrap_or_default(),
            path: r.path,
            forked_from: r.forked_from,
        })
        .collect();

    Ok(Json(repo_infos))
}

/// Create a new repository in a project
async fn create_repo(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
    Json(body): Json<CreateRepoRequest>,
) -> Result<Json<RepoInfo>, AppError> {
    // Verify project exists
    state.db
        .get_project(&org, &project)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    // Validate name
    let name = body.name.trim();
    if name.is_empty() {
        return Err(AppError::bad_request("Repository name is required"));
    }

    // Check for invalid characters in name
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err(AppError::bad_request("Repository name contains invalid characters"));
    }

    // Check if already exists
    let existing = state.db
        .get_repository(&org, &project, name)
        .await
        ?;

    if existing.is_some() {
        return Err(AppError::conflict("Repository already exists"));
    }

    // Create the bare repository directory under org/project folder
    let repo_dir_name = if name.ends_with(".git") {
        name.to_string()
    } else {
        format!("{}.git", name)
    };
    let repo_path = state.repos_path.join(&org).join(&project).join(&repo_dir_name);

    // Ensure project directory exists
    fs::create_dir_all(state.repos_path.join(&org).join(&project))
        .await
        ?;

    // Initialize bare git repository with main as default branch
    let output = Command::new("git")
        .args(["init", "--bare", "--initial-branch=main"])
        .arg(&repo_path)
        .output()
        .await
        ?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(format!("Failed to create repository: {}", stderr)));
    }

    // Add to database - store relative path from repos root
    let relative_path = format!("{}/{}/{}", org, project, repo_dir_name);
    state.db
        .create_repository(&org, &project, name, &relative_path)
        .await
        ?;

    Ok(Json(RepoInfo {
        name: name.to_string(),
        org_name: org,
        project_name: project,
        path: relative_path,
        forked_from: None,
    }))
}

/// Get a single repository
async fn get_repo(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
) -> Result<Json<RepoInfo>, AppError> {
    let repo = state
        .db
        .get_repository(&org, &project, &name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    Ok(Json(RepoInfo {
        name: repo.name,
        org_name: repo.org_name,
        project_name: repo.project_name.unwrap_or_default(),
        path: repo.path,
        forked_from: repo.forked_from,
    }))
}

/// List files in a repository (root of default branch)
async fn list_files(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
) -> Result<Json<Vec<FileEntry>>, AppError> {
    let repo = state
        .db
        .get_repository(&org, &project, &name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);
    let files = list_git_files(&repo_path, "HEAD", "").await?;

    Ok(Json(files))
}

/// Update a file in the repository and commit it
async fn update_file(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
    Json(body): Json<UpdateFileRequest>,
) -> Result<(), AppError> {
    // Validate inputs
    if body.path.is_empty() {
        return Err(AppError::bad_request("File path is required"));
    }
    if body.message.is_empty() {
        return Err(AppError::bad_request("Commit message is required"));
    }
    if body.path.contains("..") {
        return Err(AppError::bad_request("Invalid file path"));
    }

    let repo = state
        .db
        .get_repository(&org, &project, &name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);

    // For a bare repository, we need to use a worktree to make changes
    // We'll create a temporary worktree, make the change, commit, and clean up
    let temp_dir = std::env::temp_dir().join(format!("git-server-{}-{}", name, std::process::id()));

    // Clone the bare repo to a temporary directory
    let output = Command::new("git")
        .args(["clone", "--local"])
        .arg(&repo_path)
        .arg(&temp_dir)
        .output()
        .await
        ?;

    // If clone fails (empty repo), init a new repo
    let is_empty_repo = !output.status.success();
    if is_empty_repo {
        fs::create_dir_all(&temp_dir)
            .await
            ?;

        let output = Command::new("git")
            .args(["init"])
            .current_dir(&temp_dir)
            .output()
            .await
            ?;

        if !output.status.success() {
            let _ = fs::remove_dir_all(&temp_dir).await;
            return Err(AppError::internal("Failed to initialize temp repository"));
        }

        // Set remote to push to
        let output = Command::new("git")
            .args(["remote", "add", "origin"])
            .arg(&repo_path)
            .current_dir(&temp_dir)
            .output()
            .await
            ?;

        if !output.status.success() {
            let _ = fs::remove_dir_all(&temp_dir).await;
            return Err(AppError::internal("Failed to set remote"));
        }
    }

    // Write the file
    let file_path = temp_dir.join(&body.path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .await
            ?;
    }
    fs::write(&file_path, &body.content)
        .await
        ?;

    // Configure git user for this commit
    let email_output = Command::new("git")
        .args(["config", "user.email", "webapp@git-server.local"])
        .current_dir(&temp_dir)
        .output()
        .await
        ?;

    if !email_output.status.success() {
        let _ = fs::remove_dir_all(&temp_dir).await;
        return Err(AppError::internal("Failed to configure git user email"));
    }

    let name_output = Command::new("git")
        .args(["config", "user.name", "Git Server Webapp"])
        .current_dir(&temp_dir)
        .output()
        .await
        ?;

    if !name_output.status.success() {
        let _ = fs::remove_dir_all(&temp_dir).await;
        return Err(AppError::internal("Failed to configure git user name"));
    }

    // Add the file
    let output = Command::new("git")
        .args(["add", &body.path])
        .current_dir(&temp_dir)
        .output()
        .await
        ?;

    if !output.status.success() {
        let _ = fs::remove_dir_all(&temp_dir).await;
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(format!("Failed to add file: {}", stderr)));
    }

    // Commit
    let output = Command::new("git")
        .args(["commit", "-m", &body.message])
        .current_dir(&temp_dir)
        .output()
        .await
        ?;

    if !output.status.success() {
        let _ = fs::remove_dir_all(&temp_dir).await;
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(format!("Failed to commit: {}", stderr)));
    }

    // Push to the bare repo
    let output = Command::new("git")
        .args(["push", "origin", "HEAD:main"])
        .current_dir(&temp_dir)
        .output()
        .await
        ?;

    // Try HEAD:master if main fails
    if !output.status.success() {
        let output = Command::new("git")
            .args(["push", "origin", "HEAD:master"])
            .current_dir(&temp_dir)
            .output()
            .await
            ?;

        if !output.status.success() {
            let _ = fs::remove_dir_all(&temp_dir).await;
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::internal(format!("Failed to push: {}", stderr)));
        }
    }

    // Clean up temp directory
    let _ = fs::remove_dir_all(&temp_dir).await;

    Ok(())
}

/// Delete a file in the repository and commit it
async fn delete_file(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
    Json(body): Json<DeleteFileRequest>,
) -> Result<(), AppError> {
    // Validate inputs
    if body.path.is_empty() {
        return Err(AppError::bad_request("File path is required"));
    }
    if body.message.is_empty() {
        return Err(AppError::bad_request("Commit message is required"));
    }
    if body.path.contains("..") {
        return Err(AppError::bad_request("Invalid file path"));
    }

    let repo = state
        .db
        .get_repository(&org, &project, &name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);

    // For a bare repository, we need to use a temporary worktree
    let temp_dir = std::env::temp_dir().join(format!("git-server-{}-{}-{}-{}", org, project, name, std::process::id()));

    // Clone the bare repo to a temporary directory
    let output = Command::new("git")
        .args(["clone", "--local"])
        .arg(&repo_path)
        .arg(&temp_dir)
        .output()
        .await
        ?;

    if !output.status.success() {
        let _ = fs::remove_dir_all(&temp_dir).await;
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(format!("Failed to clone: {}", stderr)));
    }

    // Configure git user for this commit
    let _ = Command::new("git")
        .args(["config", "user.email", "webapp@git-server.local"])
        .current_dir(&temp_dir)
        .output()
        .await;

    let _ = Command::new("git")
        .args(["config", "user.name", "Git Server Webapp"])
        .current_dir(&temp_dir)
        .output()
        .await;

    // Remove the file
    let output = Command::new("git")
        .args(["rm", &body.path])
        .current_dir(&temp_dir)
        .output()
        .await
        ?;

    if !output.status.success() {
        let _ = fs::remove_dir_all(&temp_dir).await;
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(format!("Failed to remove file: {}", stderr)));
    }

    // Commit
    let output = Command::new("git")
        .args(["commit", "-m", &body.message])
        .current_dir(&temp_dir)
        .output()
        .await
        ?;

    if !output.status.success() {
        let _ = fs::remove_dir_all(&temp_dir).await;
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(format!("Failed to commit: {}", stderr)));
    }

    // Push to the bare repo
    let output = Command::new("git")
        .args(["push", "origin", "HEAD:main"])
        .current_dir(&temp_dir)
        .output()
        .await
        ?;

    // Try HEAD:master if main fails
    if !output.status.success() {
        let output = Command::new("git")
            .args(["push", "origin", "HEAD:master"])
            .current_dir(&temp_dir)
            .output()
            .await
            ?;

        if !output.status.success() {
            let _ = fs::remove_dir_all(&temp_dir).await;
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::internal(format!("Failed to push: {}", stderr)));
        }
    }

    // Clean up temp directory
    let _ = fs::remove_dir_all(&temp_dir).await;

    Ok(())
}

/// List commits in a repository
async fn list_commits(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
) -> Result<Json<Vec<CommitInfo>>, AppError> {
    let repo = state
        .db
        .get_repository(&org, &project, &name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);
    let commits = list_git_commits(&repo_path).await?;

    Ok(Json(commits))
}

/// Get tree at a specific ref and path
async fn get_tree(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
    Query(query): Query<TreeQuery>,
) -> Result<Json<Vec<FileEntry>>, AppError> {
    let repo = state
        .db
        .get_repository(&org, &project, &name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);
    let git_ref = query.git_ref.as_deref().unwrap_or("HEAD");
    let path = query.path.as_deref().unwrap_or("");
    let files = list_git_files(&repo_path, git_ref, path).await?;

    Ok(Json(files))
}

/// Get blob content (uses query params for ref and path)
async fn get_blob_root(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
    Query(query): Query<TreeQuery>,
) -> Result<String, AppError> {
    let repo = state
        .db
        .get_repository(&org, &project, &name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);
    let git_ref = query.git_ref.as_deref().unwrap_or("HEAD");
    let path = query.path.as_deref().ok_or_else(|| AppError::bad_request("path query parameter required"))?;
    let content = get_git_blob(&repo_path, git_ref, path).await?;

    Ok(content)
}

/// List files in a git repository at a specific ref and path
async fn list_git_files(
    repo_path: &PathBuf,
    git_ref: &str,
    path: &str,
) -> Result<Vec<FileEntry>, AppError> {
    // Validate ref to prevent command injection
    if !is_safe_git_ref(git_ref) {
        return Err(AppError::bad_request("Invalid git ref"));
    }

    // Validate path to prevent path traversal
    if path.contains("..") {
        return Err(AppError::bad_request("Invalid path"));
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
        ?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Not a valid object") || stderr.contains("fatal:") {
            return Ok(vec![]);
        }
        return Err(AppError::internal(stderr.to_string()));
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
async fn list_git_commits(repo_path: &PathBuf) -> Result<Vec<CommitInfo>, AppError> {
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
        ?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("does not have any commits") {
            return Ok(vec![]);
        }
        return Err(AppError::internal(stderr.to_string()));
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
) -> Result<String, AppError> {
    // Validate ref to prevent command injection
    if !is_safe_git_ref(git_ref) {
        return Err(AppError::bad_request("Invalid git ref"));
    }

    // Validate path to prevent path traversal
    if path.contains("..") {
        return Err(AppError::bad_request("Invalid path"));
    }

    let blob_path = format!("{}:{}", git_ref, path);

    let output = Command::new("git")
        .args(["show", &blob_path])
        .current_dir(repo_path)
        .output()
        .await
        ?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::not_found(stderr.to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Validate that a git ref is safe (no special characters that could be used for injection)
fn is_safe_git_ref(git_ref: &str) -> bool {
    if git_ref.is_empty() || git_ref.len() > 255 {
        return false;
    }

    // Check for invalid patterns
    if git_ref.starts_with('-') || git_ref.starts_with('.') {
        return false;
    }

    // Don't allow consecutive special characters that could be malformed
    if git_ref.contains("..") || git_ref.contains("//") {
        return false;
    }

    // Allow alphanumeric, dash, underscore, dot, forward slash
    // Allow caret and tilde only when followed by digits (e.g., HEAD~1, HEAD^2)
    let mut chars = git_ref.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/' {
            continue;
        }
        if c == '^' || c == '~' {
            // Must be followed by nothing or digits
            match chars.peek() {
                None => continue,
                Some(next) if next.is_ascii_digit() => continue,
                Some(next) if *next == '^' || *next == '~' => {
                    // Allow chained like HEAD^^
                    continue;
                }
                _ => return false,
            }
        }
        return false;
    }

    true
}

/// List branches in a repository
async fn list_branches(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
) -> Result<Json<Vec<String>>, AppError> {
    let repo = state
        .db
        .get_repository(&org, &project, &name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);

    let output = Command::new("git")
        .args(["branch", "--list", "--format=%(refname:short)"])
        .current_dir(&repo_path)
        .output()
        .await
        ?;

    if !output.status.success() {
        return Ok(Json(vec![]));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();

    Ok(Json(branches))
}

/// Fork request body with optional target org and project
#[derive(Debug, Deserialize)]
pub struct ForkRepoRequest {
    pub name: String,
    pub target_org: Option<String>,
    pub target_project: Option<String>,
}

/// Fork a repository
async fn fork_repo(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
    Json(body): Json<ForkRepoRequest>,
) -> Result<Json<RepoInfo>, AppError> {
    // Get the source repository
    let source_repo = state
        .db
        .get_repository(&org, &project, &name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let new_name = body.name.trim();
    if new_name.is_empty() {
        return Err(AppError::bad_request("New repository name is required"));
    }

    if !new_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err(AppError::bad_request("Repository name contains invalid characters"));
    }

    // Determine target org and project (default to source org and project)
    let target_org = body.target_org.as_deref().unwrap_or(&org);
    let target_project = body.target_project.as_deref().unwrap_or(&project);

    // Verify target project exists
    state.db
        .get_project(target_org, target_project)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    // Check if already exists
    let existing = state.db
        .get_repository(target_org, target_project, new_name)
        .await
        ?;

    if existing.is_some() {
        return Err(AppError::conflict("Repository already exists"));
    }

    // Create the new repository directory
    let new_repo_dir_name = if new_name.ends_with(".git") {
        new_name.to_string()
    } else {
        format!("{}.git", new_name)
    };
    let new_repo_path = state.repos_path.join(target_org).join(target_project).join(&new_repo_dir_name);
    let source_repo_path = state.repos_path.join(&source_repo.path);

    // Ensure target project directory exists
    fs::create_dir_all(state.repos_path.join(target_org).join(target_project))
        .await
        ?;

    // Clone the repository
    let output = Command::new("git")
        .args(["clone", "--bare"])
        .arg(&source_repo_path)
        .arg(&new_repo_path)
        .output()
        .await
        ?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(format!("Failed to fork repository: {}", stderr)));
    }

    // Add to database with fork info
    let forked_from = format!("{}/{}/{}", org, project, name);
    let relative_path = format!("{}/{}/{}", target_org, target_project, new_repo_dir_name);
    state.db
        .create_repository_with_fork(target_org, target_project, new_name, &relative_path, &forked_from)
        .await
        ?;

    Ok(Json(RepoInfo {
        name: new_name.to_string(),
        org_name: target_org.to_string(),
        project_name: target_project.to_string(),
        path: relative_path,
        forked_from: Some(forked_from),
    }))
}

// ============ Issue Handlers ============

/// List issues for a repository
async fn list_issues(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
) -> Result<Json<Vec<IssueInfo>>, AppError> {
    let _repo = state
        .db
        .get_repository(&org, &project, &name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let full_name = format!("{}/{}/{}", org, project, name);
    let issues = state.db
        .list_issues(&full_name)
        .await
        ?;

    Ok(Json(issues))
}

/// Get a single issue
async fn get_issue(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
) -> Result<Json<IssueInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let issue = state.db
        .get_issue(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    Ok(Json(issue))
}

/// Create a new issue
async fn create_issue(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
    Json(body): Json<CreateIssueRequest>,
) -> Result<Json<IssueInfo>, AppError> {
    let _repo = state
        .db
        .get_repository(&org, &project, &name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    if body.title.trim().is_empty() {
        return Err(AppError::bad_request("Issue title is required"));
    }

    let full_name = format!("{}/{}/{}", org, project, name);
    let issue = state.db
        .create_issue(&full_name, &body.title, &body.body, "anonymous")
        .await
        ?;

    Ok(Json(issue))
}

/// Update an issue
async fn update_issue(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
    Json(body): Json<UpdateIssueRequest>,
) -> Result<Json<IssueInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let issue = state.db
        .update_issue(&full_name, number, body.title.as_deref(), body.body.as_deref(), body.state.as_deref())
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    Ok(Json(issue))
}

/// List comments for an issue
async fn list_issue_comments(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
) -> Result<Json<Vec<IssueCommentInfo>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let issue = state.db
        .get_issue(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let comments = state.db
        .list_issue_comments(issue.id)
        .await
        ?;

    Ok(Json(comments))
}

/// Create a comment on an issue
async fn create_issue_comment(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
    Json(body): Json<CreateCommentRequest>,
) -> Result<Json<IssueCommentInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let issue = state.db
        .get_issue(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    if body.body.trim().is_empty() {
        return Err(AppError::bad_request("Comment body is required"));
    }

    let comment = state.db
        .create_issue_comment(issue.id, &body.body, "anonymous")
        .await
        ?;

    Ok(Json(comment))
}

// ============ Pull Request Handlers ============

/// List pull requests for a repository
async fn list_pull_requests(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
) -> Result<Json<Vec<PullRequestInfo>>, AppError> {
    let _repo = state
        .db
        .get_repository(&org, &project, &name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let full_name = format!("{}/{}/{}", org, project, name);
    let prs = state.db
        .list_pull_requests(&full_name)
        .await
        ?;

    Ok(Json(prs))
}

/// Get a single pull request
async fn get_pull_request(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
) -> Result<Json<PullRequestInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let pr = state.db
        .get_pull_request(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    Ok(Json(pr))
}

/// Create a new pull request
async fn create_pull_request(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
    Json(body): Json<CreatePullRequestRequest>,
) -> Result<Json<PullRequestInfo>, AppError> {
    let _repo = state
        .db
        .get_repository(&org, &project, &name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    if body.title.trim().is_empty() {
        return Err(AppError::bad_request("Pull request title is required"));
    }

    let full_name = format!("{}/{}/{}", org, project, name);
    let pr = state.db
        .create_pull_request(
            &full_name,
            &body.title,
            &body.body,
            &body.source_repo,
            &body.source_branch,
            &body.target_branch,
            "anonymous",
        )
        .await
        ?;

    Ok(Json(pr))
}

/// Update a pull request
async fn update_pull_request(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
    Json(body): Json<UpdatePullRequestRequest>,
) -> Result<Json<PullRequestInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let pr = state.db
        .update_pull_request(&full_name, number, body.title.as_deref(), body.body.as_deref(), body.state.as_deref())
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    Ok(Json(pr))
}

/// List comments for a pull request
async fn list_pr_comments(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
) -> Result<Json<Vec<PullRequestCommentInfo>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let pr = state.db
        .get_pull_request(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let comments = state.db
        .list_pr_comments(pr.id)
        .await
        ?;

    Ok(Json(comments))
}

/// Create a comment on a pull request
async fn create_pr_comment(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
    Json(body): Json<CreateCommentRequest>,
) -> Result<Json<PullRequestCommentInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let pr = state.db
        .get_pull_request(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    if body.body.trim().is_empty() {
        return Err(AppError::bad_request("Comment body is required"));
    }

    let comment = state.db
        .create_pr_comment(pr.id, &body.body, "anonymous")
        .await
        ?;

    Ok(Json(comment))
}

/// Get commits for a pull request
async fn get_pr_commits(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
) -> Result<Json<Vec<CommitInfo>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let pr = state.db
        .get_pull_request(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    // Get source repo - source_repo is in org/project/name format
    let source_parts: Vec<&str> = pr.source_repo.split('/').collect();
    let (source_org, source_project, source_name) = if source_parts.len() == 3 {
        (source_parts[0], source_parts[1], source_parts[2])
    } else {
        return Err(AppError::internal("Invalid source repo format"));
    };

    let source_repo = state.db
        .get_repository(source_org, source_project, source_name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&source_repo.path);

    // Get commits on the source branch that aren't in the target branch
    let output = Command::new("git")
        .args([
            "log",
            "--format=%H%n%h%n%an%n%ai%n%s%n---",
            &format!("{}..{}", pr.target_branch, pr.source_branch),
        ])
        .current_dir(&repo_path)
        .output()
        .await
        ?;

    if !output.status.success() {
        // If the comparison fails, just get commits from source branch
        let output = Command::new("git")
            .args([
                "log",
                "--format=%H%n%h%n%an%n%ai%n%s%n---",
                "-n",
                "50",
                &pr.source_branch,
            ])
            .current_dir(&repo_path)
            .output()
            .await
            ?;

        if !output.status.success() {
            return Ok(Json(vec![]));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let commits = parse_commits(&stdout);
        return Ok(Json(commits));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits = parse_commits(&stdout);

    Ok(Json(commits))
}

/// Parse commits from git log output
fn parse_commits(stdout: &str) -> Vec<CommitInfo> {
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
    commits
}

/// Get files changed in a pull request
async fn get_pr_files(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
) -> Result<Json<Vec<FileDiff>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let pr = state.db
        .get_pull_request(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    // Get target repo
    let target_repo = state.db
        .get_repository(&org, &project, &name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    // Get source repo - source_repo is in org/project/name format
    let source_parts: Vec<&str> = pr.source_repo.split('/').collect();
    let (source_org, source_project, source_name) = if source_parts.len() == 3 {
        (source_parts[0], source_parts[1], source_parts[2])
    } else {
        return Err(AppError::internal("Invalid source repo format"));
    };

    let source_repo = state.db
        .get_repository(source_org, source_project, source_name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let source_repo_path = state.repos_path.join(&source_repo.path);
    let target_repo_path = state.repos_path.join(&target_repo.path);

    // If same repo, do a simple diff
    if source_repo.org_name == target_repo.org_name && source_repo.project_name == target_repo.project_name && source_repo.name == target_repo.name {
        return get_branch_diff(&source_repo_path, &pr.target_branch, &pr.source_branch).await;
    }

    // For cross-repo PRs, we need to fetch the target and compare
    // First, add the target as a remote if not already done
    let _ = Command::new("git")
        .args(["remote", "add", "target-for-pr"])
        .arg(&target_repo_path)
        .current_dir(&source_repo_path)
        .output()
        .await;

    // Fetch from target
    let _ = Command::new("git")
        .args(["fetch", "target-for-pr", &pr.target_branch])
        .current_dir(&source_repo_path)
        .output()
        .await;

    let target_ref = format!("target-for-pr/{}", pr.target_branch);
    get_branch_diff(&source_repo_path, &target_ref, &pr.source_branch).await
}

/// Get diff between two branches
async fn get_branch_diff(
    repo_path: &PathBuf,
    base: &str,
    head: &str,
) -> Result<Json<Vec<FileDiff>>, AppError> {
    // Get list of changed files with stats
    let output = Command::new("git")
        .args(["diff", "--numstat", base, head])
        .current_dir(repo_path)
        .output()
        .await
        ?;

    if !output.status.success() {
        return Ok(Json(vec![]));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let additions: i32 = parts[0].parse().unwrap_or(0);
            let deletions: i32 = parts[1].parse().unwrap_or(0);
            let path = parts[2].to_string();

            // Get the actual diff for this file
            let diff_output = Command::new("git")
                .args(["diff", base, head, "--", &path])
                .current_dir(repo_path)
                .output()
                .await
                ?;

            let diff = String::from_utf8_lossy(&diff_output.stdout).to_string();

            let status = if additions > 0 && deletions == 0 {
                "added"
            } else if additions == 0 && deletions > 0 {
                "deleted"
            } else {
                "modified"
            };

            files.push(FileDiff {
                path,
                status: status.to_string(),
                additions,
                deletions,
                diff,
            });
        }
    }

    Ok(Json(files))
}

/// Query parameters for git info/refs
#[derive(Debug, Deserialize)]
pub struct GitInfoRefsQuery {
    service: Option<String>,
}

/// Get git info/refs for HTTP smart protocol
async fn git_info_refs(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
    Query(query): Query<GitInfoRefsQuery>,
) -> Result<Response, AppError> {
    let service = query.service.as_deref().unwrap_or("git-upload-pack");

    // Remove .git suffix if present
    let repo_name = name.strip_suffix(".git").unwrap_or(&name);

    // Get repository
    let repo = state.db
        .get_repository(&org, &project, repo_name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);

    // Service name comes with git- prefix, but the git command needs it without
    let git_cmd = service.strip_prefix("git-").unwrap_or(service);

    // Run git command
    let output = Command::new("git")
        .args([git_cmd, "--stateless-rpc", "--advertise-refs", "."])
        .current_dir(&repo_path)
        .output()
        .await
        ?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(stderr.to_string()));
    }

    // Build response with proper git smart protocol format
    let pkt_line = format!("# service={}\n", service);
    let pkt_len = format!("{:04x}", pkt_line.len() + 4);

    let mut body = Vec::new();
    body.extend_from_slice(pkt_len.as_bytes());
    body.extend_from_slice(pkt_line.as_bytes());
    body.extend_from_slice(b"0000"); // flush packet
    body.extend_from_slice(&output.stdout);

    let content_type = format!("application/x-{}-advertisement", service);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header("Cache-Control", "no-cache")
        .body(Body::from(body))
        .unwrap())
}

/// Handle git-upload-pack (for git fetch/clone)
async fn git_upload_pack(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
    body: axum::body::Bytes,
) -> Result<Response, AppError> {
    // Remove .git suffix if present
    let repo_name = name.strip_suffix(".git").unwrap_or(&name);

    // Get repository
    let repo = state.db
        .get_repository(&org, &project, repo_name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);

    // Create a child process
    let mut child = tokio::process::Command::new("git")
        .args(["upload-pack", "--stateless-rpc", "."])
        .current_dir(&repo_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        ?;

    // Write request body to stdin
    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(&body).await?;
    }

    // Read output
    let output = child.wait_with_output().await
        ?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(stderr.to_string()));
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-git-upload-pack-result")
        .header("Cache-Control", "no-cache")
        .body(Body::from(output.stdout))
        .unwrap())
}

/// Handle git-receive-pack (for git push)
async fn git_receive_pack(
    State(state): State<AppState>,
    Path((org, project, name)): Path<(String, String, String)>,
    body: axum::body::Bytes,
) -> Result<Response, AppError> {
    // Remove .git suffix if present
    let repo_name = name.strip_suffix(".git").unwrap_or(&name);

    // Get repository
    let repo = state.db
        .get_repository(&org, &project, repo_name)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);

    // Create a child process
    let mut child = tokio::process::Command::new("git")
        .args(["receive-pack", "--stateless-rpc", "."])
        .current_dir(&repo_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        ?;

    // Write request body to stdin
    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(&body).await?;
    }

    // Read output
    let output = child.wait_with_output().await
        ?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(stderr.to_string()));
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-git-receive-pack-result")
        .header("Cache-Control", "no-cache")
        .body(Body::from(output.stdout))
        .unwrap())
}

// ============ Project-level Git Handlers (project = repository) ============

/// Helper to get the repository for a project (project name = repo name)
async fn get_project_repo(
    state: &AppState,
    org: &str,
    project: &str,
) -> Result<crate::database::Repository, AppError> {
    // In the new model, project name = repo name
    state.db
        .get_repository(org, project, project)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))
}

/// List files in a project's repository
async fn project_list_files(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
) -> Result<Json<Vec<FileEntry>>, AppError> {
    let repo = get_project_repo(&state, &org, &project).await?;
    let repo_path = state.repos_path.join(&repo.path);
    let files = list_git_files(&repo_path, "HEAD", "").await?;
    Ok(Json(files))
}

/// Update a file in the project's repository
async fn project_update_file(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
    Json(body): Json<UpdateFileRequest>,
) -> Result<(), AppError> {
    // Call the existing update_file logic
    let repo = get_project_repo(&state, &org, &project).await?;
    update_file_impl(&state, &repo, body).await
}

/// Delete a file in the project's repository
async fn project_delete_file(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
    Json(body): Json<DeleteFileRequest>,
) -> Result<(), AppError> {
    let repo = get_project_repo(&state, &org, &project).await?;
    delete_file_impl(&state, &repo, body).await
}

/// List commits in a project's repository
async fn project_list_commits(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
) -> Result<Json<Vec<CommitInfo>>, AppError> {
    let repo = get_project_repo(&state, &org, &project).await?;
    let repo_path = state.repos_path.join(&repo.path);
    let commits = list_git_commits(&repo_path).await?;
    Ok(Json(commits))
}

/// Get tree at a specific ref and path for a project
async fn project_get_tree(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
    Query(query): Query<TreeQuery>,
) -> Result<Json<Vec<FileEntry>>, AppError> {
    let repo = get_project_repo(&state, &org, &project).await?;
    let repo_path = state.repos_path.join(&repo.path);
    let git_ref = query.git_ref.as_deref().unwrap_or("HEAD");
    let path = query.path.as_deref().unwrap_or("");
    let files = list_git_files(&repo_path, git_ref, path).await?;
    Ok(Json(files))
}

/// Get blob content for a project
async fn project_get_blob(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
    Query(query): Query<TreeQuery>,
) -> Result<String, AppError> {
    let repo = get_project_repo(&state, &org, &project).await?;
    let repo_path = state.repos_path.join(&repo.path);
    let git_ref = query.git_ref.as_deref().unwrap_or("HEAD");
    let path = query.path.as_deref().ok_or_else(|| AppError::bad_request("path query parameter required"))?;
    let content = get_git_blob(&repo_path, git_ref, path).await?;
    Ok(content)
}

/// List branches in a project's repository
async fn project_list_branches(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
) -> Result<Json<Vec<String>>, AppError> {
    let repo = get_project_repo(&state, &org, &project).await?;
    let repo_path = state.repos_path.join(&repo.path);

    let output = Command::new("git")
        .args(["branch", "--list", "--format=%(refname:short)"])
        .current_dir(&repo_path)
        .output()
        .await
        ?;

    if !output.status.success() {
        return Ok(Json(vec![]));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();

    Ok(Json(branches))
}

/// Fork a project's repository
async fn project_fork(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
    Json(body): Json<ForkRepoRequest>,
) -> Result<Json<RepoInfo>, AppError> {
    // Get the source project's repository
    let source_repo = get_project_repo(&state, &org, &project).await?;

    let new_name = body.name.trim();
    if new_name.is_empty() {
        return Err(AppError::bad_request("New project name is required"));
    }

    if !new_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err(AppError::bad_request("Project name contains invalid characters"));
    }

    // Determine target org (default to source org)
    let target_org = body.target_org.as_deref().unwrap_or(&org);
    // For projects, the target_project is the new name
    let target_project = new_name;

    // Create the target project first
    let display_name = new_name.to_string();
    let description = format!("Forked from {}/{}", org, project);

    // Check if target project already exists
    let existing = state.db
        .get_project(target_org, target_project)
        .await
        ?;

    if existing.is_some() {
        return Err(AppError::conflict("Project already exists"));
    }

    // Create project directory
    let project_path = state.repos_path.join(target_org).join(target_project);
    fs::create_dir_all(&project_path)
        .await
        ?;

    // Create the project in database
    state.db
        .create_project(target_org, target_project, &display_name, &description)
        .await
        ?;

    // Create the new repository directory
    let new_repo_dir_name = format!("{}.git", target_project);
    let new_repo_path = state.repos_path.join(target_org).join(target_project).join(&new_repo_dir_name);
    let source_repo_path = state.repos_path.join(&source_repo.path);

    // Clone the repository
    let output = Command::new("git")
        .args(["clone", "--bare"])
        .arg(&source_repo_path)
        .arg(&new_repo_path)
        .output()
        .await
        ?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(format!("Failed to fork repository: {}", stderr)));
    }

    // Add to database with fork info
    let forked_from = format!("{}/{}", org, project);
    let relative_path = format!("{}/{}/{}", target_org, target_project, new_repo_dir_name);
    state.db
        .create_repository_with_fork(target_org, target_project, target_project, &relative_path, &forked_from)
        .await
        ?;

    Ok(Json(RepoInfo {
        name: target_project.to_string(),
        org_name: target_org.to_string(),
        project_name: target_project.to_string(),
        path: relative_path,
        forked_from: Some(forked_from),
    }))
}

// ============ Project-level Issue Handlers ============

/// List issues for a project
async fn project_list_issues(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
) -> Result<Json<Vec<IssueInfo>>, AppError> {
    let _repo = get_project_repo(&state, &org, &project).await?;
    let full_name = format!("{}/{}/{}", org, project, project);
    let issues = state.db
        .list_issues(&full_name)
        .await
        ?;
    Ok(Json(issues))
}

/// Get a single issue for a project
async fn project_get_issue(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
) -> Result<Json<IssueInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let issue = state.db
        .get_issue(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;
    Ok(Json(issue))
}

/// Create a new issue for a project
async fn project_create_issue(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
    Json(body): Json<CreateIssueRequest>,
) -> Result<Json<IssueInfo>, AppError> {
    let _repo = get_project_repo(&state, &org, &project).await?;

    if body.title.trim().is_empty() {
        return Err(AppError::bad_request("Issue title is required"));
    }

    let full_name = format!("{}/{}/{}", org, project, project);
    let issue = state.db
        .create_issue(&full_name, &body.title, &body.body, "anonymous")
        .await
        ?;
    Ok(Json(issue))
}

/// Update an issue for a project
async fn project_update_issue(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
    Json(body): Json<UpdateIssueRequest>,
) -> Result<Json<IssueInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let issue = state.db
        .update_issue(&full_name, number, body.title.as_deref(), body.body.as_deref(), body.state.as_deref())
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;
    Ok(Json(issue))
}

/// List comments for an issue in a project
async fn project_list_issue_comments(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
) -> Result<Json<Vec<IssueCommentInfo>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let issue = state.db
        .get_issue(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let comments = state.db
        .list_issue_comments(issue.id)
        .await
        ?;
    Ok(Json(comments))
}

/// Create a comment on an issue in a project
async fn project_create_issue_comment(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
    Json(body): Json<CreateCommentRequest>,
) -> Result<Json<IssueCommentInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let issue = state.db
        .get_issue(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    if body.body.trim().is_empty() {
        return Err(AppError::bad_request("Comment body is required"));
    }

    let comment = state.db
        .create_issue_comment(issue.id, &body.body, "anonymous")
        .await
        ?;
    Ok(Json(comment))
}

// ============ Project-level Pull Request Handlers ============

/// List pull requests for a project
async fn project_list_pull_requests(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
) -> Result<Json<Vec<PullRequestInfo>>, AppError> {
    let _repo = get_project_repo(&state, &org, &project).await?;
    let full_name = format!("{}/{}/{}", org, project, project);
    let prs = state.db
        .list_pull_requests(&full_name)
        .await
        ?;
    Ok(Json(prs))
}

/// Get a single pull request for a project
async fn project_get_pull_request(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
) -> Result<Json<PullRequestInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let pr = state.db
        .get_pull_request(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;
    Ok(Json(pr))
}

/// Create a new pull request for a project
async fn project_create_pull_request(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
    Json(body): Json<CreatePullRequestRequest>,
) -> Result<Json<PullRequestInfo>, AppError> {
    let _repo = get_project_repo(&state, &org, &project).await?;

    if body.title.trim().is_empty() {
        return Err(AppError::bad_request("Pull request title is required"));
    }

    let full_name = format!("{}/{}/{}", org, project, project);
    let pr = state.db
        .create_pull_request(
            &full_name,
            &body.title,
            &body.body,
            &body.source_repo,
            &body.source_branch,
            &body.target_branch,
            "anonymous",
        )
        .await
        ?;
    Ok(Json(pr))
}

/// Update a pull request for a project
async fn project_update_pull_request(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
    Json(body): Json<UpdatePullRequestRequest>,
) -> Result<Json<PullRequestInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let pr = state.db
        .update_pull_request(&full_name, number, body.title.as_deref(), body.body.as_deref(), body.state.as_deref())
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;
    Ok(Json(pr))
}

/// List comments for a pull request in a project
async fn project_list_pr_comments(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
) -> Result<Json<Vec<PullRequestCommentInfo>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let pr = state.db
        .get_pull_request(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let comments = state.db
        .list_pr_comments(pr.id)
        .await
        ?;
    Ok(Json(comments))
}

/// Create a comment on a pull request in a project
async fn project_create_pr_comment(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
    Json(body): Json<CreateCommentRequest>,
) -> Result<Json<PullRequestCommentInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let pr = state.db
        .get_pull_request(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    if body.body.trim().is_empty() {
        return Err(AppError::bad_request("Comment body is required"));
    }

    let comment = state.db
        .create_pr_comment(pr.id, &body.body, "anonymous")
        .await
        ?;
    Ok(Json(comment))
}

/// Get commits for a pull request in a project
async fn project_get_pr_commits(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
) -> Result<Json<Vec<CommitInfo>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let pr = state.db
        .get_pull_request(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    // Get source repo
    let source_parts: Vec<&str> = pr.source_repo.split('/').collect();
    let (source_org, source_project) = if source_parts.len() >= 2 {
        (source_parts[0], source_parts[1])
    } else {
        return Err(AppError::internal("Invalid source repo format"));
    };

    let source_repo = get_project_repo(&state, source_org, source_project).await?;
    let repo_path = state.repos_path.join(&source_repo.path);

    // Get commits on the source branch
    let output = Command::new("git")
        .args([
            "log",
            "--format=%H%n%h%n%an%n%ai%n%s%n---",
            "-n",
            "50",
            &pr.source_branch,
        ])
        .current_dir(&repo_path)
        .output()
        .await
        ?;

    if !output.status.success() {
        return Ok(Json(vec![]));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits = parse_commits(&stdout);
    Ok(Json(commits))
}

/// Get files changed in a pull request for a project
async fn project_get_pr_files(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
) -> Result<Json<Vec<FileDiff>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let pr = state.db
        .get_pull_request(&full_name, number)
        .await
        ?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    // Get target repo
    let target_repo = get_project_repo(&state, &org, &project).await?;

    // Get source repo
    let source_parts: Vec<&str> = pr.source_repo.split('/').collect();
    let (source_org, source_project) = if source_parts.len() >= 2 {
        (source_parts[0], source_parts[1])
    } else {
        return Err(AppError::internal("Invalid source repo format"));
    };

    let source_repo = get_project_repo(&state, source_org, source_project).await?;

    let source_repo_path = state.repos_path.join(&source_repo.path);
    let target_repo_path = state.repos_path.join(&target_repo.path);

    // If same repo, do a simple diff
    if source_repo.org_name == target_repo.org_name && source_repo.project_name == target_repo.project_name {
        return get_branch_diff(&source_repo_path, &pr.target_branch, &pr.source_branch).await;
    }

    // For cross-repo PRs, fetch and compare
    let _ = Command::new("git")
        .args(["remote", "add", "target-for-pr"])
        .arg(&target_repo_path)
        .current_dir(&source_repo_path)
        .output()
        .await;

    let _ = Command::new("git")
        .args(["fetch", "target-for-pr", &pr.target_branch])
        .current_dir(&source_repo_path)
        .output()
        .await;

    let target_ref = format!("target-for-pr/{}", pr.target_branch);
    get_branch_diff(&source_repo_path, &target_ref, &pr.source_branch).await
}

// ============ Project-level Git HTTP Smart Protocol ============

/// Get git info/refs for HTTP smart protocol (project-level)
async fn project_git_info_refs(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
    Query(query): Query<GitInfoRefsQuery>,
) -> Result<Response, AppError> {
    let service = query.service.as_deref().unwrap_or("git-upload-pack");

    // Remove .git suffix if present
    let project_name = project.strip_suffix(".git").unwrap_or(&project);

    // Get repository
    let repo = get_project_repo(&state, &org, project_name).await?;
    let repo_path = state.repos_path.join(&repo.path);

    let git_cmd = service.strip_prefix("git-").unwrap_or(service);

    let output = Command::new("git")
        .args([git_cmd, "--stateless-rpc", "--advertise-refs", "."])
        .current_dir(&repo_path)
        .output()
        .await
        ?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(stderr.to_string()));
    }

    let pkt_line = format!("# service={}\n", service);
    let pkt_len = format!("{:04x}", pkt_line.len() + 4);

    let mut body = Vec::new();
    body.extend_from_slice(pkt_len.as_bytes());
    body.extend_from_slice(pkt_line.as_bytes());
    body.extend_from_slice(b"0000");
    body.extend_from_slice(&output.stdout);

    let content_type = format!("application/x-{}-advertisement", service);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header("Cache-Control", "no-cache")
        .body(Body::from(body))
        .unwrap())
}

/// Handle git-upload-pack (for git fetch/clone) - project-level
async fn project_git_upload_pack(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
    body: axum::body::Bytes,
) -> Result<Response, AppError> {
    let project_name = project.strip_suffix(".git").unwrap_or(&project);
    let repo = get_project_repo(&state, &org, project_name).await?;
    let repo_path = state.repos_path.join(&repo.path);

    let mut child = tokio::process::Command::new("git")
        .args(["upload-pack", "--stateless-rpc", "."])
        .current_dir(&repo_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        ?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(&body).await?;
    }

    let output = child.wait_with_output().await
        ?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(stderr.to_string()));
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-git-upload-pack-result")
        .header("Cache-Control", "no-cache")
        .body(Body::from(output.stdout))
        .unwrap())
}

/// Handle git-receive-pack (for git push) - project-level
async fn project_git_receive_pack(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
    body: axum::body::Bytes,
) -> Result<Response, AppError> {
    let project_name = project.strip_suffix(".git").unwrap_or(&project);
    let repo = get_project_repo(&state, &org, project_name).await?;
    let repo_path = state.repos_path.join(&repo.path);

    let mut child = tokio::process::Command::new("git")
        .args(["receive-pack", "--stateless-rpc", "."])
        .current_dir(&repo_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        ?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(&body).await?;
    }

    let output = child.wait_with_output().await
        ?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(stderr.to_string()));
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-git-receive-pack-result")
        .header("Cache-Control", "no-cache")
        .body(Body::from(output.stdout))
        .unwrap())
}

// ============ Helper implementations for file operations ============

/// Implementation for update_file that can be called with a Repository
async fn update_file_impl(
    state: &AppState,
    repo: &crate::database::Repository,
    body: UpdateFileRequest,
) -> Result<(), AppError> {
    // Validate inputs
    if body.path.is_empty() {
        return Err(AppError::bad_request("File path is required"));
    }
    if body.message.is_empty() {
        return Err(AppError::bad_request("Commit message is required"));
    }
    if body.path.contains("..") {
        return Err(AppError::bad_request("Invalid file path"));
    }

    let repo_path = state.repos_path.join(&repo.path);

    // For a bare repository, we need to use a worktree to make changes
    let temp_dir = std::env::temp_dir().join(format!("git-server-{}-{}", repo.name, std::process::id()));

    // Clone the bare repo to a temporary directory
    let output = Command::new("git")
        .args(["clone", "--local"])
        .arg(&repo_path)
        .arg(&temp_dir)
        .output()
        .await
        ?;

    // If clone fails (empty repo), init a new repo
    let is_empty_repo = !output.status.success();
    if is_empty_repo {
        fs::create_dir_all(&temp_dir)
            .await
            ?;

        let output = Command::new("git")
            .args(["init"])
            .current_dir(&temp_dir)
            .output()
            .await
            ?;

        if !output.status.success() {
            let _ = fs::remove_dir_all(&temp_dir).await;
            return Err(AppError::internal("Failed to initialize temp repository"));
        }

        let output = Command::new("git")
            .args(["remote", "add", "origin"])
            .arg(&repo_path)
            .current_dir(&temp_dir)
            .output()
            .await
            ?;

        if !output.status.success() {
            let _ = fs::remove_dir_all(&temp_dir).await;
            return Err(AppError::internal("Failed to set remote"));
        }
    }

    // Write the file
    let file_path = temp_dir.join(&body.path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .await
            ?;
    }
    fs::write(&file_path, &body.content)
        .await
        ?;

    // Configure git user
    let _ = Command::new("git")
        .args(["config", "user.email", "webapp@git-server.local"])
        .current_dir(&temp_dir)
        .output()
        .await;

    let _ = Command::new("git")
        .args(["config", "user.name", "Git Server Webapp"])
        .current_dir(&temp_dir)
        .output()
        .await;

    // Add the file
    let output = Command::new("git")
        .args(["add", &body.path])
        .current_dir(&temp_dir)
        .output()
        .await
        ?;

    if !output.status.success() {
        let _ = fs::remove_dir_all(&temp_dir).await;
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(format!("Failed to add file: {}", stderr)));
    }

    // Commit
    let output = Command::new("git")
        .args(["commit", "-m", &body.message])
        .current_dir(&temp_dir)
        .output()
        .await
        ?;

    if !output.status.success() {
        let _ = fs::remove_dir_all(&temp_dir).await;
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(format!("Failed to commit: {}", stderr)));
    }

    // Push to the bare repo
    let output = Command::new("git")
        .args(["push", "origin", "HEAD:main"])
        .current_dir(&temp_dir)
        .output()
        .await
        ?;

    if !output.status.success() {
        let output = Command::new("git")
            .args(["push", "origin", "HEAD:master"])
            .current_dir(&temp_dir)
            .output()
            .await
            ?;

        if !output.status.success() {
            let _ = fs::remove_dir_all(&temp_dir).await;
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::internal(format!("Failed to push: {}", stderr)));
        }
    }

    let _ = fs::remove_dir_all(&temp_dir).await;
    Ok(())
}

/// Implementation for delete_file that can be called with a Repository
async fn delete_file_impl(
    state: &AppState,
    repo: &crate::database::Repository,
    body: DeleteFileRequest,
) -> Result<(), AppError> {
    // Validate inputs
    if body.path.is_empty() {
        return Err(AppError::bad_request("File path is required"));
    }
    if body.message.is_empty() {
        return Err(AppError::bad_request("Commit message is required"));
    }
    if body.path.contains("..") {
        return Err(AppError::bad_request("Invalid file path"));
    }

    let repo_path = state.repos_path.join(&repo.path);

    let temp_dir = std::env::temp_dir().join(format!("git-server-{}-{}", repo.name, std::process::id()));

    let output = Command::new("git")
        .args(["clone", "--local"])
        .arg(&repo_path)
        .arg(&temp_dir)
        .output()
        .await
        ?;

    if !output.status.success() {
        let _ = fs::remove_dir_all(&temp_dir).await;
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(format!("Failed to clone: {}", stderr)));
    }

    let _ = Command::new("git")
        .args(["config", "user.email", "webapp@git-server.local"])
        .current_dir(&temp_dir)
        .output()
        .await;

    let _ = Command::new("git")
        .args(["config", "user.name", "Git Server Webapp"])
        .current_dir(&temp_dir)
        .output()
        .await;

    let output = Command::new("git")
        .args(["rm", &body.path])
        .current_dir(&temp_dir)
        .output()
        .await
        ?;

    if !output.status.success() {
        let _ = fs::remove_dir_all(&temp_dir).await;
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(format!("Failed to remove file: {}", stderr)));
    }

    let output = Command::new("git")
        .args(["commit", "-m", &body.message])
        .current_dir(&temp_dir)
        .output()
        .await
        ?;

    if !output.status.success() {
        let _ = fs::remove_dir_all(&temp_dir).await;
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal(format!("Failed to commit: {}", stderr)));
    }

    let output = Command::new("git")
        .args(["push", "origin", "HEAD:main"])
        .current_dir(&temp_dir)
        .output()
        .await
        ?;

    if !output.status.success() {
        let output = Command::new("git")
            .args(["push", "origin", "HEAD:master"])
            .current_dir(&temp_dir)
            .output()
            .await
            ?;

        if !output.status.success() {
            let _ = fs::remove_dir_all(&temp_dir).await;
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::internal(format!("Failed to push: {}", stderr)));
        }
    }

    let _ = fs::remove_dir_all(&temp_dir).await;
    Ok(())
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

    info!("HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
