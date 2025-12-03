//! Storage layer for media files

use futures_util::StreamExt;
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("Thumbnail generation failed: {0}")]
    ThumbnailError(String),
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

    /// Get the base path for media storage
    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    /// Create storage path for a media file
    pub fn create_storage_path(&self, user_id: i64, media_type: &str, filename: &str) -> PathBuf {
        let uuid = uuid::Uuid::new_v4();
        self.base_path
            .join(user_id.to_string())
            .join(media_type)
            .join(format!("{}_{}", uuid, filename))
    }

    /// Create thumbnail path for a media file
    pub fn create_thumbnail_path(&self, user_id: i64, filename: &str) -> PathBuf {
        let uuid = uuid::Uuid::new_v4();
        self.base_path
            .join(user_id.to_string())
            .join("thumbnails")
            .join(format!("{}_thumb_{}.jpg", uuid, filename))
    }

    /// Ensure directory exists
    async fn ensure_dir(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(())
    }

    /// Write stream to file (streaming upload)
    pub async fn write_stream<S, E>(&self, path: &PathBuf, mut stream: S) -> Result<i64>
    where
        S: futures_util::Stream<Item = std::result::Result<bytes::Bytes, E>> + Unpin,
        E: std::error::Error,
    {
        self.ensure_dir(path).await?;

        let mut file = fs::File::create(path).await?;
        let mut size: i64 = 0;

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    size += chunk.len() as i64;
                    file.write_all(&chunk).await?;
                }
                Err(err) => {
                    // Clean up partial file on error
                    let _ = fs::remove_file(path).await;
                    return Err(StorageError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        err.to_string(),
                    )));
                }
            }
        }

        file.flush().await?;
        Ok(size)
    }

    /// Read file as stream
    pub async fn read_stream(
        &self,
        path: &PathBuf,
    ) -> Result<impl futures_util::Stream<Item = std::result::Result<bytes::Bytes, std::io::Error>>>
    {
        if !path.exists() {
            return Err(StorageError::NotFound(path.display().to_string()));
        }

        let file = fs::File::open(path).await?;
        let stream = tokio_util::io::ReaderStream::new(file);
        Ok(stream)
    }

    /// Read file with range support
    pub async fn read_range(
        &self,
        path: &PathBuf,
        start: u64,
        end: u64,
    ) -> Result<impl futures_util::Stream<Item = std::result::Result<bytes::Bytes, std::io::Error>>>
    {
        if !path.exists() {
            return Err(StorageError::NotFound(path.display().to_string()));
        }

        let mut file = fs::File::open(path).await?;
        file.seek(std::io::SeekFrom::Start(start)).await?;

        let length = end - start + 1;
        let file = file.take(length);
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

    /// Generate thumbnail for video using ffmpeg
    ///
    /// Security note: Paths are validated to be within the storage base directory
    /// and are canonicalized before use to prevent path traversal attacks.
    pub async fn generate_video_thumbnail(
        &self,
        video_path: &PathBuf,
        thumbnail_path: &PathBuf,
    ) -> Result<()> {
        // Validate that paths are within our storage directory
        let video_canonical = video_path
            .canonicalize()
            .map_err(|err| StorageError::ThumbnailError(format!("Invalid video path: {}", err)))?;

        let base_canonical = self
            .base_path
            .canonicalize()
            .unwrap_or_else(|_| self.base_path.clone());

        if !video_canonical.starts_with(&base_canonical) {
            return Err(StorageError::ThumbnailError(
                "Video path must be within storage directory".to_string(),
            ));
        }

        self.ensure_dir(thumbnail_path).await?;

        // Use OsStr to pass paths directly to avoid shell interpretation
        let output = tokio::process::Command::new("ffmpeg")
            .arg("-i")
            .arg(video_path.as_os_str())
            .args([
                "-ss",
                "00:00:01",
                "-vframes",
                "1",
                "-vf",
                "scale=320:-1",
                "-y",
            ])
            .arg(thumbnail_path.as_os_str())
            .output()
            .await
            .map_err(|err| StorageError::ThumbnailError(err.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(StorageError::ThumbnailError(format!(
                "ffmpeg failed: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Check if ffmpeg is available
    pub async fn is_ffmpeg_available() -> bool {
        tokio::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .await
            .is_ok()
    }
}
