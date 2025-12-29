pub mod http_client;
pub mod models;
pub mod ui;

// Re-export commonly used types
pub use http_client::HttpClient;
pub use models::{Bookmark, HistoryEntry, NavigationHistory, Tab};
