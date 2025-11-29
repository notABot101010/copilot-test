use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use tokio::fs;

/// Memory cache using LRU eviction
pub struct MemoryCache {
    cache: Mutex<LruCache<String, Vec<u8>>>,
    max_size_bytes: usize,
    current_size: Mutex<usize>,
}

impl MemoryCache {
    pub fn new(max_size_mb: usize) -> Self {
        let max_entries = NonZeroUsize::new(10000).unwrap();
        Self {
            cache: Mutex::new(LruCache::new(max_entries)),
            max_size_bytes: max_size_mb * 1024 * 1024,
            current_size: Mutex::new(0),
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.cache.lock().get(key).cloned()
    }

    pub fn put(&self, key: String, value: Vec<u8>) {
        let value_size = value.len();
        let mut current = self.current_size.lock();
        let mut cache = self.cache.lock();

        // Evict entries if necessary
        while *current + value_size > self.max_size_bytes && !cache.is_empty() {
            if let Some((_, evicted)) = cache.pop_lru() {
                *current = current.saturating_sub(evicted.len());
            }
        }

        // Only add if it fits
        if value_size <= self.max_size_bytes {
            cache.put(key, value);
            *current += value_size;
        }
    }

    pub fn invalidate(&self, key: &str) {
        let mut cache = self.cache.lock();
        if let Some(removed) = cache.pop(key) {
            let mut current = self.current_size.lock();
            *current = current.saturating_sub(removed.len());
        }
    }

    pub fn clear(&self) {
        self.cache.lock().clear();
        *self.current_size.lock() = 0;
    }
}

/// Disk cache for larger data sets
pub struct DiskCache {
    path: PathBuf,
    max_size_bytes: usize,
}

impl DiskCache {
    pub async fn new(path: &PathBuf, max_size_mb: usize) -> std::io::Result<Self> {
        fs::create_dir_all(path).await?;
        Ok(Self {
            path: path.clone(),
            max_size_bytes: max_size_mb * 1024 * 1024,
        })
    }

    fn key_to_path(&self, key: &str) -> PathBuf {
        // Create a safe filename from the key using SHA256
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = hex::encode(hasher.finalize());
        
        // Use first 2 chars as directory for better distribution
        self.path.join(&hash[..2]).join(&hash)
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let path = self.key_to_path(key);
        fs::read(&path).await.ok()
    }

    pub async fn put(&self, key: &str, value: &[u8]) -> std::io::Result<()> {
        // Simple size check - don't write if too large for cache
        if value.len() > self.max_size_bytes {
            return Ok(());
        }

        let path = self.key_to_path(key);
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        fs::write(&path, value).await
    }

    pub async fn invalidate(&self, key: &str) -> std::io::Result<()> {
        let path = self.key_to_path(key);
        match fs::remove_file(&path).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn clear(&self) -> std::io::Result<()> {
        // Remove all files in cache directory
        let mut entries = fs::read_dir(&self.path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let _ = fs::remove_dir_all(&path).await;
            } else {
                let _ = fs::remove_file(&path).await;
            }
        }
        Ok(())
    }

    /// Get total size of cache in bytes
    #[allow(dead_code)]
    pub async fn size(&self) -> std::io::Result<u64> {
        let mut total = 0u64;
        let mut stack = vec![self.path.clone()];

        while let Some(dir) = stack.pop() {
            if let Ok(mut entries) = fs::read_dir(&dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else if let Ok(metadata) = fs::metadata(&path).await {
                        total += metadata.len();
                    }
                }
            }
        }

        Ok(total)
    }
}

/// Two-level cache combining memory and disk
pub struct TwoLevelCache {
    memory: MemoryCache,
    disk: DiskCache,
}

impl TwoLevelCache {
    pub async fn new(
        cache_path: &PathBuf,
        memory_cache_mb: usize,
        disk_cache_mb: usize,
    ) -> std::io::Result<Self> {
        Ok(Self {
            memory: MemoryCache::new(memory_cache_mb),
            disk: DiskCache::new(cache_path, disk_cache_mb).await?,
        })
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        // Try memory first
        if let Some(data) = self.memory.get(key) {
            return Some(data);
        }

        // Try disk
        if let Some(data) = self.disk.get(key).await {
            // Promote to memory cache
            self.memory.put(key.to_string(), data.clone());
            return Some(data);
        }

        None
    }

    pub async fn put(&self, key: &str, value: Vec<u8>) {
        // Write to both caches
        self.memory.put(key.to_string(), value.clone());
        let _ = self.disk.put(key, &value).await;
    }

    pub async fn invalidate(&self, key: &str) {
        self.memory.invalidate(key);
        let _ = self.disk.invalidate(key).await;
    }

    #[allow(dead_code)]
    pub async fn clear(&self) {
        self.memory.clear();
        let _ = self.disk.clear().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_cache() {
        let cache = MemoryCache::new(1); // 1MB

        cache.put("key1".to_string(), vec![1, 2, 3]);
        assert_eq!(cache.get("key1"), Some(vec![1, 2, 3]));

        cache.invalidate("key1");
        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn test_memory_cache_eviction() {
        let cache = MemoryCache::new(1); // 1MB = 1048576 bytes

        // Add entries until eviction is needed
        let large_value = vec![0u8; 500_000]; // 500KB
        cache.put("key1".to_string(), large_value.clone());
        cache.put("key2".to_string(), large_value.clone());

        // Both should exist
        assert!(cache.get("key1").is_some());
        assert!(cache.get("key2").is_some());

        // Add a third - should evict oldest
        cache.put("key3".to_string(), large_value);
        
        // key3 should exist
        assert!(cache.get("key3").is_some());
    }

    #[tokio::test]
    async fn test_disk_cache() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = DiskCache::new(&temp_dir.path().to_path_buf(), 10).await.unwrap();

        cache.put("key1", b"hello world").await.unwrap();
        assert_eq!(cache.get("key1").await, Some(b"hello world".to_vec()));

        cache.invalidate("key1").await.unwrap();
        assert_eq!(cache.get("key1").await, None);
    }

    #[tokio::test]
    async fn test_two_level_cache() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = TwoLevelCache::new(&temp_dir.path().to_path_buf(), 1, 10)
            .await
            .unwrap();

        cache.put("key1", b"hello".to_vec()).await;
        assert_eq!(cache.get("key1").await, Some(b"hello".to_vec()));

        cache.invalidate("key1").await;
        assert_eq!(cache.get("key1").await, None);
    }
}
