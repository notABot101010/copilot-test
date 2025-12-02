//! HTTP handlers for TVflix API

use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_http::services::ServeDir;
use tracing::info;

/// Cookie name for session token
const SESSION_COOKIE_NAME: &str = "tvflix_session";

use crate::auth::{generate_token, hash_password, verify_password};
use crate::database::{DatabaseError, MediaType};
use crate::storage::StorageError;
use crate::AppState;

/// Error type for HTTP handlers
#[derive(Debug)]
pub enum ApiError {
    Database(DatabaseError),
    Storage(StorageError),
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    Internal(String),
}

impl From<DatabaseError> for ApiError {
    fn from(err: DatabaseError) -> Self {
        match &err {
            DatabaseError::UserNotFound => ApiError::NotFound(err.to_string()),
            DatabaseError::MediaNotFound(_) => ApiError::NotFound(err.to_string()),
            DatabaseError::PlaylistNotFound(_) => ApiError::NotFound(err.to_string()),
            DatabaseError::AlbumNotFound(_) => ApiError::NotFound(err.to_string()),
            DatabaseError::InvalidSession => ApiError::Unauthorized(err.to_string()),
            DatabaseError::UserAlreadyExists(_) => ApiError::BadRequest(err.to_string()),
            _ => ApiError::Database(err),
        }
    }
}

impl From<StorageError> for ApiError {
    fn from(err: StorageError) -> Self {
        match &err {
            StorageError::NotFound(_) => ApiError::NotFound(err.to_string()),
            _ => ApiError::Storage(err),
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::Database(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            ApiError::Storage(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

type Result<T> = std::result::Result<T, ApiError>;

// Request/Response types
#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
    user: UserResponse,
}

#[derive(Serialize)]
pub struct UserResponse {
    id: i64,
    username: String,
}

#[derive(Deserialize)]
pub struct CreatePlaylistRequest {
    name: String,
}

#[derive(Deserialize)]
pub struct CreateAlbumRequest {
    name: String,
}

#[derive(Deserialize)]
pub struct AddToPlaylistRequest {
    media_id: i64,
}

#[derive(Deserialize)]
pub struct AddToAlbumRequest {
    media_id: i64,
}

#[derive(Deserialize)]
pub struct MediaQuery {
    #[serde(rename = "type")]
    media_type: Option<String>,
}

/// Create the router for the API
pub fn create_router(state: AppState) -> Router {
    // Get static file path (for serving webapp)
    let static_path = std::env::current_dir()
        .map(|p| p.join("static/dist"))
        .unwrap_or_else(|_| PathBuf::from("static/dist"));

    Router::new()
        // Auth routes
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/me", get(get_current_user))
        // Media routes
        .route("/api/media", get(list_media))
        .route("/api/media", post(upload_media))
        .route("/api/media/:id", get(get_media))
        .route("/api/media/:id", delete(delete_media))
        .route("/api/media/:id/stream", get(stream_media))
        .route("/api/media/:id/thumbnail", get(get_thumbnail))
        // Playlist routes
        .route("/api/playlists", get(list_playlists))
        .route("/api/playlists", post(create_playlist))
        .route("/api/playlists/:id", get(get_playlist))
        .route("/api/playlists/:id", delete(delete_playlist))
        .route("/api/playlists/:id/items", post(add_to_playlist))
        .route("/api/playlists/:id/items/:media_id", delete(remove_from_playlist))
        // Album routes
        .route("/api/albums", get(list_albums))
        .route("/api/albums", post(create_album))
        .route("/api/albums/:id", get(get_album))
        .route("/api/albums/:id", delete(delete_album))
        .route("/api/albums/:id/items", post(add_to_album))
        .route("/api/albums/:id/items/:media_id", delete(remove_from_album))
        // Serve static files (webapp)
        .nest_service("/", ServeDir::new(static_path).append_index_html_on_directories(true))
        // Increase body limit for large file uploads (10GB)
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Extract user ID from session token in headers
async fn get_user_from_headers(state: &AppState, headers: &HeaderMap) -> Result<i64> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Unauthorized("Invalid authorization header format".to_string()))?;

    let session = state
        .db
        .get_session_by_token(token)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid or expired session".to_string()))?;

    Ok(session.user_id)
}

/// Extract session token from cookie header
fn get_token_from_cookie(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    
    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if let Some(value) = cookie.strip_prefix(&format!("{}=", SESSION_COOKIE_NAME)) {
            return Some(value.to_string());
        }
    }
    None
}

/// Extract user ID from session token in headers or cookie
/// This is used for endpoints that need to authenticate when accessed from HTML elements
/// like video/audio/img src attributes, which can't send Authorization headers
async fn get_user_from_headers_or_cookie(state: &AppState, headers: &HeaderMap) -> Result<i64> {
    // First try Authorization header
    if let Some(auth_header) = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if let Some(session) = state.db.get_session_by_token(token).await? {
                return Ok(session.user_id);
            }
        }
    }
    
    // Fall back to cookie
    if let Some(token) = get_token_from_cookie(headers) {
        if let Some(session) = state.db.get_session_by_token(&token).await? {
            return Ok(session.user_id);
        }
    }
    
    Err(ApiError::Unauthorized("Invalid or expired session".to_string()))
}

/// Create a Set-Cookie header value for the session token
fn create_session_cookie(token: &str) -> String {
    format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=604800",
        SESSION_COOKIE_NAME,
        token
    )
}

/// Create a Set-Cookie header value to clear the session cookie
fn create_logout_cookie() -> String {
    format!(
        "{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0",
        SESSION_COOKIE_NAME
    )
}

// Auth handlers
async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Response> {
    if req.username.is_empty() || req.password.is_empty() {
        return Err(ApiError::BadRequest("Username and password are required".to_string()));
    }

    if req.password.len() < 6 {
        return Err(ApiError::BadRequest("Password must be at least 6 characters".to_string()));
    }

    let password_hash = hash_password(&req.password);
    let user = state.db.create_user(&req.username, &password_hash).await?;

    let token = generate_token();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .ok_or_else(|| ApiError::Internal("Failed to calculate expiry".to_string()))?;

    state.db.create_session(user.id, &token, &expires_at).await?;

    let auth_response = AuthResponse {
        token: token.clone(),
        user: UserResponse {
            id: user.id,
            username: user.username,
        },
    };

    let cookie = create_session_cookie(&token);
    let mut response = Json(auth_response).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|err| ApiError::Internal(err.to_string()))?,
    );
    
    Ok(response)
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Response> {
    let user = state
        .db
        .get_user_by_username(&req.username)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid username or password".to_string()))?;

    if !verify_password(&req.password, &user.password_hash) {
        return Err(ApiError::Unauthorized("Invalid username or password".to_string()));
    }

    let token = generate_token();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .ok_or_else(|| ApiError::Internal("Failed to calculate expiry".to_string()))?;

    state.db.create_session(user.id, &token, &expires_at).await?;

    let auth_response = AuthResponse {
        token: token.clone(),
        user: UserResponse {
            id: user.id,
            username: user.username,
        },
    };

    let cookie = create_session_cookie(&token);
    let mut response = Json(auth_response).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|err| ApiError::Internal(err.to_string()))?,
    );
    
    Ok(response)
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Result<Response> {
    // Try to delete session from Authorization header
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                state.db.delete_session(token).await?;
            }
        }
    }
    
    // Also try to delete session from cookie
    if let Some(token) = get_token_from_cookie(&headers) {
        state.db.delete_session(&token).await?;
    }

    // Clear the cookie
    let cookie = create_logout_cookie();
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|err| ApiError::Internal(err.to_string()))?,
    );
    
    Ok(response)
}

async fn get_current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserResponse>> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    let user = state
        .db
        .get_user_by_id(user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
    }))
}

// Media handlers
async fn list_media(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<MediaQuery>,
) -> Result<Json<Vec<crate::database::Media>>> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    let media_type = if let Some(ref mt) = query.media_type {
        Some(mt.parse::<MediaType>().map_err(|err| ApiError::BadRequest(err))?)
    } else {
        None
    };

    let media = state.db.list_media_by_user(user_id, media_type).await?;
    Ok(Json(media))
}

async fn upload_media(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<crate::database::Media>> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    let mut title: Option<String> = None;
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut storage_path: Option<PathBuf> = None;
    let mut size: i64 = 0;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ApiError::BadRequest(err.to_string()))?
    {
        let name = field.name().map(|s| s.to_string());

        match name.as_deref() {
            Some("title") => {
                title = Some(
                    field
                        .text()
                        .await
                        .map_err(|err| ApiError::BadRequest(err.to_string()))?,
                );
            }
            Some("file") => {
                filename = field.file_name().map(|s| s.to_string());
                content_type = field.content_type().map(|s| s.to_string());

                let ct = content_type.clone().unwrap_or_default();
                let media_type = determine_media_type(&ct);
                let fname = filename.clone().unwrap_or_else(|| "unknown".to_string());

                let path = state.storage.create_storage_path(user_id, &media_type.to_string(), &fname);

                // Stream upload to file
                let stream = field.map(|result| {
                    result.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))
                });

                size = state.storage.write_stream(&path, stream).await?;
                storage_path = Some(path);
            }
            _ => {}
        }
    }

    let filename = filename.ok_or_else(|| ApiError::BadRequest("No file provided".to_string()))?;
    let storage_path = storage_path.ok_or_else(|| ApiError::BadRequest("No file provided".to_string()))?;
    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let title = title.unwrap_or_else(|| {
        // Use filename without extension as title
        std::path::Path::new(&filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&filename)
            .to_string()
    });

    let media_type = determine_media_type(&content_type);

    // Create media record
    let media = state
        .db
        .create_media(
            user_id,
            &title,
            media_type,
            &filename,
            &storage_path.display().to_string(),
            None,
            &content_type,
            size,
            None,
        )
        .await?;

    // Generate thumbnail for videos asynchronously
    if media_type == MediaType::Video {
        let storage = state.storage.clone();
        let db = state.db.clone();
        let media_id = media.id;
        let video_path = storage_path.clone();
        let thumb_path = storage.create_thumbnail_path(user_id, &filename);

        tokio::spawn(async move {
            if let Err(err) = storage.generate_video_thumbnail(&video_path, &thumb_path).await {
                tracing::warn!("Failed to generate thumbnail: {}", err);
            } else if let Err(err) = db.update_media_thumbnail(media_id, &thumb_path.display().to_string()).await {
                tracing::warn!("Failed to update thumbnail path: {}", err);
            } else {
                info!("Generated thumbnail for media {}", media_id);
            }
        });
    }

    Ok(Json(media))
}

fn determine_media_type(content_type: &str) -> MediaType {
    if content_type.starts_with("video/") {
        MediaType::Video
    } else if content_type.starts_with("audio/") {
        MediaType::Music
    } else if content_type.starts_with("image/") {
        MediaType::Photo
    } else {
        // Try to guess from common extensions
        MediaType::Video
    }
}

async fn get_media(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<crate::database::Media>> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    let media = state
        .db
        .get_media_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Media not found".to_string()))?;

    // Check ownership
    if media.user_id != user_id {
        return Err(ApiError::NotFound("Media not found".to_string()));
    }

    Ok(Json(media))
}

async fn delete_media(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    // Check ownership
    let media = state
        .db
        .get_media_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Media not found".to_string()))?;

    if media.user_id != user_id {
        return Err(ApiError::NotFound("Media not found".to_string()));
    }

    // Delete from database and get paths
    if let Some((storage_path, thumbnail_path)) = state.db.delete_media(id).await? {
        // Delete files
        let _ = state.storage.delete(&PathBuf::from(storage_path)).await;
        if let Some(thumb) = thumbnail_path {
            let _ = state.storage.delete(&PathBuf::from(thumb)).await;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn stream_media(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Response<Body>> {
    // Use cookie-aware authentication for browser media elements
    let user_id = get_user_from_headers_or_cookie(&state, &headers).await?;

    let media = state
        .db
        .get_media_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Media not found".to_string()))?;

    if media.user_id != user_id {
        return Err(ApiError::NotFound("Media not found".to_string()));
    }

    let path = PathBuf::from(&media.storage_path);
    let file_size = state.storage.get_file_size(&path).await?;

    // Check for range header
    let range = headers.get(header::RANGE).and_then(|v| v.to_str().ok());

    if let Some(range_str) = range {
        if let Some((start, end)) = parse_range(range_str, file_size) {
            let stream = state.storage.read_range(&path, start, end).await?;
            let len = end - start + 1;
            let body = Body::from_stream(stream);

            return Ok(Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(header::CONTENT_LENGTH, len.to_string())
                .header(header::CONTENT_TYPE, &media.content_type)
                .header(header::ACCEPT_RANGES, "bytes")
                .header(
                    header::CONTENT_RANGE,
                    format!("bytes {}-{}/{}", start, end, file_size),
                )
                .body(body)
                .map_err(|err| ApiError::Internal(err.to_string()))?);
        }
    }

    // Full file response
    let stream = state.storage.read_stream(&path).await?;
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_LENGTH, file_size.to_string())
        .header(header::CONTENT_TYPE, &media.content_type)
        .header(header::ACCEPT_RANGES, "bytes")
        .body(body)
        .map_err(|err| ApiError::Internal(err.to_string()))?)
}

async fn get_thumbnail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Response<Body>> {
    // Use cookie-aware authentication for browser img elements
    let user_id = get_user_from_headers_or_cookie(&state, &headers).await?;

    let media = state
        .db
        .get_media_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Media not found".to_string()))?;

    if media.user_id != user_id {
        return Err(ApiError::NotFound("Media not found".to_string()));
    }

    let thumbnail_path = media
        .thumbnail_path
        .ok_or_else(|| ApiError::NotFound("Thumbnail not available".to_string()))?;

    let path = PathBuf::from(&thumbnail_path);
    let stream = state.storage.read_stream(&path).await?;
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/jpeg")
        .body(body)
        .map_err(|err| ApiError::Internal(err.to_string()))?)
}

fn parse_range(range_str: &str, total_size: u64) -> Option<(u64, u64)> {
    let range_str = range_str.strip_prefix("bytes=")?;
    let parts: Vec<&str> = range_str.split('-').collect();

    if parts.len() != 2 {
        return None;
    }

    let start: u64 = parts[0].parse().ok()?;
    let end: u64 = if parts[1].is_empty() {
        total_size - 1
    } else {
        parts[1].parse().ok()?
    };

    if start > end || end >= total_size {
        return None;
    }

    Some((start, end))
}

// Playlist handlers
async fn list_playlists(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::database::Playlist>>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    let playlists = state.db.list_playlists_by_user(user_id).await?;
    Ok(Json(playlists))
}

async fn create_playlist(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreatePlaylistRequest>,
) -> Result<Json<crate::database::Playlist>> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    if req.name.is_empty() {
        return Err(ApiError::BadRequest("Playlist name is required".to_string()));
    }

    let playlist = state.db.create_playlist(user_id, &req.name).await?;
    Ok(Json(playlist))
}

#[derive(Serialize)]
struct PlaylistWithMedia {
    #[serde(flatten)]
    playlist: crate::database::Playlist,
    items: Vec<crate::database::Media>,
}

async fn get_playlist(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<PlaylistWithMedia>> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    let playlist = state
        .db
        .get_playlist_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Playlist not found".to_string()))?;

    if playlist.user_id != user_id {
        return Err(ApiError::NotFound("Playlist not found".to_string()));
    }

    let items = state.db.get_playlist_media(id).await?;

    Ok(Json(PlaylistWithMedia { playlist, items }))
}

async fn delete_playlist(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    let playlist = state
        .db
        .get_playlist_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Playlist not found".to_string()))?;

    if playlist.user_id != user_id {
        return Err(ApiError::NotFound("Playlist not found".to_string()));
    }

    state.db.delete_playlist(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_to_playlist(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(req): Json<AddToPlaylistRequest>,
) -> Result<StatusCode> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    let playlist = state
        .db
        .get_playlist_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Playlist not found".to_string()))?;

    if playlist.user_id != user_id {
        return Err(ApiError::NotFound("Playlist not found".to_string()));
    }

    // Verify media exists and belongs to user
    let media = state
        .db
        .get_media_by_id(req.media_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Media not found".to_string()))?;

    if media.user_id != user_id {
        return Err(ApiError::NotFound("Media not found".to_string()));
    }

    state.db.add_to_playlist(id, req.media_id).await?;
    Ok(StatusCode::CREATED)
}

async fn remove_from_playlist(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((playlist_id, media_id)): Path<(i64, i64)>,
) -> Result<StatusCode> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    let playlist = state
        .db
        .get_playlist_by_id(playlist_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Playlist not found".to_string()))?;

    if playlist.user_id != user_id {
        return Err(ApiError::NotFound("Playlist not found".to_string()));
    }

    state.db.remove_from_playlist(playlist_id, media_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Album handlers
async fn list_albums(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::database::Album>>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    let albums = state.db.list_albums_by_user(user_id).await?;
    Ok(Json(albums))
}

async fn create_album(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateAlbumRequest>,
) -> Result<Json<crate::database::Album>> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    if req.name.is_empty() {
        return Err(ApiError::BadRequest("Album name is required".to_string()));
    }

    let album = state.db.create_album(user_id, &req.name).await?;
    Ok(Json(album))
}

#[derive(Serialize)]
struct AlbumWithMedia {
    #[serde(flatten)]
    album: crate::database::Album,
    items: Vec<crate::database::Media>,
}

async fn get_album(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<AlbumWithMedia>> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    let album = state
        .db
        .get_album_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Album not found".to_string()))?;

    if album.user_id != user_id {
        return Err(ApiError::NotFound("Album not found".to_string()));
    }

    let items = state.db.get_album_media(id).await?;

    Ok(Json(AlbumWithMedia { album, items }))
}

async fn delete_album(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    let album = state
        .db
        .get_album_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Album not found".to_string()))?;

    if album.user_id != user_id {
        return Err(ApiError::NotFound("Album not found".to_string()));
    }

    state.db.delete_album(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_to_album(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(req): Json<AddToAlbumRequest>,
) -> Result<StatusCode> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    let album = state
        .db
        .get_album_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Album not found".to_string()))?;

    if album.user_id != user_id {
        return Err(ApiError::NotFound("Album not found".to_string()));
    }

    // Verify media exists and belongs to user
    let media = state
        .db
        .get_media_by_id(req.media_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Media not found".to_string()))?;

    if media.user_id != user_id {
        return Err(ApiError::NotFound("Media not found".to_string()));
    }

    state.db.add_to_album(id, req.media_id).await?;
    Ok(StatusCode::CREATED)
}

async fn remove_from_album(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((album_id, media_id)): Path<(i64, i64)>,
) -> Result<StatusCode> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    let album = state
        .db
        .get_album_by_id(album_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Album not found".to_string()))?;

    if album.user_id != user_id {
        return Err(ApiError::NotFound("Album not found".to_string()));
    }

    state.db.remove_from_album(album_id, media_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
