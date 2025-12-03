//! File storage operations with streaming support

use bytes::Bytes;
use futures_util::Stream;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio_util::io::ReaderStream;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("File not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Clone)]
pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Get the full path for a storage key
    fn get_path(&self, key: &str) -> PathBuf {
        self.base_path.join(key)
    }

    /// Create a unique storage path for a new object
    pub fn create_storage_path(&self, bucket: &str, key: &str) -> String {
        let hash = crate::auth::sha256_hex_public(format!("{}/{}", bucket, key).as_bytes());
        format!("{}/{}/{}", &hash[0..2], &hash[2..4], hash)
    }

    /// Create a unique storage path for a multipart upload part
    pub fn create_part_storage_path(&self, upload_id: &str, part_number: i32) -> String {
        let hash =
            crate::auth::sha256_hex_public(format!("{}/{}", upload_id, part_number).as_bytes());
        format!("parts/{}/{}/{}", &hash[0..2], &hash[2..4], hash)
    }

    /// Write data from a stream to storage
    pub async fn write_stream<S, E>(
        &self,
        storage_path: &str,
        mut stream: S,
    ) -> Result<(i64, String)>
    where
        S: Stream<Item = std::result::Result<Bytes, E>> + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        use aws_lc_rs::digest;
        use futures_util::StreamExt;

        let path = self.get_path(storage_path);

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let file = File::create(&path).await?;
        let mut writer = BufWriter::new(file);
        let mut hasher = digest::Context::new(&digest::SHA256);
        let mut total_size: i64 = 0;

        while let Some(chunk) = stream.next().await {
            let data = chunk
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
            hasher.update(&data);
            total_size += data.len() as i64;
            writer.write_all(&data).await?;
        }

        writer.flush().await?;

        let hash = hasher.finish();
        let etag = format!("\"{}\"", hex::encode(hash.as_ref()));

        Ok((total_size, etag))
    }

    /// Write data directly to storage
    pub async fn write(&self, storage_path: &str, data: &[u8]) -> Result<String> {
        use aws_lc_rs::digest;

        let path = self.get_path(storage_path);

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let file = File::create(&path).await?;
        let mut writer = BufWriter::new(file);
        writer.write_all(data).await?;
        writer.flush().await?;

        let hash = digest::digest(&digest::SHA256, data);
        let etag = format!("\"{}\"", hex::encode(hash.as_ref()));

        Ok(etag)
    }

    /// Read data from storage as a stream
    pub async fn read_stream(
        &self,
        storage_path: &str,
    ) -> Result<impl Stream<Item = std::result::Result<Bytes, std::io::Error>>> {
        let path = self.get_path(storage_path);

        if !path.exists() {
            return Err(StorageError::NotFound(storage_path.to_string()));
        }

        let file = File::open(&path).await?;
        let reader = BufReader::new(file);
        let stream = ReaderStream::new(reader);

        Ok(stream)
    }

    /// Read a range of bytes from storage
    pub async fn read_range(
        &self,
        storage_path: &str,
        start: u64,
        end: u64,
    ) -> Result<impl Stream<Item = std::result::Result<Bytes, std::io::Error>>> {
        let path = self.get_path(storage_path);

        if !path.exists() {
            return Err(StorageError::NotFound(storage_path.to_string()));
        }

        let mut file = File::open(&path).await?;

        // Seek to start position
        use tokio::io::AsyncSeekExt;
        file.seek(std::io::SeekFrom::Start(start)).await?;

        // Create a take reader to limit bytes
        let len = end - start + 1;
        let reader = BufReader::new(file.take(len));
        let stream = ReaderStream::new(reader);

        Ok(stream)
    }

    /// Read entire file into memory (for small files)
    pub async fn read(&self, storage_path: &str) -> Result<Vec<u8>> {
        let path = self.get_path(storage_path);

        if !path.exists() {
            return Err(StorageError::NotFound(storage_path.to_string()));
        }

        let data = fs::read(&path).await?;
        Ok(data)
    }

    /// Delete a file from storage
    pub async fn delete(&self, storage_path: &str) -> Result<()> {
        let path = self.get_path(storage_path);

        if path.exists() {
            fs::remove_file(&path).await?;
        }

        Ok(())
    }

    /// Get file size
    pub async fn size(&self, storage_path: &str) -> Result<u64> {
        let path = self.get_path(storage_path);

        if !path.exists() {
            return Err(StorageError::NotFound(storage_path.to_string()));
        }

        let metadata = fs::metadata(&path).await?;
        Ok(metadata.len())
    }

    /// Check if file exists
    pub async fn exists(&self, storage_path: &str) -> bool {
        self.get_path(storage_path).exists()
    }

    /// Concatenate multiple files into one (for completing multipart uploads)
    pub async fn concatenate_files(
        &self,
        parts: &[String],
        output_path: &str,
    ) -> Result<(i64, String)> {
        use aws_lc_rs::digest;

        let out_path = self.get_path(output_path);

        // Create parent directories if needed
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let out_file = File::create(&out_path).await?;
        let mut writer = BufWriter::new(out_file);
        let mut hasher = digest::Context::new(&digest::SHA256);
        let mut total_size: i64 = 0;

        for part_path in parts {
            let path = self.get_path(part_path);
            let mut file = File::open(&path).await?;
            let mut buffer = [0u8; 64 * 1024]; // 64KB buffer

            loop {
                let bytes_read = file.read(&mut buffer).await?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
                writer.write_all(&buffer[..bytes_read]).await?;
                total_size += bytes_read as i64;
            }
        }

        writer.flush().await?;

        // For multipart uploads, the ETag is different (MD5 of MD5s)
        // But for simplicity, we'll use the concatenated SHA256
        let hash = hasher.finish();
        let etag = format!("\"{}-{}\"", hex::encode(hash.as_ref()), parts.len());

        Ok((total_size, etag))
    }

    /// Get the base path
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_write_and_read() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().to_path_buf());

        let data = b"Hello, World!";
        let path = "test/file.txt";

        let etag = storage.write(path, data).await.unwrap();
        assert!(!etag.is_empty());

        let read_data = storage.read(path).await.unwrap();
        assert_eq!(read_data, data);
    }

    #[tokio::test]
    async fn test_write_stream() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().to_path_buf());

        let chunks: Vec<std::result::Result<Bytes, std::io::Error>> =
            vec![Ok(Bytes::from("Hello, ")), Ok(Bytes::from("World!"))];
        let input_stream = stream::iter(chunks);

        let path = "test/stream.txt";
        let (size, etag) = storage.write_stream(path, input_stream).await.unwrap();

        assert_eq!(size, 13);
        assert!(!etag.is_empty());

        let read_data = storage.read(path).await.unwrap();
        assert_eq!(read_data, b"Hello, World!");
    }

    #[tokio::test]
    async fn test_delete() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().to_path_buf());

        let path = "test/to_delete.txt";
        storage.write(path, b"test").await.unwrap();
        assert!(storage.exists(path).await);

        storage.delete(path).await.unwrap();
        assert!(!storage.exists(path).await);
    }
}
