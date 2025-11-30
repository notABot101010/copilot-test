//! Database layer for S3 server metadata storage

use sqlx::{sqlite::SqlitePool, FromRow};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Bucket not found: {0}")]
    BucketNotFound(String),
    #[error("Object not found: {0}/{1}")]
    ObjectNotFound(String, String),
    #[error("Bucket already exists: {0}")]
    BucketAlreadyExists(String),
    #[error("Upload not found: {0}")]
    UploadNotFound(String),
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Bucket {
    pub id: i64,
    pub name: String,
    pub owner: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct ObjectMetadata {
    pub id: i64,
    pub bucket_id: i64,
    pub key: String,
    pub size: i64,
    pub etag: String,
    pub content_type: Option<String>,
    pub storage_path: String,
    pub created_at: String,
    pub last_modified: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct MultipartUpload {
    pub id: i64,
    pub upload_id: String,
    pub bucket_id: i64,
    pub key: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct UploadPart {
    pub id: i64,
    pub upload_id: i64,
    pub part_number: i32,
    pub size: i64,
    pub etag: String,
    pub storage_path: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct User {
    pub id: i64,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(url).await?;
        Ok(Self { pool })
    }

    pub async fn init(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                access_key_id TEXT NOT NULL UNIQUE,
                secret_access_key TEXT NOT NULL,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS buckets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                owner TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS objects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                bucket_id INTEGER NOT NULL,
                key TEXT NOT NULL,
                size INTEGER NOT NULL,
                etag TEXT NOT NULL,
                content_type TEXT,
                storage_path TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_modified TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (bucket_id) REFERENCES buckets(id) ON DELETE CASCADE,
                UNIQUE(bucket_id, key)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS multipart_uploads (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                upload_id TEXT NOT NULL UNIQUE,
                bucket_id INTEGER NOT NULL,
                key TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (bucket_id) REFERENCES buckets(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS upload_parts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                upload_id INTEGER NOT NULL,
                part_number INTEGER NOT NULL,
                size INTEGER NOT NULL,
                etag TEXT NOT NULL,
                storage_path TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (upload_id) REFERENCES multipart_uploads(id) ON DELETE CASCADE,
                UNIQUE(upload_id, part_number)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indices for common queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_objects_bucket_key ON objects(bucket_id, key)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_multipart_bucket_key ON multipart_uploads(bucket_id, key)",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // User operations
    pub async fn create_user(
        &self,
        access_key_id: &str,
        secret_access_key: &str,
        name: &str,
    ) -> Result<User> {
        let result = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (access_key_id, secret_access_key, name)
            VALUES (?, ?, ?)
            RETURNING id, access_key_id, secret_access_key, name, created_at
            "#,
        )
        .bind(access_key_id)
        .bind(secret_access_key)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_user_by_access_key(&self, access_key_id: &str) -> Result<Option<User>> {
        let result = sqlx::query_as::<_, User>(
            "SELECT id, access_key_id, secret_access_key, name, created_at FROM users WHERE access_key_id = ?",
        )
        .bind(access_key_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_users(&self) -> Result<Vec<User>> {
        let result = sqlx::query_as::<_, User>(
            "SELECT id, access_key_id, secret_access_key, name, created_at FROM users ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    // Bucket operations
    pub async fn create_bucket(&self, name: &str, owner: &str) -> Result<Bucket> {
        // Check if bucket already exists
        if self.get_bucket(name).await?.is_some() {
            return Err(DatabaseError::BucketAlreadyExists(name.to_string()));
        }

        let result = sqlx::query_as::<_, Bucket>(
            r#"
            INSERT INTO buckets (name, owner)
            VALUES (?, ?)
            RETURNING id, name, owner, created_at
            "#,
        )
        .bind(name)
        .bind(owner)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_bucket(&self, name: &str) -> Result<Option<Bucket>> {
        let result = sqlx::query_as::<_, Bucket>(
            "SELECT id, name, owner, created_at FROM buckets WHERE name = ?",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_buckets(&self) -> Result<Vec<Bucket>> {
        let result = sqlx::query_as::<_, Bucket>(
            "SELECT id, name, owner, created_at FROM buckets ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_buckets_for_owner(&self, owner: &str) -> Result<Vec<Bucket>> {
        let result = sqlx::query_as::<_, Bucket>(
            "SELECT id, name, owner, created_at FROM buckets WHERE owner = ? ORDER BY name",
        )
        .bind(owner)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_bucket(&self, name: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM buckets WHERE name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::BucketNotFound(name.to_string()));
        }

        Ok(())
    }

    // Object operations
    pub async fn create_object(
        &self,
        bucket_name: &str,
        key: &str,
        size: i64,
        etag: &str,
        content_type: Option<&str>,
        storage_path: &str,
    ) -> Result<ObjectMetadata> {
        let bucket = self
            .get_bucket(bucket_name)
            .await?
            .ok_or_else(|| DatabaseError::BucketNotFound(bucket_name.to_string()))?;

        // Use INSERT OR REPLACE to handle updates
        let result = sqlx::query_as::<_, ObjectMetadata>(
            r#"
            INSERT INTO objects (bucket_id, key, size, etag, content_type, storage_path, last_modified)
            VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
            ON CONFLICT(bucket_id, key) DO UPDATE SET
                size = excluded.size,
                etag = excluded.etag,
                content_type = excluded.content_type,
                storage_path = excluded.storage_path,
                last_modified = datetime('now')
            RETURNING id, bucket_id, key, size, etag, content_type, storage_path, created_at, last_modified
            "#,
        )
        .bind(bucket.id)
        .bind(key)
        .bind(size)
        .bind(etag)
        .bind(content_type)
        .bind(storage_path)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_object(&self, bucket_name: &str, key: &str) -> Result<Option<ObjectMetadata>> {
        let bucket = match self.get_bucket(bucket_name).await? {
            Some(b) => b,
            None => return Ok(None),
        };

        let result = sqlx::query_as::<_, ObjectMetadata>(
            r#"
            SELECT id, bucket_id, key, size, etag, content_type, storage_path, created_at, last_modified
            FROM objects
            WHERE bucket_id = ? AND key = ?
            "#,
        )
        .bind(bucket.id)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_objects(
        &self,
        bucket_name: &str,
        prefix: Option<&str>,
        delimiter: Option<&str>,
        max_keys: i32,
        continuation_token: Option<&str>,
    ) -> Result<(Vec<ObjectMetadata>, Vec<String>, Option<String>)> {
        let bucket = self
            .get_bucket(bucket_name)
            .await?
            .ok_or_else(|| DatabaseError::BucketNotFound(bucket_name.to_string()))?;

        let prefix = prefix.unwrap_or("");
        let start_after = continuation_token.unwrap_or("");

        let objects = sqlx::query_as::<_, ObjectMetadata>(
            r#"
            SELECT id, bucket_id, key, size, etag, content_type, storage_path, created_at, last_modified
            FROM objects
            WHERE bucket_id = ? AND key LIKE ? || '%' AND key > ?
            ORDER BY key
            LIMIT ?
            "#,
        )
        .bind(bucket.id)
        .bind(prefix)
        .bind(start_after)
        .bind(max_keys + 1) // Fetch one extra to check if there are more
        .fetch_all(&self.pool)
        .await?;

        // Handle delimiter for common prefixes
        let mut common_prefixes: Vec<String> = Vec::new();
        let mut filtered_objects: Vec<ObjectMetadata> = Vec::new();

        if let Some(delim) = delimiter {
            let prefix_len = prefix.len();
            let mut seen_prefixes = std::collections::HashSet::new();

            for obj in objects {
                let key_after_prefix = &obj.key[prefix_len..];
                if let Some(pos) = key_after_prefix.find(delim) {
                    let common_prefix = format!("{}{}", prefix, &key_after_prefix[..=pos]);
                    if seen_prefixes.insert(common_prefix.clone()) {
                        common_prefixes.push(common_prefix);
                    }
                } else {
                    filtered_objects.push(obj);
                }
            }
        } else {
            filtered_objects = objects;
        }

        // Determine next continuation token
        let has_more = filtered_objects.len() > max_keys as usize;
        let next_token = if has_more {
            filtered_objects.pop();
            filtered_objects.last().map(|o| o.key.clone())
        } else {
            None
        };

        Ok((filtered_objects, common_prefixes, next_token))
    }

    pub async fn delete_object(&self, bucket_name: &str, key: &str) -> Result<Option<String>> {
        let bucket = match self.get_bucket(bucket_name).await? {
            Some(b) => b,
            None => return Err(DatabaseError::BucketNotFound(bucket_name.to_string())),
        };

        // Get the storage path before deleting
        let obj = sqlx::query_as::<_, ObjectMetadata>(
            "SELECT id, bucket_id, key, size, etag, content_type, storage_path, created_at, last_modified FROM objects WHERE bucket_id = ? AND key = ?",
        )
        .bind(bucket.id)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        let storage_path = obj.map(|o| o.storage_path);

        sqlx::query("DELETE FROM objects WHERE bucket_id = ? AND key = ?")
            .bind(bucket.id)
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(storage_path)
    }

    pub async fn object_exists(&self, bucket_name: &str, key: &str) -> Result<bool> {
        Ok(self.get_object(bucket_name, key).await?.is_some())
    }

    // Multipart upload operations
    pub async fn create_multipart_upload(
        &self,
        bucket_name: &str,
        key: &str,
        upload_id: &str,
    ) -> Result<MultipartUpload> {
        let bucket = self
            .get_bucket(bucket_name)
            .await?
            .ok_or_else(|| DatabaseError::BucketNotFound(bucket_name.to_string()))?;

        let result = sqlx::query_as::<_, MultipartUpload>(
            r#"
            INSERT INTO multipart_uploads (upload_id, bucket_id, key)
            VALUES (?, ?, ?)
            RETURNING id, upload_id, bucket_id, key, created_at
            "#,
        )
        .bind(upload_id)
        .bind(bucket.id)
        .bind(key)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_multipart_upload(
        &self,
        bucket_name: &str,
        upload_id: &str,
    ) -> Result<Option<MultipartUpload>> {
        let bucket = match self.get_bucket(bucket_name).await? {
            Some(b) => b,
            None => return Ok(None),
        };

        let result = sqlx::query_as::<_, MultipartUpload>(
            r#"
            SELECT id, upload_id, bucket_id, key, created_at
            FROM multipart_uploads
            WHERE bucket_id = ? AND upload_id = ?
            "#,
        )
        .bind(bucket.id)
        .bind(upload_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn add_upload_part(
        &self,
        upload_db_id: i64,
        part_number: i32,
        size: i64,
        etag: &str,
        storage_path: &str,
    ) -> Result<UploadPart> {
        let result = sqlx::query_as::<_, UploadPart>(
            r#"
            INSERT INTO upload_parts (upload_id, part_number, size, etag, storage_path)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(upload_id, part_number) DO UPDATE SET
                size = excluded.size,
                etag = excluded.etag,
                storage_path = excluded.storage_path
            RETURNING id, upload_id, part_number, size, etag, storage_path, created_at
            "#,
        )
        .bind(upload_db_id)
        .bind(part_number)
        .bind(size)
        .bind(etag)
        .bind(storage_path)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_upload_parts(&self, upload_db_id: i64) -> Result<Vec<UploadPart>> {
        let result = sqlx::query_as::<_, UploadPart>(
            r#"
            SELECT id, upload_id, part_number, size, etag, storage_path, created_at
            FROM upload_parts
            WHERE upload_id = ?
            ORDER BY part_number
            "#,
        )
        .bind(upload_db_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_multipart_upload(&self, upload_db_id: i64) -> Result<Vec<String>> {
        // Get all part storage paths first
        let parts = self.list_upload_parts(upload_db_id).await?;
        let storage_paths: Vec<String> = parts.into_iter().map(|p| p.storage_path).collect();

        // Delete the upload (parts will be cascade deleted)
        sqlx::query("DELETE FROM multipart_uploads WHERE id = ?")
            .bind(upload_db_id)
            .execute(&self.pool)
            .await?;

        Ok(storage_paths)
    }

    pub async fn list_multipart_uploads(&self, bucket_name: &str) -> Result<Vec<MultipartUpload>> {
        let bucket = self
            .get_bucket(bucket_name)
            .await?
            .ok_or_else(|| DatabaseError::BucketNotFound(bucket_name.to_string()))?;

        let result = sqlx::query_as::<_, MultipartUpload>(
            r#"
            SELECT id, upload_id, bucket_id, key, created_at
            FROM multipart_uploads
            WHERE bucket_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(bucket.id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }
}
