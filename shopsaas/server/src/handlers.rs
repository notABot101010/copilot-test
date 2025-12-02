//! HTTP handlers for ShopSaaS API

use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_http::services::ServeDir;

use crate::auth::{generate_token, hash_password, verify_password};
use crate::database::{DatabaseError, OrderStatus, ProductType};
use crate::storage::StorageError;
use crate::AppState;

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
            DatabaseError::StoreNotFound(_) => ApiError::NotFound(err.to_string()),
            DatabaseError::ProductNotFound(_) => ApiError::NotFound(err.to_string()),
            DatabaseError::PageNotFound(_) => ApiError::NotFound(err.to_string()),
            DatabaseError::OrderNotFound(_) => ApiError::NotFound(err.to_string()),
            DatabaseError::InvalidSession => ApiError::Unauthorized(err.to_string()),
            DatabaseError::UserAlreadyExists(_) => ApiError::BadRequest(err.to_string()),
            DatabaseError::StoreSlugExists(_) => ApiError::BadRequest(err.to_string()),
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
    email: String,
    password: String,
    name: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
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
    email: String,
    name: String,
}

#[derive(Deserialize)]
pub struct CreateStoreRequest {
    name: String,
    slug: String,
    description: Option<String>,
    currency: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateStoreRequest {
    name: String,
    description: Option<String>,
    logo_url: Option<String>,
    currency: String,
}

#[derive(Deserialize)]
pub struct CreateProductRequest {
    name: String,
    description: Option<String>,
    price: i64,
    product_type: String,
    stock_quantity: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdateProductRequest {
    name: String,
    description: Option<String>,
    price: i64,
    stock_quantity: Option<i64>,
    image_url: Option<String>,
    is_active: bool,
}

#[derive(Deserialize)]
pub struct CreatePageRequest {
    title: String,
    slug: String,
    is_homepage: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdatePageRequest {
    title: String,
    slug: String,
    is_published: bool,
    is_homepage: bool,
}

#[derive(Deserialize)]
pub struct CreateBlockRequest {
    block_type: String,
    content: String,
    position: i32,
}

#[derive(Deserialize)]
pub struct UpdateBlockRequest {
    content: String,
    position: i32,
}

#[derive(Deserialize)]
pub struct ReorderBlocksRequest {
    block_ids: Vec<i64>,
}

#[derive(Deserialize)]
pub struct UpdateOrderStatusRequest {
    status: String,
}

#[derive(Deserialize)]
pub struct CreateContactRequest {
    email: String,
    name: Option<String>,
    subscribed_to_newsletter: Option<bool>,
}

#[derive(Deserialize)]
pub struct CreateMailingListRequest {
    name: String,
    description: Option<String>,
}

#[derive(Deserialize)]
pub struct AddToMailingListRequest {
    contact_id: i64,
}

// Storefront types
#[derive(Deserialize)]
pub struct CustomerRegisterRequest {
    email: String,
    password: String,
    name: String,
}

#[derive(Deserialize)]
pub struct CustomerLoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct CustomerAuthResponse {
    token: String,
    customer: CustomerResponse,
}

#[derive(Serialize)]
pub struct CustomerResponse {
    id: i64,
    email: String,
    name: String,
}

#[derive(Deserialize)]
pub struct CartItem {
    product_id: i64,
    quantity: i32,
}

#[derive(Deserialize)]
pub struct CheckoutRequest {
    items: Vec<CartItem>,
    customer_email: String,
    customer_name: String,
    shipping_address: Option<String>,
}

#[derive(Serialize)]
pub struct OrderWithItems {
    #[serde(flatten)]
    order: crate::database::Order,
    items: Vec<crate::database::OrderItem>,
}

#[derive(Serialize)]
pub struct PageWithBlocks {
    #[serde(flatten)]
    page: crate::database::Page,
    blocks: Vec<crate::database::PageBlock>,
}

/// Create the router for the API
pub fn create_router(state: AppState) -> Router {
    let static_path = std::env::current_dir()
        .map(|p| p.join("static/dist"))
        .unwrap_or_else(|_| PathBuf::from("static/dist"));

    Router::new()
        // Admin auth routes
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/me", get(get_current_user))
        // Store routes
        .route("/api/stores", get(list_stores))
        .route("/api/stores", post(create_store))
        .route("/api/stores/:id", get(get_store))
        .route("/api/stores/:id", put(update_store))
        .route("/api/stores/:id", delete(delete_store))
        // Product routes
        .route("/api/stores/:store_id/products", get(list_products))
        .route("/api/stores/:store_id/products", post(create_product))
        .route("/api/stores/:store_id/products/:id", get(get_product))
        .route("/api/stores/:store_id/products/:id", put(update_product))
        .route("/api/stores/:store_id/products/:id", delete(delete_product))
        // Page routes
        .route("/api/stores/:store_id/pages", get(list_pages))
        .route("/api/stores/:store_id/pages", post(create_page))
        .route("/api/stores/:store_id/pages/:id", get(get_page))
        .route("/api/stores/:store_id/pages/:id", put(update_page))
        .route("/api/stores/:store_id/pages/:id", delete(delete_page))
        // Page block routes
        .route("/api/pages/:page_id/blocks", get(list_blocks))
        .route("/api/pages/:page_id/blocks", post(create_block))
        .route("/api/pages/:page_id/blocks/reorder", post(reorder_blocks))
        .route("/api/blocks/:id", put(update_block))
        .route("/api/blocks/:id", delete(delete_block))
        // Order routes
        .route("/api/stores/:store_id/orders", get(list_orders))
        .route("/api/stores/:store_id/orders/:id", get(get_order))
        .route("/api/stores/:store_id/orders/:id/status", put(update_order_status))
        // Customer routes (admin view)
        .route("/api/stores/:store_id/customers", get(list_customers))
        // Contact routes
        .route("/api/stores/:store_id/contacts", get(list_contacts))
        .route("/api/stores/:store_id/contacts", post(create_contact))
        .route("/api/stores/:store_id/contacts/:id", delete(delete_contact))
        // Mailing list routes
        .route("/api/stores/:store_id/mailing-lists", get(list_mailing_lists))
        .route("/api/stores/:store_id/mailing-lists", post(create_mailing_list))
        .route("/api/stores/:store_id/mailing-lists/:id", get(get_mailing_list))
        .route("/api/stores/:store_id/mailing-lists/:id", delete(delete_mailing_list))
        .route("/api/stores/:store_id/mailing-lists/:id/subscribers", post(add_to_mailing_list))
        .route("/api/stores/:store_id/mailing-lists/:id/subscribers/:contact_id", delete(remove_from_mailing_list))
        // Storefront routes
        .route("/api/storefront/:slug", get(get_storefront))
        .route("/api/storefront/:slug/products", get(storefront_list_products))
        .route("/api/storefront/:slug/products/:id", get(storefront_get_product))
        .route("/api/storefront/:slug/pages", get(storefront_list_pages))
        .route("/api/storefront/:slug/pages/:page_slug", get(storefront_get_page))
        .route("/api/storefront/:slug/auth/register", post(customer_register))
        .route("/api/storefront/:slug/auth/login", post(customer_login))
        .route("/api/storefront/:slug/auth/logout", post(customer_logout))
        .route("/api/storefront/:slug/auth/me", get(get_current_customer))
        .route("/api/storefront/:slug/checkout", post(checkout))
        .route("/api/storefront/:slug/orders", get(customer_orders))
        .route("/api/storefront/:slug/orders/:id", get(customer_get_order))
        .route("/api/storefront/:slug/subscribe", post(newsletter_subscribe))
        // Serve static files
        .nest_service("/", ServeDir::new(static_path).append_index_html_on_directories(true))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100MB
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// Helper functions
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

async fn get_customer_from_headers(state: &AppState, headers: &HeaderMap) -> Result<i64> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Unauthorized("Invalid authorization header format".to_string()))?;

    let session = state
        .db
        .get_customer_session_by_token(token)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid or expired session".to_string()))?;

    Ok(session.customer_id)
}

async fn verify_store_ownership(state: &AppState, store_id: i64, user_id: i64) -> Result<()> {
    let store = state
        .db
        .get_store_by_id(store_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Store not found".to_string()))?;

    if store.user_id != user_id {
        return Err(ApiError::NotFound("Store not found".to_string()));
    }

    Ok(())
}

// Admin Auth handlers
async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>> {
    if req.email.is_empty() || req.password.is_empty() || req.name.is_empty() {
        return Err(ApiError::BadRequest("Email, password, and name are required".to_string()));
    }

    if req.password.len() < 6 {
        return Err(ApiError::BadRequest("Password must be at least 6 characters".to_string()));
    }

    let password_hash = hash_password(&req.password);
    let user = state.db.create_user(&req.email, &password_hash, &req.name).await?;

    let token = generate_token();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .ok_or_else(|| ApiError::Internal("Failed to calculate expiry".to_string()))?;

    state.db.create_session(user.id, &token, &expires_at).await?;

    Ok(Json(AuthResponse {
        token,
        user: UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
        },
    }))
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    let user = state
        .db
        .get_user_by_email(&req.email)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid email or password".to_string()))?;

    if !verify_password(&req.password, &user.password_hash) {
        return Err(ApiError::Unauthorized("Invalid email or password".to_string()));
    }

    let token = generate_token();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .ok_or_else(|| ApiError::Internal("Failed to calculate expiry".to_string()))?;

    state.db.create_session(user.id, &token, &expires_at).await?;

    Ok(Json(AuthResponse {
        token,
        user: UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
        },
    }))
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Result<StatusCode> {
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                state.db.delete_session(token).await?;
            }
        }
    }
    Ok(StatusCode::NO_CONTENT)
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
        email: user.email,
        name: user.name,
    }))
}

// Store handlers
async fn list_stores(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::database::Store>>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    let stores = state.db.list_stores_by_user(user_id).await?;
    Ok(Json(stores))
}

async fn create_store(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateStoreRequest>,
) -> Result<Json<crate::database::Store>> {
    let user_id = get_user_from_headers(&state, &headers).await?;

    if req.name.is_empty() || req.slug.is_empty() {
        return Err(ApiError::BadRequest("Name and slug are required".to_string()));
    }

    let currency = req.currency.unwrap_or_else(|| "USD".to_string());
    let store = state.db.create_store(user_id, &req.name, &req.slug, req.description.as_deref(), &currency).await?;
    Ok(Json(store))
}

async fn get_store(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<crate::database::Store>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, id, user_id).await?;

    let store = state.db.get_store_by_id(id).await?.ok_or_else(|| ApiError::NotFound("Store not found".to_string()))?;
    Ok(Json(store))
}

async fn update_store(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(req): Json<UpdateStoreRequest>,
) -> Result<Json<crate::database::Store>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, id, user_id).await?;

    let store = state.db.update_store(id, &req.name, req.description.as_deref(), req.logo_url.as_deref(), &req.currency).await?;
    Ok(Json(store))
}

async fn delete_store(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, id, user_id).await?;

    state.db.delete_store(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Product handlers
async fn list_products(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(store_id): Path<i64>,
) -> Result<Json<Vec<crate::database::Product>>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let products = state.db.list_products_by_store(store_id, false).await?;
    Ok(Json(products))
}

async fn create_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(store_id): Path<i64>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<crate::database::Product>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let product_type: ProductType = req.product_type.parse().map_err(|err| ApiError::BadRequest(err))?;
    let product = state.db.create_product(
        store_id,
        &req.name,
        req.description.as_deref(),
        req.price,
        product_type,
        req.stock_quantity,
        None,
        None,
    ).await?;
    Ok(Json(product))
}

async fn get_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((store_id, id)): Path<(i64, i64)>,
) -> Result<Json<crate::database::Product>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let product = state.db.get_product_by_id(id).await?.ok_or_else(|| ApiError::NotFound("Product not found".to_string()))?;
    if product.store_id != store_id {
        return Err(ApiError::NotFound("Product not found".to_string()));
    }
    Ok(Json(product))
}

async fn update_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((store_id, id)): Path<(i64, i64)>,
    Json(req): Json<UpdateProductRequest>,
) -> Result<Json<crate::database::Product>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let existing = state.db.get_product_by_id(id).await?.ok_or_else(|| ApiError::NotFound("Product not found".to_string()))?;
    if existing.store_id != store_id {
        return Err(ApiError::NotFound("Product not found".to_string()));
    }

    let product = state.db.update_product(
        id,
        &req.name,
        req.description.as_deref(),
        req.price,
        req.stock_quantity,
        req.image_url.as_deref(),
        existing.digital_file_path.as_deref(),
        req.is_active,
    ).await?;
    Ok(Json(product))
}

async fn delete_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((store_id, id)): Path<(i64, i64)>,
) -> Result<StatusCode> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let product = state.db.get_product_by_id(id).await?.ok_or_else(|| ApiError::NotFound("Product not found".to_string()))?;
    if product.store_id != store_id {
        return Err(ApiError::NotFound("Product not found".to_string()));
    }

    state.db.delete_product(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Page handlers
async fn list_pages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(store_id): Path<i64>,
) -> Result<Json<Vec<crate::database::Page>>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let pages = state.db.list_pages_by_store(store_id).await?;
    Ok(Json(pages))
}

async fn create_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(store_id): Path<i64>,
    Json(req): Json<CreatePageRequest>,
) -> Result<Json<crate::database::Page>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let page = state.db.create_page(store_id, &req.title, &req.slug, req.is_homepage.unwrap_or(false)).await?;
    Ok(Json(page))
}

async fn get_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((store_id, id)): Path<(i64, i64)>,
) -> Result<Json<PageWithBlocks>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let page = state.db.get_page_by_id(id).await?.ok_or_else(|| ApiError::NotFound("Page not found".to_string()))?;
    if page.store_id != store_id {
        return Err(ApiError::NotFound("Page not found".to_string()));
    }

    let blocks = state.db.list_page_blocks(id).await?;
    Ok(Json(PageWithBlocks { page, blocks }))
}

async fn update_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((store_id, id)): Path<(i64, i64)>,
    Json(req): Json<UpdatePageRequest>,
) -> Result<Json<crate::database::Page>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let existing = state.db.get_page_by_id(id).await?.ok_or_else(|| ApiError::NotFound("Page not found".to_string()))?;
    if existing.store_id != store_id {
        return Err(ApiError::NotFound("Page not found".to_string()));
    }

    let page = state.db.update_page(id, &req.title, &req.slug, req.is_published, req.is_homepage).await?;
    Ok(Json(page))
}

async fn delete_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((store_id, id)): Path<(i64, i64)>,
) -> Result<StatusCode> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let page = state.db.get_page_by_id(id).await?.ok_or_else(|| ApiError::NotFound("Page not found".to_string()))?;
    if page.store_id != store_id {
        return Err(ApiError::NotFound("Page not found".to_string()));
    }

    state.db.delete_page(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Block handlers
async fn list_blocks(
    State(state): State<AppState>,
    Path(page_id): Path<i64>,
) -> Result<Json<Vec<crate::database::PageBlock>>> {
    let blocks = state.db.list_page_blocks(page_id).await?;
    Ok(Json(blocks))
}

async fn create_block(
    State(state): State<AppState>,
    Path(page_id): Path<i64>,
    Json(req): Json<CreateBlockRequest>,
) -> Result<Json<crate::database::PageBlock>> {
    let block = state.db.create_page_block(page_id, &req.block_type, &req.content, req.position).await?;
    Ok(Json(block))
}

async fn update_block(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateBlockRequest>,
) -> Result<Json<crate::database::PageBlock>> {
    let block = state.db.update_page_block(id, &req.content, req.position).await?;
    Ok(Json(block))
}

async fn delete_block(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    state.db.delete_page_block(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn reorder_blocks(
    State(state): State<AppState>,
    Path(page_id): Path<i64>,
    Json(req): Json<ReorderBlocksRequest>,
) -> Result<StatusCode> {
    state.db.reorder_page_blocks(page_id, &req.block_ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Order handlers
async fn list_orders(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(store_id): Path<i64>,
) -> Result<Json<Vec<crate::database::Order>>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let orders = state.db.list_orders_by_store(store_id).await?;
    Ok(Json(orders))
}

async fn get_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((store_id, id)): Path<(i64, i64)>,
) -> Result<Json<OrderWithItems>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let order = state.db.get_order_by_id(id).await?.ok_or_else(|| ApiError::NotFound("Order not found".to_string()))?;
    if order.store_id != store_id {
        return Err(ApiError::NotFound("Order not found".to_string()));
    }

    let items = state.db.get_order_items(id).await?;
    Ok(Json(OrderWithItems { order, items }))
}

async fn update_order_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((store_id, id)): Path<(i64, i64)>,
    Json(req): Json<UpdateOrderStatusRequest>,
) -> Result<Json<crate::database::Order>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let order = state.db.get_order_by_id(id).await?.ok_or_else(|| ApiError::NotFound("Order not found".to_string()))?;
    if order.store_id != store_id {
        return Err(ApiError::NotFound("Order not found".to_string()));
    }

    let status: OrderStatus = req.status.parse().map_err(|err| ApiError::BadRequest(err))?;
    let updated = state.db.update_order_status(id, status).await?;
    Ok(Json(updated))
}

// Customer handlers (admin view)
async fn list_customers(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(store_id): Path<i64>,
) -> Result<Json<Vec<crate::database::Customer>>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let customers = state.db.list_customers_by_store(store_id).await?;
    Ok(Json(customers))
}

// Contact handlers
async fn list_contacts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(store_id): Path<i64>,
) -> Result<Json<Vec<crate::database::Contact>>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let contacts = state.db.list_contacts_by_store(store_id).await?;
    Ok(Json(contacts))
}

async fn create_contact(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(store_id): Path<i64>,
    Json(req): Json<CreateContactRequest>,
) -> Result<Json<crate::database::Contact>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let contact = state.db.create_or_update_contact(
        store_id,
        &req.email,
        req.name.as_deref(),
        req.subscribed_to_newsletter.unwrap_or(false),
    ).await?;
    Ok(Json(contact))
}

async fn delete_contact(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((store_id, id)): Path<(i64, i64)>,
) -> Result<StatusCode> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    state.db.delete_contact(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Mailing list handlers
async fn list_mailing_lists(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(store_id): Path<i64>,
) -> Result<Json<Vec<crate::database::MailingList>>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let lists = state.db.list_mailing_lists_by_store(store_id).await?;
    Ok(Json(lists))
}

async fn create_mailing_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(store_id): Path<i64>,
    Json(req): Json<CreateMailingListRequest>,
) -> Result<Json<crate::database::MailingList>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let list = state.db.create_mailing_list(store_id, &req.name, req.description.as_deref()).await?;
    Ok(Json(list))
}

#[derive(Serialize)]
struct MailingListWithSubscribers {
    #[serde(flatten)]
    list: crate::database::MailingList,
    subscribers: Vec<crate::database::Contact>,
}

async fn get_mailing_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((store_id, id)): Path<(i64, i64)>,
) -> Result<Json<MailingListWithSubscribers>> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let list = state.db.get_mailing_list_by_id(id).await?.ok_or_else(|| ApiError::NotFound("Mailing list not found".to_string()))?;
    if list.store_id != store_id {
        return Err(ApiError::NotFound("Mailing list not found".to_string()));
    }

    let subscribers = state.db.get_mailing_list_subscribers(id).await?;
    Ok(Json(MailingListWithSubscribers { list, subscribers }))
}

async fn delete_mailing_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((store_id, id)): Path<(i64, i64)>,
) -> Result<StatusCode> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    let list = state.db.get_mailing_list_by_id(id).await?.ok_or_else(|| ApiError::NotFound("Mailing list not found".to_string()))?;
    if list.store_id != store_id {
        return Err(ApiError::NotFound("Mailing list not found".to_string()));
    }

    state.db.delete_mailing_list(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_to_mailing_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((store_id, id)): Path<(i64, i64)>,
    Json(req): Json<AddToMailingListRequest>,
) -> Result<StatusCode> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    state.db.add_to_mailing_list(id, req.contact_id).await?;
    Ok(StatusCode::CREATED)
}

async fn remove_from_mailing_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((store_id, id, contact_id)): Path<(i64, i64, i64)>,
) -> Result<StatusCode> {
    let user_id = get_user_from_headers(&state, &headers).await?;
    verify_store_ownership(&state, store_id, user_id).await?;

    state.db.remove_from_mailing_list(id, contact_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Storefront handlers
#[derive(Serialize)]
struct StorefrontInfo {
    id: i64,
    name: String,
    description: Option<String>,
    logo_url: Option<String>,
    currency: String,
}

async fn get_storefront(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<StorefrontInfo>> {
    let store = state.db.get_store_by_slug(&slug).await?.ok_or_else(|| ApiError::NotFound("Store not found".to_string()))?;
    Ok(Json(StorefrontInfo {
        id: store.id,
        name: store.name,
        description: store.description,
        logo_url: store.logo_url,
        currency: store.currency,
    }))
}

async fn storefront_list_products(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<crate::database::Product>>> {
    let store = state.db.get_store_by_slug(&slug).await?.ok_or_else(|| ApiError::NotFound("Store not found".to_string()))?;
    let products = state.db.list_products_by_store(store.id, true).await?;
    Ok(Json(products))
}

async fn storefront_get_product(
    State(state): State<AppState>,
    Path((slug, id)): Path<(String, i64)>,
) -> Result<Json<crate::database::Product>> {
    let store = state.db.get_store_by_slug(&slug).await?.ok_or_else(|| ApiError::NotFound("Store not found".to_string()))?;
    let product = state.db.get_product_by_id(id).await?.ok_or_else(|| ApiError::NotFound("Product not found".to_string()))?;
    
    if product.store_id != store.id || !product.is_active {
        return Err(ApiError::NotFound("Product not found".to_string()));
    }
    
    Ok(Json(product))
}

async fn storefront_list_pages(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<crate::database::Page>>> {
    let store = state.db.get_store_by_slug(&slug).await?.ok_or_else(|| ApiError::NotFound("Store not found".to_string()))?;
    let pages = state.db.list_pages_by_store(store.id).await?;
    let published: Vec<_> = pages.into_iter().filter(|p| p.is_published).collect();
    Ok(Json(published))
}

async fn storefront_get_page(
    State(state): State<AppState>,
    Path((slug, page_slug)): Path<(String, String)>,
) -> Result<Json<PageWithBlocks>> {
    let store = state.db.get_store_by_slug(&slug).await?.ok_or_else(|| ApiError::NotFound("Store not found".to_string()))?;
    let page = state.db.get_page_by_slug(store.id, &page_slug).await?.ok_or_else(|| ApiError::NotFound("Page not found".to_string()))?;
    
    if !page.is_published {
        return Err(ApiError::NotFound("Page not found".to_string()));
    }
    
    let blocks = state.db.list_page_blocks(page.id).await?;
    Ok(Json(PageWithBlocks { page, blocks }))
}

async fn customer_register(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(req): Json<CustomerRegisterRequest>,
) -> Result<Json<CustomerAuthResponse>> {
    let store = state.db.get_store_by_slug(&slug).await?.ok_or_else(|| ApiError::NotFound("Store not found".to_string()))?;

    if req.email.is_empty() || req.password.is_empty() || req.name.is_empty() {
        return Err(ApiError::BadRequest("Email, password, and name are required".to_string()));
    }

    let password_hash = hash_password(&req.password);
    let customer = state.db.create_customer(store.id, &req.email, &password_hash, &req.name).await?;

    let token = generate_token();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .ok_or_else(|| ApiError::Internal("Failed to calculate expiry".to_string()))?;

    state.db.create_customer_session(customer.id, &token, &expires_at).await?;

    // Also add as contact
    state.db.create_or_update_contact(store.id, &req.email, Some(&req.name), false).await?;

    Ok(Json(CustomerAuthResponse {
        token,
        customer: CustomerResponse {
            id: customer.id,
            email: customer.email,
            name: customer.name,
        },
    }))
}

async fn customer_login(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(req): Json<CustomerLoginRequest>,
) -> Result<Json<CustomerAuthResponse>> {
    let store = state.db.get_store_by_slug(&slug).await?.ok_or_else(|| ApiError::NotFound("Store not found".to_string()))?;

    let customer = state.db.get_customer_by_email(store.id, &req.email).await?.ok_or_else(|| ApiError::Unauthorized("Invalid email or password".to_string()))?;

    if !verify_password(&req.password, &customer.password_hash) {
        return Err(ApiError::Unauthorized("Invalid email or password".to_string()));
    }

    let token = generate_token();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .ok_or_else(|| ApiError::Internal("Failed to calculate expiry".to_string()))?;

    state.db.create_customer_session(customer.id, &token, &expires_at).await?;

    Ok(Json(CustomerAuthResponse {
        token,
        customer: CustomerResponse {
            id: customer.id,
            email: customer.email,
            name: customer.name,
        },
    }))
}

async fn customer_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode> {
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                state.db.delete_customer_session(token).await?;
            }
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn get_current_customer(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CustomerResponse>> {
    let customer_id = get_customer_from_headers(&state, &headers).await?;
    let customer = state.db.get_customer_by_id(customer_id).await?.ok_or_else(|| ApiError::NotFound("Customer not found".to_string()))?;

    Ok(Json(CustomerResponse {
        id: customer.id,
        email: customer.email,
        name: customer.name,
    }))
}

async fn checkout(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    headers: HeaderMap,
    Json(req): Json<CheckoutRequest>,
) -> Result<Json<OrderWithItems>> {
    let store = state.db.get_store_by_slug(&slug).await?.ok_or_else(|| ApiError::NotFound("Store not found".to_string()))?;

    // Try to get customer if authenticated
    let customer_id = get_customer_from_headers(&state, &headers).await.ok();

    if req.items.is_empty() {
        return Err(ApiError::BadRequest("Cart is empty".to_string()));
    }

    // Calculate total and validate products
    let mut total: i64 = 0;
    let mut order_items_data = Vec::new();

    for item in &req.items {
        let product = state.db.get_product_by_id(item.product_id).await?.ok_or_else(|| ApiError::NotFound(format!("Product {} not found", item.product_id)))?;
        
        if product.store_id != store.id || !product.is_active {
            return Err(ApiError::BadRequest(format!("Product {} not available", item.product_id)));
        }

        // Check stock for physical products
        if product.product_type == "physical" {
            if let Some(stock) = product.stock_quantity {
                if stock < item.quantity as i64 {
                    return Err(ApiError::BadRequest(format!("Insufficient stock for {}", product.name)));
                }
            }
        }

        total += product.price * item.quantity as i64;
        order_items_data.push((product, item.quantity));
    }

    // Create order
    let order = state.db.create_order(
        store.id,
        customer_id,
        &req.customer_email,
        &req.customer_name,
        req.shipping_address.as_deref(),
        total,
    ).await?;

    // Add order items and update stock
    let mut items = Vec::new();
    for (product, quantity) in order_items_data {
        let item = state.db.add_order_item(order.id, product.id, &product.name, quantity, product.price).await?;
        items.push(item);

        // Decrease stock for physical products
        if product.product_type == "physical" && product.stock_quantity.is_some() {
            state.db.update_product_stock(product.id, -(quantity as i64)).await?;
        }
    }

    // Add customer as contact
    state.db.create_or_update_contact(store.id, &req.customer_email, Some(&req.customer_name), false).await?;

    Ok(Json(OrderWithItems { order, items }))
}

async fn customer_orders(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::database::Order>>> {
    let store = state.db.get_store_by_slug(&slug).await?.ok_or_else(|| ApiError::NotFound("Store not found".to_string()))?;
    let customer_id = get_customer_from_headers(&state, &headers).await?;
    
    let customer = state.db.get_customer_by_id(customer_id).await?.ok_or_else(|| ApiError::NotFound("Customer not found".to_string()))?;
    if customer.store_id != store.id {
        return Err(ApiError::Unauthorized("Invalid customer".to_string()));
    }

    let orders = state.db.list_orders_by_customer(customer_id).await?;
    Ok(Json(orders))
}

async fn customer_get_order(
    State(state): State<AppState>,
    Path((slug, id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<OrderWithItems>> {
    let store = state.db.get_store_by_slug(&slug).await?.ok_or_else(|| ApiError::NotFound("Store not found".to_string()))?;
    let customer_id = get_customer_from_headers(&state, &headers).await?;

    let customer = state.db.get_customer_by_id(customer_id).await?.ok_or_else(|| ApiError::NotFound("Customer not found".to_string()))?;
    if customer.store_id != store.id {
        return Err(ApiError::Unauthorized("Invalid customer".to_string()));
    }

    let order = state.db.get_order_by_id(id).await?.ok_or_else(|| ApiError::NotFound("Order not found".to_string()))?;
    if order.customer_id != Some(customer_id) {
        return Err(ApiError::NotFound("Order not found".to_string()));
    }

    let items = state.db.get_order_items(id).await?;
    Ok(Json(OrderWithItems { order, items }))
}

#[derive(Deserialize)]
pub struct NewsletterSubscribeRequest {
    email: String,
    name: Option<String>,
}

async fn newsletter_subscribe(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(req): Json<NewsletterSubscribeRequest>,
) -> Result<StatusCode> {
    let store = state.db.get_store_by_slug(&slug).await?.ok_or_else(|| ApiError::NotFound("Store not found".to_string()))?;

    state.db.create_or_update_contact(store.id, &req.email, req.name.as_deref(), true).await?;
    Ok(StatusCode::CREATED)
}
