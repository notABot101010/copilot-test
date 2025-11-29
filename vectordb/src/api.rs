use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Method, Request, StatusCode},
    middleware::{self, Next},
    response::{Json, Response},
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tower_http::cors::{Any, CorsLayer};

use crate::database::Database;
use crate::filter::FilterExpr;
use crate::index::{DistanceMetric, Document, NamespaceIndex};
use crate::search::{RankBy, SearchEngine};
use crate::storage::S3Storage;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub storage: Arc<S3Storage>,
}

// ============ Request/Response Types ============

#[derive(Debug, Deserialize)]
pub struct UpsertRequest {
    pub documents: Vec<DocumentInput>,
    #[serde(default)]
    pub distance_metric: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentInput {
    pub id: String,
    #[serde(default)]
    pub vector: Option<Vec<f32>>,
    #[serde(flatten)]
    pub attributes: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub rank_by: serde_json::Value,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default)]
    pub filters: Option<serde_json::Value>,
    #[serde(default)]
    pub include_attributes: Option<Vec<String>>,
    #[serde(default)]
    pub include_vector: bool,
}

fn default_top_k() -> usize {
    10
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub results: Vec<QueryResult>,
    pub total_count: usize,
}

#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub id: String,
    pub score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Serialize)]
pub struct NamespaceResponse {
    pub name: String,
    pub document_count: i64,
    pub distance_metric: String,
    pub vector_dimensions: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector: Option<Vec<f32>>,
    #[serde(flatten)]
    pub attributes: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteDocumentsRequest {
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ============ API Key Endpoints ============

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub id: i64,
    pub key: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyInfo {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

/// Create a new API key
async fn create_api_key(
    State(state): State<AppState>,
    Json(body): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Generate a random API key
    let key = generate_api_key();
    let key_hash = hash_api_key(&key);

    let api_key = state
        .db
        .create_api_key(&key_hash, &body.name)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(Json(CreateApiKeyResponse {
        id: api_key.id,
        key,
        name: api_key.name,
    }))
}

/// List all API keys
async fn list_api_keys(
    State(state): State<AppState>,
) -> Result<Json<Vec<ApiKeyInfo>>, (StatusCode, Json<ErrorResponse>)> {
    let keys = state.db.list_api_keys().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    Ok(Json(
        keys.into_iter()
            .map(|k| ApiKeyInfo {
                id: k.id,
                name: k.name,
                created_at: k.created_at,
                last_used_at: k.last_used_at,
            })
            .collect(),
    ))
}

/// Delete an API key
async fn delete_api_key(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.db.delete_api_key(id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============ Namespace Endpoints ============

/// List all namespaces
async fn list_namespaces(
    State(state): State<AppState>,
) -> Result<Json<Vec<NamespaceResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let namespaces = state.db.list_namespaces().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    Ok(Json(
        namespaces
            .into_iter()
            .map(|ns| NamespaceResponse {
                name: ns.name,
                document_count: ns.document_count,
                distance_metric: ns.distance_metric,
                vector_dimensions: ns.vector_dimensions,
            })
            .collect(),
    ))
}

/// Get namespace info
async fn get_namespace(
    State(state): State<AppState>,
    Path(namespace): Path<String>,
) -> Result<Json<NamespaceResponse>, (StatusCode, Json<ErrorResponse>)> {
    let ns = state.db.get_namespace(&namespace).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    match ns {
        Some(ns) => Ok(Json(NamespaceResponse {
            name: ns.name,
            document_count: ns.document_count,
            distance_metric: ns.distance_metric,
            vector_dimensions: ns.vector_dimensions,
        })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Namespace not found".to_string(),
            }),
        )),
    }
}

/// Delete a namespace
async fn delete_namespace(
    State(state): State<AppState>,
    Path(namespace): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Delete from storage
    state.storage.delete_namespace(&namespace).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    // Delete from database
    state.db.delete_namespace(&namespace).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Upsert documents to a namespace
async fn upsert_documents(
    State(state): State<AppState>,
    Path(namespace): Path<String>,
    Json(body): Json<UpsertRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    if body.documents.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "No documents provided".to_string(),
            }),
        ));
    }

    // Load or create namespace index
    let mut index = state
        .storage
        .load_namespace(&namespace)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?
        .unwrap_or_else(|| {
            let metric = body
                .distance_metric
                .as_deref()
                .map(DistanceMetric::from_str)
                .unwrap_or_default();
            NamespaceIndex::new(metric)
        });

    // Upsert documents
    for doc_input in body.documents {
        let doc = Document {
            id: doc_input.id,
            vector: doc_input.vector,
            attributes: doc_input.attributes,
        };
        index.upsert_document(doc);
    }

    // Save to storage
    state.storage.save_namespace(&namespace, &index).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    // Update database metadata
    let distance_metric = body.distance_metric.as_deref().unwrap_or("cosine_distance");
    let vector_dims = index.vector_dimensions.map(|d| d as i64);

    state
        .db
        .upsert_namespace(&namespace, distance_metric, vector_dims)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    state
        .db
        .update_document_count(&namespace, index.document_count() as i64)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(Json(serde_json::json!({
        "status": "ok",
        "document_count": index.document_count()
    })))
}

/// Query a namespace
async fn query_namespace(
    State(state): State<AppState>,
    Path(namespace): Path<String>,
    Json(body): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Load namespace index
    let index = state
        .storage
        .load_namespace(&namespace)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Namespace not found".to_string(),
                }),
            )
        })?;

    // Parse rank_by
    let rank_by = RankBy::from_json(&body.rank_by).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )
    })?;

    // Parse filters
    let filter = if let Some(ref filter_json) = body.filters {
        Some(FilterExpr::from_json(filter_json).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?)
    } else {
        None
    };

    // Execute search
    let results = SearchEngine::search(&index, &rank_by, filter.as_ref(), body.top_k);

    // Format response
    let query_results: Vec<QueryResult> = results
        .into_iter()
        .map(|r| {
            let attributes = if let Some(ref include_attrs) = body.include_attributes {
                let mut filtered = serde_json::Map::new();
                for attr in include_attrs {
                    if let Some(value) = r.document.attributes.get(attr) {
                        filtered.insert(attr.clone(), value.clone());
                    }
                }
                Some(filtered)
            } else {
                None
            };

            QueryResult {
                id: r.document.id,
                score: r.score,
                vector: if body.include_vector {
                    r.document.vector
                } else {
                    None
                },
                attributes,
            }
        })
        .collect();

    Ok(Json(QueryResponse {
        total_count: query_results.len(),
        results: query_results,
    }))
}

/// Get documents from a namespace
async fn get_documents(
    State(state): State<AppState>,
    Path(namespace): Path<String>,
) -> Result<Json<Vec<DocumentResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let index = state
        .storage
        .load_namespace(&namespace)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Namespace not found".to_string(),
                }),
            )
        })?;

    let docs: Vec<DocumentResponse> = index
        .documents
        .values()
        .map(|d| DocumentResponse {
            id: d.id.clone(),
            vector: d.vector.clone(),
            attributes: d.attributes.clone(),
        })
        .collect();

    Ok(Json(docs))
}

/// Get a single document
async fn get_document(
    State(state): State<AppState>,
    Path((namespace, doc_id)): Path<(String, String)>,
) -> Result<Json<DocumentResponse>, (StatusCode, Json<ErrorResponse>)> {
    let index = state
        .storage
        .load_namespace(&namespace)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Namespace not found".to_string(),
                }),
            )
        })?;

    let doc = index.get_document(&doc_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Document not found".to_string(),
            }),
        )
    })?;

    Ok(Json(DocumentResponse {
        id: doc.id.clone(),
        vector: doc.vector.clone(),
        attributes: doc.attributes.clone(),
    }))
}

/// Delete documents from a namespace
async fn delete_documents(
    State(state): State<AppState>,
    Path(namespace): Path<String>,
    Json(body): Json<DeleteDocumentsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    if body.ids.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "No document IDs provided".to_string(),
            }),
        ));
    }

    // Load namespace index
    let mut index = state
        .storage
        .load_namespace(&namespace)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Namespace not found".to_string(),
                }),
            )
        })?;

    // Delete documents
    let mut deleted_count = 0;
    for id in &body.ids {
        if index.remove_document(id).is_some() {
            deleted_count += 1;
        }
    }

    // Save updated index
    state.storage.save_namespace(&namespace, &index).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    // Update document count
    state
        .db
        .update_document_count(&namespace, index.document_count() as i64)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(Json(serde_json::json!({
        "status": "ok",
        "deleted_count": deleted_count
    })))
}

// ============ Auth Middleware ============

/// Authentication middleware
async fn auth_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Check if any API keys exist - if not, allow all requests (initial setup)
    let keys = match state.db.list_api_keys().await {
        Ok(k) => k,
        Err(_) => return next.run(request).await,
    };

    if keys.is_empty() {
        return next.run(request).await;
    }

    // Get the Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            let token = &auth[7..];
            let key_hash = hash_api_key(token);

            if let Ok(Some(_)) = state.db.validate_api_key(&key_hash).await {
                return next.run(request).await;
            }
        }
        _ => {}
    }

    // Return 401 Unauthorized
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::WWW_AUTHENTICATE, "Bearer")
        .body(Body::from("Unauthorized"))
        .unwrap()
}

// ============ Utility Functions ============

fn generate_api_key() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    format!("vdb_{}", hex::encode(bytes))
}

fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

// ============ Router Setup ============

pub fn create_router(state: AppState) -> Router {
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    // API key management routes (no auth required for initial key creation)
    let api_key_routes = Router::new()
        .route("/", get(list_api_keys).post(create_api_key))
        .route("/{id}", delete(delete_api_key));

    // Protected namespace routes
    let namespace_routes = Router::new()
        .route("/", get(list_namespaces))
        .route("/{namespace}", get(get_namespace).post(upsert_documents).delete(delete_namespace))
        .route("/{namespace}/query", post(query_namespace))
        .route("/{namespace}/documents", get(get_documents).delete(delete_documents))
        .route("/{namespace}/documents/{doc_id}", get(get_document))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .nest("/api/keys", api_key_routes)
        .nest("/api/namespaces", namespace_routes)
        .layer(cors)
        .with_state(state)
}

/// Run the HTTP server
pub async fn run_server(
    port: u16,
    db: Arc<Database>,
    storage: Arc<S3Storage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = AppState { db, storage };
    let app = create_router(state);
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_upsert_request_parsing() {
        let json = json!({
            "documents": [
                {
                    "id": "1",
                    "vector": [0.1, 0.2],
                    "text": "Hello world",
                    "category": "greeting"
                }
            ],
            "distance_metric": "cosine_distance"
        });

        let request: UpsertRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.documents.len(), 1);
        assert_eq!(request.documents[0].id, "1");
        assert!(request.documents[0].vector.is_some());
        assert_eq!(
            request.distance_metric,
            Some("cosine_distance".to_string())
        );
    }

    #[test]
    fn test_query_request_parsing() {
        let json = json!({
            "rank_by": ["vector", "ANN", [0.1, 0.2]],
            "top_k": 5,
            "filters": ["category", "Eq", "greeting"],
            "include_attributes": ["text"]
        });

        let request: QueryRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.top_k, 5);
        assert!(request.filters.is_some());
        assert!(request.include_attributes.is_some());
    }

    #[test]
    fn test_api_key_generation() {
        let key = generate_api_key();
        assert!(key.starts_with("vdb_"));
        assert_eq!(key.len(), 68); // "vdb_" + 64 hex chars

        let hash = hash_api_key(&key);
        assert_eq!(hash.len(), 64); // SHA256 hex
    }
}
