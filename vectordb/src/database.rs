use sqlx::{Row, SqlitePool};

/// Database manager for VectorDB
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

/// API key information
#[derive(Debug, Clone)]
pub struct ApiKey {
    pub id: i64,
    pub key_hash: String,
    pub name: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

/// Namespace metadata
#[derive(Debug, Clone, serde::Serialize)]
pub struct NamespaceInfo {
    pub name: String,
    pub document_count: i64,
    pub distance_metric: String,
    pub vector_dimensions: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

impl Database {
    /// Connect to the SQLite database
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Database { pool })
    }

    /// Initialize the database schema
    pub async fn init(&self) -> Result<(), sqlx::Error> {
        // API keys table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS api_keys (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key_hash TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_used_at DATETIME
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Namespaces table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS namespaces (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                document_count INTEGER NOT NULL DEFAULT 0,
                distance_metric TEXT NOT NULL DEFAULT 'cosine_distance',
                vector_dimensions INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_namespaces_name ON namespaces(name)")
            .execute(&self.pool)
            .await;

        Ok(())
    }

    // ============ API Key Methods ============

    /// Create a new API key
    pub async fn create_api_key(&self, key_hash: &str, name: &str) -> Result<ApiKey, sqlx::Error> {
        let result = sqlx::query("INSERT INTO api_keys (key_hash, name) VALUES (?, ?)")
            .bind(key_hash)
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(ApiKey {
            id: result.last_insert_rowid(),
            key_hash: key_hash.to_string(),
            name: name.to_string(),
            created_at: chrono_now(),
            last_used_at: None,
        })
    }

    /// Validate an API key and update last used time
    pub async fn validate_api_key(&self, key_hash: &str) -> Result<Option<ApiKey>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, key_hash, name, created_at, last_used_at FROM api_keys WHERE key_hash = ?",
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = &row {
            // Update last used time
            sqlx::query("UPDATE api_keys SET last_used_at = CURRENT_TIMESTAMP WHERE id = ?")
                .bind(r.get::<i64, _>("id"))
                .execute(&self.pool)
                .await?;
        }

        Ok(row.map(|r| ApiKey {
            id: r.get("id"),
            key_hash: r.get("key_hash"),
            name: r.get("name"),
            created_at: r.get("created_at"),
            last_used_at: r.get("last_used_at"),
        }))
    }

    /// List all API keys
    pub async fn list_api_keys(&self) -> Result<Vec<ApiKey>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, key_hash, name, created_at, last_used_at FROM api_keys ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ApiKey {
                id: r.get("id"),
                key_hash: r.get("key_hash"),
                name: r.get("name"),
                created_at: r.get("created_at"),
                last_used_at: r.get("last_used_at"),
            })
            .collect())
    }

    /// Delete an API key
    pub async fn delete_api_key(&self, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM api_keys WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ============ Namespace Methods ============

    /// Create or update a namespace
    pub async fn upsert_namespace(
        &self,
        name: &str,
        distance_metric: &str,
        vector_dimensions: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO namespaces (name, distance_metric, vector_dimensions)
            VALUES (?, ?, ?)
            ON CONFLICT(name) DO UPDATE SET
                distance_metric = excluded.distance_metric,
                vector_dimensions = COALESCE(excluded.vector_dimensions, namespaces.vector_dimensions),
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(name)
        .bind(distance_metric)
        .bind(vector_dimensions)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get namespace info
    pub async fn get_namespace(&self, name: &str) -> Result<Option<NamespaceInfo>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT name, document_count, distance_metric, vector_dimensions, created_at, updated_at FROM namespaces WHERE name = ?",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| NamespaceInfo {
            name: r.get("name"),
            document_count: r.get("document_count"),
            distance_metric: r.get("distance_metric"),
            vector_dimensions: r.get("vector_dimensions"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    /// List all namespaces
    pub async fn list_namespaces(&self) -> Result<Vec<NamespaceInfo>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT name, document_count, distance_metric, vector_dimensions, created_at, updated_at FROM namespaces ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| NamespaceInfo {
                name: r.get("name"),
                document_count: r.get("document_count"),
                distance_metric: r.get("distance_metric"),
                vector_dimensions: r.get("vector_dimensions"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    /// Update document count for a namespace
    pub async fn update_document_count(
        &self,
        name: &str,
        count: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE namespaces SET document_count = ?, updated_at = CURRENT_TIMESTAMP WHERE name = ?",
        )
        .bind(count)
        .bind(name)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete a namespace
    pub async fn delete_namespace(&self, name: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM namespaces WHERE name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

/// Get current timestamp as string
fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| format!("{}", d.as_secs()))
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> Database {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        db.init().await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_api_key_crud() {
        let db = setup_test_db().await;

        // Create
        let key = db.create_api_key("hash123", "test-key").await.unwrap();
        assert_eq!(key.name, "test-key");

        // Validate
        let validated = db.validate_api_key("hash123").await.unwrap();
        assert!(validated.is_some());
        assert_eq!(validated.unwrap().name, "test-key");

        // Invalid key
        let invalid = db.validate_api_key("invalid").await.unwrap();
        assert!(invalid.is_none());

        // List
        let keys = db.list_api_keys().await.unwrap();
        assert_eq!(keys.len(), 1);

        // Delete
        let deleted = db.delete_api_key(key.id).await.unwrap();
        assert!(deleted);

        let keys = db.list_api_keys().await.unwrap();
        assert_eq!(keys.len(), 0);
    }

    #[tokio::test]
    async fn test_namespace_crud() {
        let db = setup_test_db().await;

        // Upsert
        db.upsert_namespace("test-ns", "cosine_distance", Some(128))
            .await
            .unwrap();

        // Get
        let ns = db.get_namespace("test-ns").await.unwrap();
        assert!(ns.is_some());
        let ns = ns.unwrap();
        assert_eq!(ns.name, "test-ns");
        assert_eq!(ns.distance_metric, "cosine_distance");
        assert_eq!(ns.vector_dimensions, Some(128));

        // Update document count
        db.update_document_count("test-ns", 100).await.unwrap();
        let ns = db.get_namespace("test-ns").await.unwrap().unwrap();
        assert_eq!(ns.document_count, 100);

        // List
        let namespaces = db.list_namespaces().await.unwrap();
        assert_eq!(namespaces.len(), 1);

        // Delete
        let deleted = db.delete_namespace("test-ns").await.unwrap();
        assert!(deleted);

        let ns = db.get_namespace("test-ns").await.unwrap();
        assert!(ns.is_none());
    }
}
