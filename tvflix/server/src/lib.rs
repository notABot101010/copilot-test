//! TVflix - Self-hosted media center library
//!
//! This module exposes the server components for integration testing.

pub mod auth;
pub mod database;
pub mod handlers;
pub mod storage;

use std::sync::Arc;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<database::Database>,
    pub storage: Arc<storage::Storage>,
}
