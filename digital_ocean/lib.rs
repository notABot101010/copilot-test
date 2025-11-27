//! A Rust client for the DigitalOcean API.
//!
//! This crate provides a type-safe interface to interact with the DigitalOcean API.
//!
//! # Example
//!
//! ```no_run
//! use digital_ocean::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), digital_ocean::Error> {
//!     let client = Client::new(
//!         reqwest::Client::new(),
//!         "your-api-token".to_string(),
//!     );
//!
//!     // List all droplets
//!     let droplets = client.list_droplets(None, None).await?;
//!     for droplet in droplets.droplets {
//!         println!("Droplet: {} ({})", droplet.name, droplet.id);
//!     }
//!
//!     Ok(())
//! }
//! ```

use serde::Deserialize;

mod account;
mod databases;
mod domains;
mod droplets;
mod regions;
mod sizes;
mod ssh_keys;
mod volumes;

pub use account::*;
pub use databases::*;
pub use domains::*;
pub use droplets::*;
pub use regions::*;
pub use sizes::*;
pub use ssh_keys::*;
pub use volumes::*;

/// Base URL for the DigitalOcean API.
pub const API_BASE_URL: &str = "https://api.digitalocean.com/v2";

/// Error types for the DigitalOcean client.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// HTTP request error from reqwest.
    #[error("HTTP request error: {0}")]
    Request(#[from] reqwest::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(serde_json::Error),

    /// API error returned by DigitalOcean.
    #[error("API error (status {0}): {1}")]
    API(u16, ApiError),
}

/// API error structure from DigitalOcean.
/// See: <https://docs.digitalocean.com/reference/api/digitalocean/#section/Introduction/HTTP-Statuses>
#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    /// Error identifier.
    pub id: String,
    /// Error message.
    pub message: String,
    /// Request ID for debugging.
    pub request_id: Option<String>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.id, self.message)
    }
}

/// The main client for interacting with the DigitalOcean API.
pub struct Client {
    pub(crate) http_client: reqwest::Client,
    pub(crate) access_token: String,
}

impl Client {
    /// Creates a new DigitalOcean API client.
    ///
    /// # Arguments
    ///
    /// * `http_client` - A configured reqwest client.
    /// * `access_token` - Your DigitalOcean API access token.
    ///
    /// # Example
    ///
    /// ```
    /// use digital_ocean::Client;
    ///
    /// let client = Client::new(
    ///     reqwest::Client::new(),
    ///     "your-api-token".to_string(),
    /// );
    /// ```
    pub fn new(http_client: reqwest::Client, access_token: String) -> Client {
        Client {
            http_client,
            access_token,
        }
    }
}

/// Helper function to check for API errors in responses.
pub(crate) async fn check_api_error(res: reqwest::Response) -> Result<reqwest::Response, Error> {
    let status_code = res.status().as_u16();
    if status_code >= 400 {
        let api_error: ApiError = res.json().await?;
        return Err(Error::API(status_code, api_error));
    }
    Ok(res)
}

/// Pagination information returned by the API.
#[derive(Debug, Clone, Deserialize)]
pub struct Meta {
    /// Total number of items.
    pub total: u64,
}

/// Links for pagination.
#[derive(Debug, Clone, Deserialize)]
pub struct Links {
    /// Links for page navigation.
    pub pages: Option<Pages>,
}

/// Page navigation links.
#[derive(Debug, Clone, Deserialize)]
pub struct Pages {
    /// Link to the first page.
    pub first: Option<String>,
    /// Link to the previous page.
    pub prev: Option<String>,
    /// Link to the next page.
    pub next: Option<String>,
    /// Link to the last page.
    pub last: Option<String>,
}

#[cfg(test)]
mod tests;
