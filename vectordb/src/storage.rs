use std::path::PathBuf;

use aws_config::Region;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;

use crate::cache::TwoLevelCache;
use crate::error::{Result, VectorDbError};
use crate::index::NamespaceIndex;

/// S3 storage backend with two-level caching
pub struct S3Storage {
    client: Client,
    bucket: String,
    cache: TwoLevelCache,
}

impl S3Storage {
    /// Create a new S3 storage backend
    pub async fn new(
        bucket: &str,
        endpoint: Option<&str>,
        region: &str,
        cache_path: &PathBuf,
        memory_cache_mb: usize,
        disk_cache_mb: usize,
    ) -> Result<Self> {
        let config = aws_config::from_env()
            .region(Region::new(region.to_string()))
            .load()
            .await;

        let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&config);
        
        // Configure endpoint if provided (for S3-compatible services)
        if let Some(ep) = endpoint {
            s3_config_builder = s3_config_builder
                .endpoint_url(ep)
                .force_path_style(true);
        }

        let client = Client::from_conf(s3_config_builder.build());

        let cache = TwoLevelCache::new(cache_path, memory_cache_mb, disk_cache_mb)
            .await
            .map_err(|e| VectorDbError::Storage(e.to_string()))?;

        Ok(Self {
            client,
            bucket: bucket.to_string(),
            cache,
        })
    }

    /// Create with explicit credentials (for testing)
    #[allow(dead_code)]
    pub async fn new_with_credentials(
        bucket: &str,
        endpoint: Option<&str>,
        region: &str,
        access_key: &str,
        secret_key: &str,
        cache_path: &PathBuf,
        memory_cache_mb: usize,
        disk_cache_mb: usize,
    ) -> Result<Self> {
        let credentials = Credentials::new(access_key, secret_key, None, None, "manual");

        let mut s3_config_builder = aws_sdk_s3::config::Builder::new()
            .credentials_provider(credentials)
            .region(Region::new(region.to_string()));

        if let Some(ep) = endpoint {
            s3_config_builder = s3_config_builder
                .endpoint_url(ep)
                .force_path_style(true);
        }

        let client = Client::from_conf(s3_config_builder.build());

        let cache = TwoLevelCache::new(cache_path, memory_cache_mb, disk_cache_mb)
            .await
            .map_err(|e| VectorDbError::Storage(e.to_string()))?;

        Ok(Self {
            client,
            bucket: bucket.to_string(),
            cache,
        })
    }

    /// Get S3 key for a namespace index
    fn namespace_key(namespace: &str) -> String {
        format!("namespaces/{}/index.json", namespace)
    }

    /// Load a namespace index from S3 (with caching)
    pub async fn load_namespace(&self, namespace: &str) -> Result<Option<NamespaceIndex>> {
        let key = Self::namespace_key(namespace);

        // Try cache first
        if let Some(data) = self.cache.get(&key).await {
            tracing::debug!("Cache hit for namespace: {}", namespace);
            let index: NamespaceIndex = serde_json::from_slice(&data)?;
            return Ok(Some(index));
        }

        tracing::debug!("Cache miss for namespace: {}, loading from S3", namespace);

        // Load from S3
        let result = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await;

        match result {
            Ok(output) => {
                let data = output
                    .body
                    .collect()
                    .await
                    .map_err(|e| VectorDbError::S3(e.to_string()))?
                    .into_bytes()
                    .to_vec();

                // Store in cache
                self.cache.put(&key, data.clone()).await;

                let index: NamespaceIndex = serde_json::from_slice(&data)?;
                Ok(Some(index))
            }
            Err(err) => {
                let service_err = err.into_service_error();
                if service_err.is_no_such_key() {
                    Ok(None)
                } else {
                    Err(VectorDbError::S3(format!("{:?}", service_err)))
                }
            }
        }
    }

    /// Save a namespace index to S3 (and update cache)
    pub async fn save_namespace(&self, namespace: &str, index: &NamespaceIndex) -> Result<()> {
        let key = Self::namespace_key(namespace);
        let data = serde_json::to_vec(index)?;

        // Update cache first
        self.cache.put(&key, data.clone()).await;

        // Save to S3
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(data))
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| VectorDbError::S3(format!("{:?}", e.into_service_error())))?;

        Ok(())
    }

    /// Delete a namespace from S3 (and cache)
    pub async fn delete_namespace(&self, namespace: &str) -> Result<()> {
        let key = Self::namespace_key(namespace);

        // Invalidate cache
        self.cache.invalidate(&key).await;

        // Delete from S3
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| VectorDbError::S3(format!("{:?}", e.into_service_error())))?;

        Ok(())
    }

    /// List all namespaces
    pub async fn list_namespaces(&self) -> Result<Vec<String>> {
        let prefix = "namespaces/";

        let mut namespaces = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix)
                .delimiter("/");

            if let Some(token) = continuation_token.take() {
                request = request.continuation_token(token);
            }

            let result = request
                .send()
                .await
                .map_err(|e| VectorDbError::S3(format!("{:?}", e.into_service_error())))?;

            // Extract namespace names from common prefixes
            let prefixes = result.common_prefixes();
            for p in prefixes {
                if let Some(prefix_str) = p.prefix() {
                    // Extract namespace name from "namespaces/{name}/"
                    if let Some(name) = prefix_str
                        .strip_prefix("namespaces/")
                        .and_then(|s| s.strip_suffix("/"))
                    {
                        namespaces.push(name.to_string());
                    }
                }
            }

            if result.is_truncated() == Some(true) {
                continuation_token = result.next_continuation_token().map(|s| s.to_string());
            } else {
                break;
            }
        }

        Ok(namespaces)
    }
}

#[cfg(test)]
mod tests {
    // S3 tests would require a mock or local S3 instance
    // These are integration tests that would be run separately
}
