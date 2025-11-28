use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Configuration for the git server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// SSH port to listen on
    pub ssh_port: u16,
    /// Public keys authorized to access the server in OpenSSH format
    pub public_keys: Vec<String>,
}

impl Config {
    /// Load configuration from a JSON file
    pub fn load(path: &PathBuf) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::ReadError(path.clone(), e))?;
        serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(path.clone(), e))
    }

    /// Create a default configuration
    pub fn default_config() -> Self {
        Config {
            ssh_port: 2222,
            public_keys: vec![],
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file {0}: {1}")]
    ReadError(PathBuf, std::io::Error),
    #[error("Failed to parse config file {0}: {1}")]
    ParseError(PathBuf, serde_json::Error),
}
