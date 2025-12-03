use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, Method, Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use tokio::fs;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::info;

use crate::config::Config;
use crate::database::Database;
use crate::error::AppError;
use crate::git_ops;

/// Shared state for the HTTP server
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<Database>,
    pub repos_path: PathBuf,
}

/// Organization info for API responses
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct IssueInfo {
    pub id: i64,
    pub repo_name: String,
    pub number: i64,
    pub title: String,
    pub body: String,
    pub state: String,
    pub status: String,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Tag info for API responses
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct TagInfo {
    pub id: i64,
    pub repo_name: String,
    pub name: String,
    pub color: String,
}

/// Issue comment for API responses
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct IssueCommentInfo {
    pub id: i64,
    pub issue_id: i64,
    pub body: String,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Pull request info for API responses
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub target_date: Option<String>,
}

/// Request body for updating an issue
#[derive(Debug, Deserialize)]
pub struct UpdateIssueRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub state: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
}

/// Request body for creating a tag
#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    #[serde(default = "default_tag_color")]
    pub color: String,
}

fn default_tag_color() -> String {
    "#6b7280".to_string()
}

/// Request body for updating a tag
#[derive(Debug, Deserialize)]
pub struct UpdateTagRequest {
    pub name: Option<String>,
    pub color: Option<String>,
}

/// Request body for creating a comment
#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub body: String,
}

/// Request body for updating a comment
#[derive(Debug, Deserialize)]
pub struct UpdateCommentRequest {
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
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    let api_routes = Router::new()
        // Organization routes
        .route("/orgs", get(list_orgs).post(create_org))
        .route("/orgs/:org", get(get_org).patch(update_org))
        // Project routes (a project now directly contains a git repository with same name)
        .route(
            "/orgs/:org/projects",
            get(list_projects).post(create_project),
        )
        .route(
            "/orgs/:org/projects/:project",
            get(get_project).patch(update_project),
        )
        // Project git operations (project = repository, using project name as repo name)
        .route(
            "/orgs/:org/projects/:project/files",
            get(project_list_files)
                .post(project_update_file)
                .delete(project_delete_file),
        )
        .route(
            "/orgs/:org/projects/:project/commits",
            get(project_list_commits),
        )
        .route("/orgs/:org/projects/:project/tree", get(project_get_tree))
        .route("/orgs/:org/projects/:project/blob", get(project_get_blob))
        .route(
            "/orgs/:org/projects/:project/branches",
            get(project_list_branches),
        )
        .route(
            "/orgs/:org/projects/:project/fork",
            axum::routing::post(project_fork),
        )
        // Project issue routes
        .route(
            "/orgs/:org/projects/:project/issues",
            get(project_list_issues).post(project_create_issue),
        )
        .route(
            "/orgs/:org/projects/:project/issues/:number",
            get(project_get_issue).patch(project_update_issue),
        )
        .route(
            "/orgs/:org/projects/:project/issues/:number/comments",
            get(project_list_issue_comments).post(project_create_issue_comment),
        )
        .route(
            "/orgs/:org/projects/:project/issues/:number/comments/:comment_id",
            axum::routing::patch(project_update_issue_comment),
        )
        .route(
            "/orgs/:org/projects/:project/issues/:number/tags",
            get(project_get_issue_tags).post(project_add_issue_tag),
        )
        .route(
            "/orgs/:org/projects/:project/issues/:number/tags/:tag_id",
            axum::routing::delete(project_remove_issue_tag),
        )
        // Project tag routes
        .route(
            "/orgs/:org/projects/:project/tags",
            get(project_list_tags).post(project_create_tag),
        )
        .route(
            "/orgs/:org/projects/:project/tags/:tag_id",
            get(project_get_tag)
                .patch(project_update_tag)
                .delete(project_delete_tag),
        )
        // Project pull request routes
        .route(
            "/orgs/:org/projects/:project/pulls",
            get(project_list_pull_requests).post(project_create_pull_request),
        )
        .route(
            "/orgs/:org/projects/:project/pulls/:number",
            get(project_get_pull_request).patch(project_update_pull_request),
        )
        .route(
            "/orgs/:org/projects/:project/pulls/:number/comments",
            get(project_list_pr_comments).post(project_create_pr_comment),
        )
        .route(
            "/orgs/:org/projects/:project/pulls/:number/commits",
            get(project_get_pr_commits),
        )
        .route(
            "/orgs/:org/projects/:project/pulls/:number/files",
            get(project_get_pr_files),
        )
        // Legacy repository routes (kept for backward compatibility)
        .route(
            "/orgs/:org/projects/:project/repos",
            get(list_repos).post(create_repo),
        )
        .route("/orgs/:org/projects/:project/repos/:name", get(get_repo))
        .route(
            "/orgs/:org/projects/:project/repos/:name/files",
            get(list_files).post(update_file).delete(delete_file),
        )
        .route(
            "/orgs/:org/projects/:project/repos/:name/commits",
            get(list_commits),
        )
        .route(
            "/orgs/:org/projects/:project/repos/:name/tree",
            get(get_tree),
        )
        .route(
            "/orgs/:org/projects/:project/repos/:name/blob",
            get(get_blob_root),
        )
        .route(
            "/orgs/:org/projects/:project/repos/:name/branches",
            get(list_branches),
        )
        .route(
            "/orgs/:org/projects/:project/repos/:name/fork",
            axum::routing::post(fork_repo),
        )
        .route(
            "/orgs/:org/projects/:project/repos/:name/issues",
            get(list_issues).post(create_issue),
        )
        .route(
            "/orgs/:org/projects/:project/repos/:name/issues/:number",
            get(get_issue).patch(update_issue),
        )
        .route(
            "/orgs/:org/projects/:project/repos/:name/issues/:number/comments",
            get(list_issue_comments).post(create_issue_comment),
        )
        .route(
            "/orgs/:org/projects/:project/repos/:name/issues/:number/comments/:comment_id",
            axum::routing::patch(update_issue_comment),
        )
        .route(
            "/orgs/:org/projects/:project/repos/:name/pulls",
            get(list_pull_requests).post(create_pull_request),
        )
        .route(
            "/orgs/:org/projects/:project/repos/:name/pulls/:number",
            get(get_pull_request).patch(update_pull_request),
        )
        .route(
            "/orgs/:org/projects/:project/repos/:name/pulls/:number/comments",
            get(list_pr_comments).post(create_pr_comment),
        )
        .route(
            "/orgs/:org/projects/:project/repos/:name/pulls/:number/commits",
            get(get_pr_commits),
        )
        .route(
            "/orgs/:org/projects/:project/repos/:name/pulls/:number/files",
            get(get_pr_files),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Git HTTP Smart Protocol routes - now just /:org/:project.git (project = repo)
    let git_routes = Router::new()
        .route("/:org/:project.git/info/refs", get(project_git_info_refs))
        .route(
            "/:org/:project.git/git-upload-pack",
            axum::routing::post(project_git_upload_pack),
        )
        .route(
            "/:org/:project.git/git-receive-pack",
            axum::routing::post(project_git_receive_pack),
        )
        // Legacy: keep old routes for backward compatibility
        .route("/:org/:project/:name.git/info/refs", get(git_info_refs))
        .route(
            "/:org/:project/:name.git/git-upload-pack",
            axum::routing::post(git_upload_pack),
        )
        .route(
            "/:org/:project/:name.git/git-receive-pack",
            axum::routing::post(git_receive_pack),
        )
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
            if let Ok(decoded) =
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
            {
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

    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::bad_request(
            "Organization name contains invalid characters",
        ));
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

    let org = state
        .db
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
    let org = state
        .db
        .update_organization(
            &org_name,
            body.display_name.as_deref(),
            body.description.as_deref(),
        )
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
    state
        .db
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
    state
        .db
        .get_organization(&org)
        .await?
        .ok_or_else(|| AppError::not_found("Organization not found"))?;

    let name = body.name.trim();
    if name.is_empty() {
        return Err(AppError::bad_request("Project name is required"));
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::bad_request(
            "Project name contains invalid characters",
        ));
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

    let project = state
        .db
        .create_project(&org, name, &display_name, &body.description)
        .await?;

    // Create the git repository for this project (project = 1 repo)
    // The repo name is the same as the project name
    let repo_dir_name = format!("{}.git", name);
    let repo_path = state.repos_path.join(&org).join(name).join(&repo_dir_name);

    // Initialize bare git repository with main as default branch using git2
    git_ops::init_bare_repo(&repo_path, "main")
        .map_err(|e| AppError::internal(format!("Failed to create repository: {}", e)))?;

    // Add to database - store relative path from repos root
    let relative_path = format!("{}/{}/{}", org, name, repo_dir_name);
    state
        .db
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
    let project = state
        .db
        .update_project(
            &org,
            &project_name,
            body.display_name.as_deref(),
            body.description.as_deref(),
        )
        .await?
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
    state
        .db
        .get_project(&org, &project)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repos = state.db.list_repositories(&org, &project).await?;

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
    state
        .db
        .get_project(&org, &project)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    // Validate name
    let name = body.name.trim();
    if name.is_empty() {
        return Err(AppError::bad_request("Repository name is required"));
    }

    // Check for invalid characters in name
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(AppError::bad_request(
            "Repository name contains invalid characters",
        ));
    }

    // Check if already exists
    let existing = state.db.get_repository(&org, &project, name).await?;

    if existing.is_some() {
        return Err(AppError::conflict("Repository already exists"));
    }

    // Create the bare repository directory under org/project folder
    let repo_dir_name = if name.ends_with(".git") {
        name.to_string()
    } else {
        format!("{}.git", name)
    };
    let repo_path = state
        .repos_path
        .join(&org)
        .join(&project)
        .join(&repo_dir_name);

    // Ensure project directory exists
    fs::create_dir_all(state.repos_path.join(&org).join(&project)).await?;

    // Initialize bare git repository with main as default branch using git2
    git_ops::init_bare_repo(&repo_path, "main")
        .map_err(|e| AppError::internal(format!("Failed to create repository: {}", e)))?;

    // Add to database - store relative path from repos root
    let relative_path = format!("{}/{}/{}", org, project, repo_dir_name);
    state
        .db
        .create_repository(&org, &project, name, &relative_path)
        .await?;

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
        .await?
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
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);
    let files = list_git_files(&repo_path, "HEAD", "")?;

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
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);

    // Use git2 to update the file directly in the bare repository
    git_ops::update_file_and_commit(&repo_path, &body.path, &body.content, &body.message)
        .map_err(|e| AppError::internal(format!("Failed to update file: {}", e)))?;

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
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);

    // Use git2 to delete the file directly in the bare repository
    git_ops::delete_file_and_commit(&repo_path, &body.path, &body.message)
        .map_err(|e| AppError::internal(format!("Failed to delete file: {}", e)))?;

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
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);
    let commits = list_git_commits(&repo_path)?;

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
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);
    let git_ref = query.git_ref.as_deref().unwrap_or("HEAD");
    let path = query.path.as_deref().unwrap_or("");
    let files = list_git_files(&repo_path, git_ref, path)?;

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
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);
    let git_ref = query.git_ref.as_deref().unwrap_or("HEAD");
    let path = query
        .path
        .as_deref()
        .ok_or_else(|| AppError::bad_request("path query parameter required"))?;
    let content = get_git_blob(&repo_path, git_ref, path)?;

    Ok(content)
}

/// List files in a git repository at a specific ref and path
fn list_git_files(
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

    // Use git2 to list files
    match git_ops::list_files(repo_path, git_ref, path) {
        Ok(entries) => Ok(entries
            .into_iter()
            .map(|e| FileEntry {
                name: e.name,
                path: e.path,
                entry_type: e.entry_type,
                size: e.size,
            })
            .collect()),
        Err(git_ops::GitError::Git(err)) => {
            // Empty repo or invalid ref
            if err.code() == git2::ErrorCode::NotFound
                || err.code() == git2::ErrorCode::UnbornBranch
            {
                Ok(vec![])
            } else {
                Err(AppError::internal(err.to_string()))
            }
        }
        Err(e) => Err(AppError::internal(e.to_string())),
    }
}

/// List commits in a git repository
fn list_git_commits(repo_path: &PathBuf) -> Result<Vec<CommitInfo>, AppError> {
    match git_ops::list_commits(repo_path, 50) {
        Ok(commits) => Ok(commits
            .into_iter()
            .map(|c| CommitInfo {
                hash: c.hash,
                short_hash: c.short_hash,
                author: c.author,
                date: c.date,
                message: c.message,
            })
            .collect()),
        Err(git_ops::GitError::Git(err)) => {
            // Empty repo
            if err.code() == git2::ErrorCode::NotFound
                || err.code() == git2::ErrorCode::UnbornBranch
            {
                Ok(vec![])
            } else {
                Err(AppError::internal(err.to_string()))
            }
        }
        Err(e) => Err(AppError::internal(e.to_string())),
    }
}

/// Get blob content from git
fn get_git_blob(repo_path: &PathBuf, git_ref: &str, path: &str) -> Result<String, AppError> {
    // Validate ref to prevent command injection
    if !is_safe_git_ref(git_ref) {
        return Err(AppError::bad_request("Invalid git ref"));
    }

    // Validate path to prevent path traversal
    if path.contains("..") {
        return Err(AppError::bad_request("Invalid path"));
    }

    match git_ops::get_blob_content(repo_path, git_ref, path) {
        Ok(content) => Ok(content),
        Err(git_ops::GitError::Git(err)) => Err(AppError::not_found(err.to_string())),
        Err(e) => Err(AppError::not_found(e.to_string())),
    }
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
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);

    match git_ops::list_branches(&repo_path) {
        Ok(branches) => Ok(Json(branches)),
        Err(_) => Ok(Json(vec![])),
    }
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
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let new_name = body.name.trim();
    if new_name.is_empty() {
        return Err(AppError::bad_request("New repository name is required"));
    }

    if !new_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(AppError::bad_request(
            "Repository name contains invalid characters",
        ));
    }

    // Determine target org and project (default to source org and project)
    let target_org = body.target_org.as_deref().unwrap_or(&org);
    let target_project = body.target_project.as_deref().unwrap_or(&project);

    // Verify target project exists
    state
        .db
        .get_project(target_org, target_project)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    // Check if already exists
    let existing = state
        .db
        .get_repository(target_org, target_project, new_name)
        .await?;

    if existing.is_some() {
        return Err(AppError::conflict("Repository already exists"));
    }

    // Create the new repository directory
    let new_repo_dir_name = if new_name.ends_with(".git") {
        new_name.to_string()
    } else {
        format!("{}.git", new_name)
    };
    let new_repo_path = state
        .repos_path
        .join(target_org)
        .join(target_project)
        .join(&new_repo_dir_name);
    let source_repo_path = state.repos_path.join(&source_repo.path);

    // Ensure target project directory exists
    fs::create_dir_all(state.repos_path.join(target_org).join(target_project)).await?;

    // Clone the repository using git2
    git_ops::clone_bare(&source_repo_path, &new_repo_path)
        .map_err(|e| AppError::internal(format!("Failed to fork repository: {}", e)))?;

    // Add to database with fork info
    let forked_from = format!("{}/{}/{}", org, project, name);
    let relative_path = format!("{}/{}/{}", target_org, target_project, new_repo_dir_name);
    state
        .db
        .create_repository_with_fork(
            target_org,
            target_project,
            new_name,
            &relative_path,
            &forked_from,
        )
        .await?;

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
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let full_name = format!("{}/{}/{}", org, project, name);
    let issues = state.db.list_issues(&full_name).await?;

    Ok(Json(issues))
}

/// Get a single issue
async fn get_issue(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
) -> Result<Json<IssueInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let issue = state
        .db
        .get_issue(&full_name, number)
        .await?
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
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    if body.title.trim().is_empty() {
        return Err(AppError::bad_request("Issue title is required"));
    }

    let full_name = format!("{}/{}/{}", org, project, name);
    let issue = state
        .db
        .create_issue(
            &full_name,
            &body.title,
            &body.body,
            "anonymous",
            body.start_date.as_deref(),
            body.target_date.as_deref(),
        )
        .await?;

    Ok(Json(issue))
}

/// Update an issue
async fn update_issue(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
    Json(body): Json<UpdateIssueRequest>,
) -> Result<Json<IssueInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let issue = state
        .db
        .update_issue(
            &full_name,
            number,
            body.title.as_deref(),
            body.body.as_deref(),
            body.state.as_deref(),
            body.status.as_deref(),
            body.start_date.as_deref(),
            body.target_date.as_deref(),
        )
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    Ok(Json(issue))
}

/// List comments for an issue
async fn list_issue_comments(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
) -> Result<Json<Vec<IssueCommentInfo>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let issue = state
        .db
        .get_issue(&full_name, number)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let comments = state.db.list_issue_comments(issue.id).await?;

    Ok(Json(comments))
}

/// Create a comment on an issue
async fn create_issue_comment(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
    Json(body): Json<CreateCommentRequest>,
) -> Result<Json<IssueCommentInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let issue = state
        .db
        .get_issue(&full_name, number)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    if body.body.trim().is_empty() {
        return Err(AppError::bad_request("Comment body is required"));
    }

    let comment = state
        .db
        .create_issue_comment(issue.id, &body.body, "anonymous")
        .await?;

    Ok(Json(comment))
}

/// Update a comment on an issue
async fn update_issue_comment(
    State(state): State<AppState>,
    Path((org, project, name, number, comment_id)): Path<(String, String, String, i64, i64)>,
    Json(body): Json<UpdateCommentRequest>,
) -> Result<Json<IssueCommentInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let _issue = state
        .db
        .get_issue(&full_name, number)
        .await?
        .ok_or_else(|| AppError::not_found("Issue not found"))?;

    if body.body.trim().is_empty() {
        return Err(AppError::bad_request("Comment body is required"));
    }

    let comment = state
        .db
        .update_issue_comment(comment_id, &body.body)
        .await?
        .ok_or_else(|| AppError::not_found("Comment not found"))?;

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
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let full_name = format!("{}/{}/{}", org, project, name);
    let prs = state.db.list_pull_requests(&full_name).await?;

    Ok(Json(prs))
}

/// Get a single pull request
async fn get_pull_request(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
) -> Result<Json<PullRequestInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let pr = state
        .db
        .get_pull_request(&full_name, number)
        .await?
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
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    if body.title.trim().is_empty() {
        return Err(AppError::bad_request("Pull request title is required"));
    }

    let full_name = format!("{}/{}/{}", org, project, name);
    let pr = state
        .db
        .create_pull_request(
            &full_name,
            &body.title,
            &body.body,
            &body.source_repo,
            &body.source_branch,
            &body.target_branch,
            "anonymous",
        )
        .await?;

    Ok(Json(pr))
}

/// Update a pull request
async fn update_pull_request(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
    Json(body): Json<UpdatePullRequestRequest>,
) -> Result<Json<PullRequestInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let pr = state
        .db
        .update_pull_request(
            &full_name,
            number,
            body.title.as_deref(),
            body.body.as_deref(),
            body.state.as_deref(),
        )
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    Ok(Json(pr))
}

/// List comments for a pull request
async fn list_pr_comments(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
) -> Result<Json<Vec<PullRequestCommentInfo>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let pr = state
        .db
        .get_pull_request(&full_name, number)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let comments = state.db.list_pr_comments(pr.id).await?;

    Ok(Json(comments))
}

/// Create a comment on a pull request
async fn create_pr_comment(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
    Json(body): Json<CreateCommentRequest>,
) -> Result<Json<PullRequestCommentInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let pr = state
        .db
        .get_pull_request(&full_name, number)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    if body.body.trim().is_empty() {
        return Err(AppError::bad_request("Comment body is required"));
    }

    let comment = state
        .db
        .create_pr_comment(pr.id, &body.body, "anonymous")
        .await?;

    Ok(Json(comment))
}

/// Get commits for a pull request
async fn get_pr_commits(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
) -> Result<Json<Vec<CommitInfo>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let pr = state
        .db
        .get_pull_request(&full_name, number)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    // Get source repo - source_repo is in org/project/name format
    let source_parts: Vec<&str> = pr.source_repo.split('/').collect();
    let (source_org, source_project, source_name) = if source_parts.len() == 3 {
        (source_parts[0], source_parts[1], source_parts[2])
    } else {
        return Err(AppError::internal("Invalid source repo format"));
    };

    let source_repo = state
        .db
        .get_repository(source_org, source_project, source_name)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&source_repo.path);

    // Get commits between target and source branches using git2
    match git_ops::get_commits_between(&repo_path, &pr.target_branch, &pr.source_branch, 50) {
        Ok(commits) => {
            let commit_infos: Vec<CommitInfo> = commits
                .into_iter()
                .map(|c| CommitInfo {
                    hash: c.hash,
                    short_hash: c.short_hash,
                    author: c.author,
                    date: c.date,
                    message: c.message,
                })
                .collect();
            Ok(Json(commit_infos))
        }
        Err(_) => Ok(Json(vec![])),
    }
}

/// Get files changed in a pull request
async fn get_pr_files(
    State(state): State<AppState>,
    Path((org, project, name, number)): Path<(String, String, String, i64)>,
) -> Result<Json<Vec<FileDiff>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, name);
    let pr = state
        .db
        .get_pull_request(&full_name, number)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    // Get target repo
    let target_repo = state
        .db
        .get_repository(&org, &project, &name)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    // Get source repo - source_repo is in org/project/name format
    let source_parts: Vec<&str> = pr.source_repo.split('/').collect();
    let (source_org, source_project, source_name) = if source_parts.len() == 3 {
        (source_parts[0], source_parts[1], source_parts[2])
    } else {
        return Err(AppError::internal("Invalid source repo format"));
    };

    let source_repo = state
        .db
        .get_repository(source_org, source_project, source_name)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let source_repo_path = state.repos_path.join(&source_repo.path);
    let target_repo_path = state.repos_path.join(&target_repo.path);

    // If same repo, do a simple diff
    if source_repo.org_name == target_repo.org_name
        && source_repo.project_name == target_repo.project_name
        && source_repo.name == target_repo.name
    {
        return get_branch_diff(&source_repo_path, &pr.target_branch, &pr.source_branch);
    }

    // For cross-repo PRs, we need to fetch the target and compare using git2
    let _ = git_ops::add_remote_and_fetch(
        &source_repo_path,
        "target-for-pr",
        &target_repo_path,
        &pr.target_branch,
    );

    let target_ref = format!("target-for-pr/{}", pr.target_branch);
    get_branch_diff(&source_repo_path, &target_ref, &pr.source_branch)
}

/// Get diff between two branches
fn get_branch_diff(
    repo_path: &PathBuf,
    base: &str,
    head: &str,
) -> Result<Json<Vec<FileDiff>>, AppError> {
    match git_ops::get_branch_diff(repo_path, base, head) {
        Ok(diffs) => {
            let file_diffs: Vec<FileDiff> = diffs
                .into_iter()
                .map(|d| FileDiff {
                    path: d.path,
                    status: d.status,
                    additions: d.additions,
                    deletions: d.deletions,
                    diff: d.diff,
                })
                .collect();
            Ok(Json(file_diffs))
        }
        Err(_) => Ok(Json(vec![])),
    }
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
    let repo = state
        .db
        .get_repository(&org, &project, repo_name)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);

    // Get advertised refs using git2
    let refs_output = git_ops::get_advertised_refs(&repo_path, service)
        .map_err(|e| AppError::internal(e.to_string()))?;

    // Build response with proper git smart protocol format
    let pkt_line = format!("# service={}\n", service);
    let pkt_len = format!("{:04x}", pkt_line.len() + 4);

    let mut body = Vec::new();
    body.extend_from_slice(pkt_len.as_bytes());
    body.extend_from_slice(pkt_line.as_bytes());
    body.extend_from_slice(b"0000"); // flush packet
    body.extend_from_slice(&refs_output);

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
    let repo = state
        .db
        .get_repository(&org, &project, repo_name)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);

    // Process upload-pack request using git2
    let response_data = git_ops::process_upload_pack_request(&repo_path, &body)
        .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-git-upload-pack-result")
        .header("Cache-Control", "no-cache")
        .body(Body::from(response_data))
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
    let repo = state
        .db
        .get_repository(&org, &project, repo_name)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let repo_path = state.repos_path.join(&repo.path);

    // Process receive-pack request using git2
    let response_data = git_ops::process_receive_pack_request(&repo_path, &body)
        .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "application/x-git-receive-pack-result",
        )
        .header("Cache-Control", "no-cache")
        .body(Body::from(response_data))
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
    state
        .db
        .get_repository(org, project, project)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))
}

/// List files in a project's repository
async fn project_list_files(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
) -> Result<Json<Vec<FileEntry>>, AppError> {
    let repo = get_project_repo(&state, &org, &project).await?;
    let repo_path = state.repos_path.join(&repo.path);
    let files = list_git_files(&repo_path, "HEAD", "")?;
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
    let commits = list_git_commits(&repo_path)?;
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
    let files = list_git_files(&repo_path, git_ref, path)?;
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
    let path = query
        .path
        .as_deref()
        .ok_or_else(|| AppError::bad_request("path query parameter required"))?;
    let content = get_git_blob(&repo_path, git_ref, path)?;
    Ok(content)
}

/// List branches in a project's repository
async fn project_list_branches(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
) -> Result<Json<Vec<String>>, AppError> {
    let repo = get_project_repo(&state, &org, &project).await?;
    let repo_path = state.repos_path.join(&repo.path);

    match git_ops::list_branches(&repo_path) {
        Ok(branches) => Ok(Json(branches)),
        Err(_) => Ok(Json(vec![])),
    }
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

    if !new_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(AppError::bad_request(
            "Project name contains invalid characters",
        ));
    }

    // Determine target org (default to source org)
    let target_org = body.target_org.as_deref().unwrap_or(&org);
    // For projects, the target_project is the new name
    let target_project = new_name;

    // Create the target project first
    let display_name = new_name.to_string();
    let description = format!("Forked from {}/{}", org, project);

    // Check if target project already exists
    let existing = state.db.get_project(target_org, target_project).await?;

    if existing.is_some() {
        return Err(AppError::conflict("Project already exists"));
    }

    // Create project directory
    let project_path = state.repos_path.join(target_org).join(target_project);
    fs::create_dir_all(&project_path).await?;

    // Create the project in database
    state
        .db
        .create_project(target_org, target_project, &display_name, &description)
        .await?;

    // Create the new repository directory
    let new_repo_dir_name = format!("{}.git", target_project);
    let new_repo_path = state
        .repos_path
        .join(target_org)
        .join(target_project)
        .join(&new_repo_dir_name);
    let source_repo_path = state.repos_path.join(&source_repo.path);

    // Clone the repository using git2
    git_ops::clone_bare(&source_repo_path, &new_repo_path)
        .map_err(|e| AppError::internal(format!("Failed to fork repository: {}", e)))?;

    // Add to database with fork info
    let forked_from = format!("{}/{}", org, project);
    let relative_path = format!("{}/{}/{}", target_org, target_project, new_repo_dir_name);
    state
        .db
        .create_repository_with_fork(
            target_org,
            target_project,
            target_project,
            &relative_path,
            &forked_from,
        )
        .await?;

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
    let issues = state.db.list_issues(&full_name).await?;
    Ok(Json(issues))
}

/// Get a single issue for a project
async fn project_get_issue(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
) -> Result<Json<IssueInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let issue = state
        .db
        .get_issue(&full_name, number)
        .await?
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
    let issue = state
        .db
        .create_issue(
            &full_name,
            &body.title,
            &body.body,
            "anonymous",
            body.start_date.as_deref(),
            body.target_date.as_deref(),
        )
        .await?;
    Ok(Json(issue))
}

/// Update an issue for a project
async fn project_update_issue(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
    Json(body): Json<UpdateIssueRequest>,
) -> Result<Json<IssueInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let issue = state
        .db
        .update_issue(
            &full_name,
            number,
            body.title.as_deref(),
            body.body.as_deref(),
            body.state.as_deref(),
            body.status.as_deref(),
            body.start_date.as_deref(),
            body.target_date.as_deref(),
        )
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;
    Ok(Json(issue))
}

/// List comments for an issue in a project
async fn project_list_issue_comments(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
) -> Result<Json<Vec<IssueCommentInfo>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let issue = state
        .db
        .get_issue(&full_name, number)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let comments = state.db.list_issue_comments(issue.id).await?;
    Ok(Json(comments))
}

/// Create a comment on an issue in a project
async fn project_create_issue_comment(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
    Json(body): Json<CreateCommentRequest>,
) -> Result<Json<IssueCommentInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let issue = state
        .db
        .get_issue(&full_name, number)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    if body.body.trim().is_empty() {
        return Err(AppError::bad_request("Comment body is required"));
    }

    let comment = state
        .db
        .create_issue_comment(issue.id, &body.body, "anonymous")
        .await?;
    Ok(Json(comment))
}

/// Update a comment on an issue in a project
async fn project_update_issue_comment(
    State(state): State<AppState>,
    Path((org, project, number, comment_id)): Path<(String, String, i64, i64)>,
    Json(body): Json<UpdateCommentRequest>,
) -> Result<Json<IssueCommentInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let _issue = state
        .db
        .get_issue(&full_name, number)
        .await?
        .ok_or_else(|| AppError::not_found("Issue not found"))?;

    if body.body.trim().is_empty() {
        return Err(AppError::bad_request("Comment body is required"));
    }

    let comment = state
        .db
        .update_issue_comment(comment_id, &body.body)
        .await?
        .ok_or_else(|| AppError::not_found("Comment not found"))?;
    Ok(Json(comment))
}

// ============ Project-level Tag Handlers ============

/// List tags for a project
async fn project_list_tags(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
) -> Result<Json<Vec<TagInfo>>, AppError> {
    let _repo = get_project_repo(&state, &org, &project).await?;
    let full_name = format!("{}/{}/{}", org, project, project);
    let tags = state.db.list_tags(&full_name).await?;
    Ok(Json(tags))
}

/// Create a new tag for a project
async fn project_create_tag(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
    Json(body): Json<CreateTagRequest>,
) -> Result<Json<TagInfo>, AppError> {
    let _repo = get_project_repo(&state, &org, &project).await?;

    if body.name.trim().is_empty() {
        return Err(AppError::bad_request("Tag name is required"));
    }

    let full_name = format!("{}/{}/{}", org, project, project);
    let tag = state
        .db
        .create_tag(&full_name, &body.name, &body.color)
        .await?;
    Ok(Json(tag))
}

/// Get a single tag for a project
async fn project_get_tag(
    State(state): State<AppState>,
    Path((org, project, tag_id)): Path<(String, String, i64)>,
) -> Result<Json<TagInfo>, AppError> {
    let _repo = get_project_repo(&state, &org, &project).await?;
    let tag = state
        .db
        .get_tag_by_id(tag_id)
        .await?
        .ok_or_else(|| AppError::not_found("Tag not found"))?;
    Ok(Json(tag))
}

/// Update a tag for a project
async fn project_update_tag(
    State(state): State<AppState>,
    Path((org, project, tag_id)): Path<(String, String, i64)>,
    Json(body): Json<UpdateTagRequest>,
) -> Result<Json<TagInfo>, AppError> {
    let _repo = get_project_repo(&state, &org, &project).await?;
    let tag = state
        .db
        .update_tag(tag_id, body.name.as_deref(), body.color.as_deref())
        .await?
        .ok_or_else(|| AppError::not_found("Tag not found"))?;
    Ok(Json(tag))
}

/// Delete a tag for a project
async fn project_delete_tag(
    State(state): State<AppState>,
    Path((org, project, tag_id)): Path<(String, String, i64)>,
) -> Result<(), AppError> {
    let _repo = get_project_repo(&state, &org, &project).await?;
    state.db.delete_tag(tag_id).await?;
    Ok(())
}

/// Get tags for an issue in a project
async fn project_get_issue_tags(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
) -> Result<Json<Vec<TagInfo>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let issue = state
        .db
        .get_issue(&full_name, number)
        .await?
        .ok_or_else(|| AppError::not_found("Issue not found"))?;

    let tags = state.db.get_issue_tags(issue.id).await?;
    Ok(Json(tags))
}

/// Add tag to issue request body
#[derive(Debug, Deserialize)]
pub struct AddTagToIssueRequest {
    pub tag_id: i64,
}

/// Add a tag to an issue in a project
async fn project_add_issue_tag(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
    Json(body): Json<AddTagToIssueRequest>,
) -> Result<(), AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let issue = state
        .db
        .get_issue(&full_name, number)
        .await?
        .ok_or_else(|| AppError::not_found("Issue not found"))?;

    state.db.add_tag_to_issue(issue.id, body.tag_id).await?;
    Ok(())
}

/// Remove a tag from an issue in a project
async fn project_remove_issue_tag(
    State(state): State<AppState>,
    Path((org, project, number, tag_id)): Path<(String, String, i64, i64)>,
) -> Result<(), AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let issue = state
        .db
        .get_issue(&full_name, number)
        .await?
        .ok_or_else(|| AppError::not_found("Issue not found"))?;

    state.db.remove_tag_from_issue(issue.id, tag_id).await?;
    Ok(())
}

// ============ Project-level Pull Request Handlers ============

/// List pull requests for a project
async fn project_list_pull_requests(
    State(state): State<AppState>,
    Path((org, project)): Path<(String, String)>,
) -> Result<Json<Vec<PullRequestInfo>>, AppError> {
    let _repo = get_project_repo(&state, &org, &project).await?;
    let full_name = format!("{}/{}/{}", org, project, project);
    let prs = state.db.list_pull_requests(&full_name).await?;
    Ok(Json(prs))
}

/// Get a single pull request for a project
async fn project_get_pull_request(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
) -> Result<Json<PullRequestInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let pr = state
        .db
        .get_pull_request(&full_name, number)
        .await?
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
    let pr = state
        .db
        .create_pull_request(
            &full_name,
            &body.title,
            &body.body,
            &body.source_repo,
            &body.source_branch,
            &body.target_branch,
            "anonymous",
        )
        .await?;
    Ok(Json(pr))
}

/// Update a pull request for a project
async fn project_update_pull_request(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
    Json(body): Json<UpdatePullRequestRequest>,
) -> Result<Json<PullRequestInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let pr = state
        .db
        .update_pull_request(
            &full_name,
            number,
            body.title.as_deref(),
            body.body.as_deref(),
            body.state.as_deref(),
        )
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;
    Ok(Json(pr))
}

/// List comments for a pull request in a project
async fn project_list_pr_comments(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
) -> Result<Json<Vec<PullRequestCommentInfo>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let pr = state
        .db
        .get_pull_request(&full_name, number)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    let comments = state.db.list_pr_comments(pr.id).await?;
    Ok(Json(comments))
}

/// Create a comment on a pull request in a project
async fn project_create_pr_comment(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
    Json(body): Json<CreateCommentRequest>,
) -> Result<Json<PullRequestCommentInfo>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let pr = state
        .db
        .get_pull_request(&full_name, number)
        .await?
        .ok_or_else(|| AppError::not_found("Not found"))?;

    if body.body.trim().is_empty() {
        return Err(AppError::bad_request("Comment body is required"));
    }

    let comment = state
        .db
        .create_pr_comment(pr.id, &body.body, "anonymous")
        .await?;
    Ok(Json(comment))
}

/// Get commits for a pull request in a project
async fn project_get_pr_commits(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
) -> Result<Json<Vec<CommitInfo>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let pr = state
        .db
        .get_pull_request(&full_name, number)
        .await?
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

    // Get commits between target and source branches using git2
    match git_ops::get_commits_between(&repo_path, &pr.target_branch, &pr.source_branch, 50) {
        Ok(commits) => {
            let commit_infos: Vec<CommitInfo> = commits
                .into_iter()
                .map(|c| CommitInfo {
                    hash: c.hash,
                    short_hash: c.short_hash,
                    author: c.author,
                    date: c.date,
                    message: c.message,
                })
                .collect();
            Ok(Json(commit_infos))
        }
        Err(_) => Ok(Json(vec![])),
    }
}

/// Get files changed in a pull request for a project
async fn project_get_pr_files(
    State(state): State<AppState>,
    Path((org, project, number)): Path<(String, String, i64)>,
) -> Result<Json<Vec<FileDiff>>, AppError> {
    let full_name = format!("{}/{}/{}", org, project, project);
    let pr = state
        .db
        .get_pull_request(&full_name, number)
        .await?
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
    if source_repo.org_name == target_repo.org_name
        && source_repo.project_name == target_repo.project_name
    {
        return get_branch_diff(&source_repo_path, &pr.target_branch, &pr.source_branch);
    }

    // For cross-repo PRs, fetch and compare using git2
    let _ = git_ops::add_remote_and_fetch(
        &source_repo_path,
        "target-for-pr",
        &target_repo_path,
        &pr.target_branch,
    );

    let target_ref = format!("target-for-pr/{}", pr.target_branch);
    get_branch_diff(&source_repo_path, &target_ref, &pr.source_branch)
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

    // Get advertised refs using git2
    let refs_output = git_ops::get_advertised_refs(&repo_path, service)
        .map_err(|e| AppError::internal(e.to_string()))?;

    let pkt_line = format!("# service={}\n", service);
    let pkt_len = format!("{:04x}", pkt_line.len() + 4);

    let mut body = Vec::new();
    body.extend_from_slice(pkt_len.as_bytes());
    body.extend_from_slice(pkt_line.as_bytes());
    body.extend_from_slice(b"0000");
    body.extend_from_slice(&refs_output);

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

    // Process upload-pack request using git2
    let response_data = git_ops::process_upload_pack_request(&repo_path, &body)
        .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-git-upload-pack-result")
        .header("Cache-Control", "no-cache")
        .body(Body::from(response_data))
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

    // Process receive-pack request using git2
    let response_data = git_ops::process_receive_pack_request(&repo_path, &body)
        .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "application/x-git-receive-pack-result",
        )
        .header("Cache-Control", "no-cache")
        .body(Body::from(response_data))
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

    // Use git2 to update the file directly in the bare repository
    git_ops::update_file_and_commit(&repo_path, &body.path, &body.content, &body.message)
        .map_err(|e| AppError::internal(format!("Failed to update file: {}", e)))?;

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

    // Use git2 to delete the file directly in the bare repository
    git_ops::delete_file_and_commit(&repo_path, &body.path, &body.message)
        .map_err(|e| AppError::internal(format!("Failed to delete file: {}", e)))?;

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
