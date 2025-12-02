//! Storage utilities for ShopSaaS

use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("File not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Debug, Clone)]
pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Create a storage path for a product file
    pub fn create_product_path(&self, store_id: i64, filename: &str) -> PathBuf {
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
        
        let unique_name = format!("{}_{}.{}", uuid::Uuid::new_v4(), filename.replace(' ', "_"), ext);
        self.base_path.join("stores").join(store_id.to_string()).join("products").join(unique_name)
    }

    /// Create a storage path for a digital product file
    pub fn create_digital_product_path(&self, store_id: i64, filename: &str) -> PathBuf {
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
        
        let unique_name = format!("{}_{}", uuid::Uuid::new_v4(), filename.replace(' ', "_"));
        self.base_path.join("stores").join(store_id.to_string()).join("digital").join(format!("{}.{}", unique_name, ext))
    }

    /// Create a storage path for store logos
    pub fn create_logo_path(&self, store_id: i64, filename: &str) -> PathBuf {
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        
        let unique_name = format!("{}.{}", uuid::Uuid::new_v4(), ext);
        self.base_path.join("stores").join(store_id.to_string()).join("logo").join(unique_name)
    }

    /// Write a stream to a file
    pub async fn write_stream<S, E>(&self, path: &PathBuf, mut stream: S) -> Result<i64>
    where
        S: futures_util::Stream<Item = std::result::Result<bytes::Bytes, E>> + Unpin,
        E: std::error::Error,
    {
        // Create parent directories
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut file = fs::File::create(path).await?;
        let mut total_bytes: i64 = 0;

        while let Some(chunk) = stream.next().await {
            if let Ok(bytes) = chunk {
                total_bytes += bytes.len() as i64;
                file.write_all(&bytes).await?;
            }
        }

        file.flush().await?;
        Ok(total_bytes)
    }

    /// Write bytes to a file
    pub async fn write_bytes(&self, path: &PathBuf, data: &[u8]) -> Result<()> {
        // Create parent directories
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(path, data).await?;
        Ok(())
    }

    /// Read a file as stream
    pub async fn read_stream(&self, path: &PathBuf) -> Result<impl futures_util::Stream<Item = std::result::Result<bytes::Bytes, std::io::Error>>> {
        if !path.exists() {
            return Err(StorageError::NotFound(path.display().to_string()));
        }

        let file = fs::File::open(path).await?;
        let stream = tokio_util::io::ReaderStream::new(file);
        Ok(stream)
    }

    /// Get file size
    pub async fn get_file_size(&self, path: &PathBuf) -> Result<u64> {
        let metadata = fs::metadata(path).await?;
        Ok(metadata.len())
    }

    /// Delete a file
    pub async fn delete(&self, path: &PathBuf) -> Result<()> {
        if path.exists() {
            fs::remove_file(path).await?;
        }
        Ok(())
    }

    /// Check if a file exists
    pub async fn exists(&self, path: &PathBuf) -> bool {
        path.exists()
    }
}
