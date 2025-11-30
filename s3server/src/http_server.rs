//! HTTP server for S3-compatible API

use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use futures_util::StreamExt;
use serde::Deserialize;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::auth::{AuthError, Authenticator};
use crate::database::{Database, DatabaseError};
use crate::storage::{Storage, StorageError};
use crate::xml_responses::*;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub storage: Storage,
    pub authenticator: Arc<Authenticator>,
    pub require_auth: bool,
}

/// Error type for HTTP handlers
#[derive(Debug)]
pub enum S3Error {
    Database(DatabaseError),
    Storage(StorageError),
    Auth(AuthError),
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    PreconditionFailed(String),
    InternalError(String),
}

impl From<DatabaseError> for S3Error {
    fn from(err: DatabaseError) -> Self {
        match &err {
            DatabaseError::BucketNotFound(_) => S3Error::NotFound(err.to_string()),
            DatabaseError::ObjectNotFound(_, _) => S3Error::NotFound(err.to_string()),
            DatabaseError::BucketAlreadyExists(_) => S3Error::Conflict(err.to_string()),
            _ => S3Error::Database(err),
        }
    }
}

impl From<StorageError> for S3Error {
    fn from(err: StorageError) -> Self {
        match &err {
            StorageError::NotFound(_) => S3Error::NotFound(err.to_string()),
            _ => S3Error::Storage(err),
        }
    }
}

impl From<AuthError> for S3Error {
    fn from(err: AuthError) -> Self {
        S3Error::Auth(err)
    }
}

impl IntoResponse for S3Error {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            S3Error::Database(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                err.to_string(),
            ),
            S3Error::Storage(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                err.to_string(),
            ),
            S3Error::Auth(err) => {
                let (status, code) = match err {
                    AuthError::MissingAuthHeader => (StatusCode::FORBIDDEN, "AccessDenied"),
                    AuthError::InvalidAuthHeader => (StatusCode::FORBIDDEN, "InvalidAccessKeyId"),
                    AuthError::AccessKeyNotFound => (StatusCode::FORBIDDEN, "InvalidAccessKeyId"),
                    AuthError::InvalidSignature => (StatusCode::FORBIDDEN, "SignatureDoesNotMatch"),
                    AuthError::RequestExpired => (StatusCode::FORBIDDEN, "RequestTimeTooSkewed"),
                    _ => (StatusCode::FORBIDDEN, "AccessDenied"),
                };
                (status, code, err.to_string())
            }
            S3Error::BadRequest(msg) => (StatusCode::BAD_REQUEST, "InvalidRequest", msg.clone()),
            S3Error::NotFound(msg) => (StatusCode::NOT_FOUND, "NoSuchKey", msg.clone()),
            S3Error::Conflict(msg) => (StatusCode::CONFLICT, "BucketAlreadyExists", msg.clone()),
            S3Error::PreconditionFailed(msg) => {
                (StatusCode::PRECONDITION_FAILED, "PreconditionFailed", msg.clone())
            }
            S3Error::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "InternalError", msg.clone())
            }
        };

        let error_response = ErrorResponse::new(code, &message, "");
        let body = error_response.to_xml();

        Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "application/xml")
            .body(Body::from(body))
            .unwrap()
    }
}

type Result<T> = std::result::Result<T, S3Error>;

/// Query parameters for list objects
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ListObjectsParams {
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: Option<i32>,
    pub continuation_token: Option<String>,
    #[serde(rename = "list-type")]
    pub list_type: Option<i32>,
}

/// Query parameters for multipart upload
#[derive(Debug, Deserialize, Default)]
pub struct MultipartParams {
    pub uploads: Option<String>,
    #[serde(rename = "uploadId")]
    pub upload_id: Option<String>,
    #[serde(rename = "partNumber")]
    pub part_number: Option<i32>,
}

/// Create the router for S3 API
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Service-level operations
        .route("/", get(list_buckets))
        // All other requests go through the fallback handler
        .fallback(handle_object_request)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

use axum::extract::OriginalUri;
use axum::http::Method;

/// Handle all object requests by parsing the path
async fn handle_object_request(
    State(state): State<AppState>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    Query(params): Query<MultipartParams>,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<Body>> {
    // Parse the path to extract bucket and key
    let path = uri.path();
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if parts.is_empty() {
        return Err(S3Error::NotFound("Not found".to_string()));
    }

    let bucket = parts[0];

    // If there's only one part (bucket only), handle bucket-level operations
    if parts.len() == 1 {
        match method {
            Method::HEAD => return head_bucket_impl(state, bucket).await,
            Method::GET => return get_bucket_or_list_objects_impl(state, bucket, params).await,
            Method::PUT => return create_bucket_impl(state, bucket).await,
            Method::DELETE => return delete_bucket_impl(state, bucket).await,
            _ => return Err(S3Error::BadRequest("Method not allowed".to_string())),
        }
    }

    // Otherwise, it's an object operation
    let key = parts[1..].join("/");

    match method {
        Method::HEAD => head_object_impl(state, bucket, &key).await,
        Method::GET => get_object_impl(state, bucket, &key, headers).await,
        Method::PUT => put_object_impl(state, bucket, &key, params, headers, body).await,
        Method::DELETE => delete_object_impl(state, bucket, &key, params).await,
        Method::POST => post_object_impl(state, bucket, &key, params, body).await,
        _ => Err(S3Error::BadRequest("Method not allowed".to_string())),
    }
}

/// List all buckets
async fn list_buckets(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let buckets = state.db.list_buckets().await?;

    let response = ListBucketsResponse::new(buckets, "owner-id", "Owner");
    let xml = response.to_xml();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml")
        .body(Body::from(xml))
        .unwrap())
}

/// Implementation of head bucket
async fn head_bucket_impl(state: AppState, bucket: &str) -> Result<Response<Body>> {
    state
        .db
        .get_bucket(bucket)
        .await?
        .ok_or_else(|| S3Error::NotFound(format!("Bucket not found: {}", bucket)))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap())
}

/// Implementation of get bucket or list objects
async fn get_bucket_or_list_objects_impl(
    state: AppState,
    bucket: &str,
    params: MultipartParams,
) -> Result<Response<Body>> {
    // Check if this is a list multipart uploads request
    if params.uploads.is_some() {
        return list_multipart_uploads(state, bucket).await;
    }

    // Default list objects with no params
    list_objects(state, bucket, ListObjectsParams::default()).await
}

/// List objects in a bucket
async fn list_objects(
    state: AppState,
    bucket: &str,
    params: ListObjectsParams,
) -> Result<Response<Body>> {
    let prefix = params.prefix.as_deref().unwrap_or("");
    let delimiter = params.delimiter.as_deref();
    let max_keys = params.max_keys.unwrap_or(1000).min(1000);
    let continuation_token = params.continuation_token.as_deref();

    let (objects, common_prefixes, next_token) = state
        .db
        .list_objects(bucket, Some(prefix), delimiter, max_keys, continuation_token)
        .await?;

    let response = ListObjectsV2Response::new(
        bucket,
        prefix,
        objects,
        common_prefixes,
        max_keys,
        params.continuation_token,
        next_token,
    );

    let xml = response.to_xml();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml")
        .body(Body::from(xml))
        .unwrap())
}

/// List multipart uploads for a bucket
async fn list_multipart_uploads(state: AppState, bucket: &str) -> Result<Response<Body>> {
    let uploads = state.db.list_multipart_uploads(bucket).await?;
    let response = ListMultipartUploadsResponse::new(bucket, uploads);
    let xml = response.to_xml();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml")
        .body(Body::from(xml))
        .unwrap())
}

/// Implementation of create bucket
async fn create_bucket_impl(state: AppState, bucket: &str) -> Result<Response<Body>> {
    // Validate bucket name
    if bucket.is_empty() || bucket.len() > 63 {
        return Err(S3Error::BadRequest("Invalid bucket name length".to_string()));
    }

    state.db.create_bucket(bucket, "owner").await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::LOCATION, format!("/{}", bucket))
        .body(Body::empty())
        .unwrap())
}

/// Implementation of delete bucket
async fn delete_bucket_impl(state: AppState, bucket: &str) -> Result<Response<Body>> {
    // Check if bucket is empty
    let (objects, _, _) = state
        .db
        .list_objects(bucket, None, None, 1, None)
        .await?;

    if !objects.is_empty() {
        return Err(S3Error::Conflict("Bucket is not empty".to_string()));
    }

    state.db.delete_bucket(bucket).await?;

    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap())
}

/// Implementation of head object
async fn head_object_impl(
    state: AppState,
    bucket: &str,
    key: &str,
) -> Result<Response<Body>> {
    let obj = state
        .db
        .get_object(bucket, key)
        .await?
        .ok_or_else(|| S3Error::NotFound(format!("Object not found: {}/{}", bucket, key)))?;

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_LENGTH, obj.size.to_string())
        .header(header::ETAG, &obj.etag)
        .header("Last-Modified", format_http_date(&obj.last_modified));

    if let Some(content_type) = &obj.content_type {
        response = response.header(header::CONTENT_TYPE, content_type);
    }

    Ok(response.body(Body::empty()).unwrap())
}

/// Implementation of get object
async fn get_object_impl(
    state: AppState,
    bucket: &str,
    key: &str,
    headers: HeaderMap,
) -> Result<Response<Body>> {
    let obj = state
        .db
        .get_object(&bucket, &key)
        .await?
        .ok_or_else(|| S3Error::NotFound(format!("Object not found: {}/{}", bucket, key)))?;

    // Check for range header
    let range = headers.get(header::RANGE).and_then(|v| v.to_str().ok());

    // Handle range requests
    if let Some(range_str) = range {
        if let Some((start, end)) = parse_range(range_str, obj.size as u64) {
            let stream = state.storage.read_range(&obj.storage_path, start, end).await?;
            let len = end - start + 1;
            let body = Body::from_stream(stream);

            let mut response = Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(header::CONTENT_LENGTH, len.to_string())
                .header(header::ETAG, &obj.etag)
                .header("Last-Modified", format_http_date(&obj.last_modified))
                .header(header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, obj.size));

            if let Some(content_type) = &obj.content_type {
                response = response.header(header::CONTENT_TYPE, content_type);
            }

            return Ok(response.body(body).unwrap());
        }
    }

    // Regular full object download
    let stream = state.storage.read_stream(&obj.storage_path).await?;
    let body = Body::from_stream(stream);

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_LENGTH, obj.size.to_string())
        .header(header::ETAG, &obj.etag)
        .header("Last-Modified", format_http_date(&obj.last_modified));

    if let Some(content_type) = &obj.content_type {
        response = response.header(header::CONTENT_TYPE, content_type);
    }

    Ok(response.body(body).unwrap())
}

/// Implementation of put object
async fn put_object_impl(
    state: AppState,
    bucket: &str,
    key: &str,
    params: MultipartParams,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<Body>> {
    // Check if this is an upload part request
    if let (Some(upload_id), Some(part_number)) = (&params.upload_id, params.part_number) {
        return upload_part(state, bucket, key, upload_id, part_number, headers, body).await;
    }

    // Check If-None-Match header for conditional write
    if let Some(if_none_match) = headers.get("if-none-match").and_then(|v| v.to_str().ok()) {
        if if_none_match == "*" {
            // Object must not exist
            if state.db.object_exists(bucket, key).await? {
                return Err(S3Error::PreconditionFailed(
                    "Object already exists".to_string(),
                ));
            }
        }
    }

    // Get content type from header
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Create storage path
    let storage_path = state.storage.create_storage_path(bucket, key);

    // Stream the body to storage
    let stream = body.into_data_stream().map(|result| {
        result.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))
    });

    let (size, etag) = state.storage.write_stream(&storage_path, stream).await?;

    // Store metadata in database
    state
        .db
        .create_object(bucket, key, size, &etag, content_type.as_deref(), &storage_path)
        .await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::ETAG, &etag)
        .body(Body::empty())
        .unwrap())
}

/// Upload a part of a multipart upload
async fn upload_part(
    state: AppState,
    bucket: &str,
    key: &str,
    upload_id: &str,
    part_number: i32,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<Body>> {
    // Get the upload
    let upload = state
        .db
        .get_multipart_upload(bucket, upload_id)
        .await?
        .ok_or_else(|| S3Error::NotFound(format!("Upload not found: {}", upload_id)))?;

    // Verify the key matches
    if upload.key != key {
        return Err(S3Error::BadRequest("Key mismatch".to_string()));
    }

    // Create storage path for this part
    let storage_path = state.storage.create_part_storage_path(upload_id, part_number);

    // Stream the body to storage
    let stream = body.into_data_stream().map(|result| {
        result.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))
    });

    let (size, etag) = state.storage.write_stream(&storage_path, stream).await?;

    // Store part metadata
    state
        .db
        .add_upload_part(upload.id, part_number, size, &etag, &storage_path)
        .await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::ETAG, &etag)
        .body(Body::empty())
        .unwrap())
}

/// Implementation of delete object
async fn delete_object_impl(
    state: AppState,
    bucket: &str,
    key: &str,
    params: MultipartParams,
) -> Result<Response<Body>> {
    // Check if this is an abort multipart upload request
    if let Some(upload_id) = &params.upload_id {
        return abort_multipart_upload(state, bucket, upload_id).await;
    }

    // Delete object from database and get storage path
    if let Some(storage_path) = state.db.delete_object(bucket, key).await? {
        // Delete from storage
        state.storage.delete(&storage_path).await?;
    }

    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap())
}

/// Abort multipart upload
async fn abort_multipart_upload(
    state: AppState,
    bucket: &str,
    upload_id: &str,
) -> Result<Response<Body>> {
    let upload = state
        .db
        .get_multipart_upload(bucket, upload_id)
        .await?
        .ok_or_else(|| S3Error::NotFound(format!("Upload not found: {}", upload_id)))?;

    // Delete all parts from storage
    let storage_paths = state.db.delete_multipart_upload(upload.id).await?;
    for path in storage_paths {
        let _ = state.storage.delete(&path).await;
    }

    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap())
}

/// Implementation of post object
async fn post_object_impl(
    state: AppState,
    bucket: &str,
    key: &str,
    params: MultipartParams,
    body: Body,
) -> Result<Response<Body>> {
    // Check if this is an initiate multipart upload request
    if params.uploads.is_some() {
        return initiate_multipart_upload(state, bucket, key).await;
    }

    // Check if this is a complete multipart upload request
    if let Some(upload_id) = &params.upload_id {
        return complete_multipart_upload(state, bucket, key, upload_id, body).await;
    }

    Err(S3Error::BadRequest("Unknown POST operation".to_string()))
}

/// Initiate multipart upload
async fn initiate_multipart_upload(
    state: AppState,
    bucket: &str,
    key: &str,
) -> Result<Response<Body>> {
    // Verify bucket exists
    state
        .db
        .get_bucket(bucket)
        .await?
        .ok_or_else(|| S3Error::NotFound(format!("Bucket not found: {}", bucket)))?;

    // Generate upload ID
    let upload_id = uuid::Uuid::new_v4().to_string();

    // Create upload record
    state
        .db
        .create_multipart_upload(bucket, key, &upload_id)
        .await?;

    let response = InitiateMultipartUploadResponse {
        bucket: bucket.to_string(),
        key: key.to_string(),
        upload_id,
    };

    let xml = response.to_xml();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml")
        .body(Body::from(xml))
        .unwrap())
}

/// Complete multipart upload
async fn complete_multipart_upload(
    state: AppState,
    bucket: &str,
    key: &str,
    upload_id: &str,
    body: Body,
) -> Result<Response<Body>> {
    // Get the upload
    let upload = state
        .db
        .get_multipart_upload(bucket, upload_id)
        .await?
        .ok_or_else(|| S3Error::NotFound(format!("Upload not found: {}", upload_id)))?;

    // Get all parts
    let parts = state.db.list_upload_parts(upload.id).await?;

    if parts.is_empty() {
        return Err(S3Error::BadRequest("No parts uploaded".to_string()));
    }

    // Get storage paths in order
    let storage_paths: Vec<String> = parts.iter().map(|p| p.storage_path.clone()).collect();

    // Create final storage path
    let final_storage_path = state.storage.create_storage_path(bucket, key);

    // Concatenate all parts
    let (size, etag) = state
        .storage
        .concatenate_files(&storage_paths, &final_storage_path)
        .await?;

    // Create object record
    state
        .db
        .create_object(bucket, key, size, &etag, None, &final_storage_path)
        .await?;

    // Delete upload and parts
    let part_paths = state.db.delete_multipart_upload(upload.id).await?;
    for path in part_paths {
        let _ = state.storage.delete(&path).await;
    }

    let response = CompleteMultipartUploadResponse {
        location: format!("/{}/{}", bucket, key),
        bucket: bucket.to_string(),
        key: key.to_string(),
        etag,
    };

    let xml = response.to_xml();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml")
        .body(Body::from(xml))
        .unwrap())
}

/// Parse HTTP Range header
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

/// Format date for HTTP header
fn format_http_date(date_str: &str) -> String {
    // Convert to HTTP date format
    // For now, return as-is, but in production you'd parse and format properly
    date_str.replace(' ', "T") + "Z"
}

/// Start the HTTP server
pub async fn run_server(state: AppState, port: u16) -> std::result::Result<(), std::io::Error> {
    let app = create_router(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    info!("Starting S3-compatible server on port {}", port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
