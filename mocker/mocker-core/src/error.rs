//! Error types for mocker-core

use thiserror::Error;

/// Result type for mocker-core operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for mocker-core operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Image not found: {0}")]
    ImageNotFound(String),

    #[error("Image already exists: {0}")]
    ImageAlreadyExists(String),

    #[error("VM not found: {0}")]
    VmNotFound(String),

    #[error("VM already running: {0}")]
    VmAlreadyRunning(String),

    #[error("Invalid volume mount format: {0}")]
    InvalidVolumeMount(String),

    #[error("Path does not exist: {0}")]
    PathNotFound(String),

    #[error("libkrun error: {0}")]
    Libkrun(String),

    #[error("OCI registry error: {0}")]
    OciRegistry(String),

    #[error("Process error: {0}")]
    Process(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
