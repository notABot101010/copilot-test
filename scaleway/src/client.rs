//! Core client module for Scaleway API

use serde::Deserialize;

/// API error returned by Scaleway
#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    /// HTTP status code received (set from HTTP response, not from JSON)
    #[serde(default)]
    pub status_code: u16,
    /// Error message from the API
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "API error ({}): {}", self.status_code, self.message)
    }
}

impl std::error::Error for ApiError {}

/// Error type for Scaleway client operations
#[derive(Debug)]
pub enum Error {
    /// HTTP request error
    Http(reqwest::Error),
    /// JSON serialization/deserialization error
    Json(reqwest::Error),
    /// API error from Scaleway
    API(ApiError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Http(e) => write!(f, "HTTP error: {}", e),
            Error::Json(e) => write!(f, "JSON error: {}", e),
            Error::API(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Http(e) => Some(e),
            Error::Json(e) => Some(e),
            Error::API(e) => Some(e),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(err)
    }
}

/// Scaleway API client
pub struct Client {
    pub(crate) http_client: reqwest::Client,
    pub(crate) secret_access_key: String,
    pub(crate) default_region: Option<String>,
    pub(crate) default_project_id: Option<String>,
}

impl Client {
    /// Create a new Scaleway API client
    ///
    /// # Arguments
    ///
    /// * `http_client` - The reqwest HTTP client to use
    /// * `secret_access_key` - The Scaleway secret access key
    /// * `default_project_id` - Optional default project ID
    /// * `default_region` - Optional default region (e.g., "fr-par", "nl-ams", "pl-waw")
    pub fn new(
        http_client: reqwest::Client,
        secret_access_key: String,
        default_project_id: Option<String>,
        default_region: Option<String>,
    ) -> Client {
        Client {
            http_client,
            secret_access_key,
            default_region,
            default_project_id,
        }
    }

    /// Get the default region, or return an error if not set
    pub(crate) fn get_default_region(&self) -> Result<&str, Error> {
        self.default_region.as_deref().ok_or_else(|| {
            Error::API(ApiError {
                status_code: 0,
                message: "No default region set".to_string(),
            })
        })
    }

    /// Get the default project ID, or return an error if not set
    pub(crate) fn get_default_project_id(&self) -> Result<&str, Error> {
        self.default_project_id.as_deref().ok_or_else(|| {
            Error::API(ApiError {
                status_code: 0,
                message: "No default project ID set".to_string(),
            })
        })
    }
}

/// Check if the response is an API error and convert it
pub(crate) async fn check_api_error(res: reqwest::Response) -> Result<reqwest::Response, Error> {
    let status_code = res.status().as_u16();
    if status_code >= 400 {
        let mut api_error: ApiError = res.json().await.map_err(Error::Json)?;
        api_error.status_code = status_code;
        return Err(Error::API(api_error));
    }
    Ok(res)
}
