use crate::document::Document;
use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

const LAST_OPENED_DOCUMENT_KEY: &str = "last_opened_document";

pub struct Storage {
    pool: SqlitePool,
}

impl Storage {
    pub async fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let connection_string = format!("sqlite://{}", db_path.display());
        let options = SqliteConnectOptions::from_str(&connection_string)?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                parent_id TEXT
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS document_access (
                document_id TEXT NOT NULL,
                accessed_at INTEGER NOT NULL,
                PRIMARY KEY (document_id, accessed_at)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    fn get_db_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        
        let db_path = PathBuf::from(home).join(".tuinotions").join("tuinotion.db");
        Ok(db_path)
    }

    pub async fn save_document(&self, document: &Document) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO documents (id, title, content, parent_id)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(document.id.to_string())
        .bind(&document.title)
        .bind(&document.content)
        .bind(document.parent_id.map(|id| id.to_string()))
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn load_document(&self, doc_id: Uuid) -> Result<Document> {
        let row = sqlx::query(
            r#"
            SELECT id, title, content, parent_id
            FROM documents
            WHERE id = ?
            "#,
        )
        .bind(doc_id.to_string())
        .fetch_one(&self.pool)
        .await?;

        let id = Uuid::parse_str(row.get::<String, _>("id").as_str())?;
        let parent_id = row
            .get::<Option<String>, _>("parent_id")
            .and_then(|s| Uuid::parse_str(&s).ok());

        Ok(Document {
            id,
            title: row.get("title"),
            content: row.get("content"),
            parent_id,
        })
    }

    pub async fn delete_document(&self, doc_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM documents WHERE id = ?
            "#,
        )
        .bind(doc_id.to_string())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn load_all_documents(&self) -> Result<Vec<Document>> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, content, parent_id
            FROM documents
            ORDER BY title
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut documents = Vec::new();
        for row in rows {
            let id = Uuid::parse_str(row.get::<String, _>("id").as_str())?;
            let parent_id = row
                .get::<Option<String>, _>("parent_id")
                .and_then(|s| Uuid::parse_str(&s).ok());

            documents.push(Document {
                id,
                title: row.get("title"),
                content: row.get("content"),
                parent_id,
            });
        }

        Ok(documents)
    }

    pub async fn set_last_opened_document(&self, doc_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO settings (key, value)
            VALUES (?, ?)
            "#,
        )
        .bind(LAST_OPENED_DOCUMENT_KEY)
        .bind(doc_id.to_string())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_last_opened_document(&self) -> Result<Option<Uuid>> {
        let row = sqlx::query(
            r#"
            SELECT value FROM settings WHERE key = ?
            "#,
        )
        .bind(LAST_OPENED_DOCUMENT_KEY)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let value: String = row.get("value");
                Ok(Some(Uuid::parse_str(&value)?))
            }
            None => Ok(None),
        }
    }

    pub async fn record_document_access(&self, doc_id: Uuid) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        sqlx::query(
            r#"
            INSERT INTO document_access (document_id, accessed_at)
            VALUES (?, ?)
            "#,
        )
        .bind(doc_id.to_string())
        .bind(timestamp)
        .execute(&self.pool)
        .await?;

        // Clean up old access records periodically (keep only the last 200 per document)
        // This prevents unbounded table growth
        sqlx::query(
            r#"
            DELETE FROM document_access
            WHERE rowid IN (
                SELECT rowid FROM document_access
                WHERE document_id = ?
                ORDER BY accessed_at DESC
                LIMIT -1 OFFSET 200
            )
            "#,
        )
        .bind(doc_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_recently_accessed_documents(&self, limit: usize) -> Result<Vec<Uuid>> {
        let rows = sqlx::query(
            r#"
            SELECT document_id, MAX(accessed_at) as last_access
            FROM document_access
            GROUP BY document_id
            ORDER BY last_access DESC
            LIMIT ?
            "#,
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut doc_ids = Vec::new();
        for row in rows {
            let doc_id = Uuid::parse_str(row.get::<String, _>("document_id").as_str())?;
            doc_ids.push(doc_id);
        }

        Ok(doc_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;

    async fn create_test_storage() -> Result<Storage> {
        // Use an in-memory database for tests
        let connection_string = "sqlite::memory:";
        let options = SqliteConnectOptions::from_str(connection_string)?;

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                parent_id TEXT
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS document_access (
                document_id TEXT NOT NULL,
                accessed_at INTEGER NOT NULL,
                PRIMARY KEY (document_id, accessed_at)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Storage { pool })
    }

    #[tokio::test]
    async fn test_recently_accessed_documents_ordering() {
        let storage = create_test_storage().await.unwrap();
        
        // Create and save test documents
        let doc1 = Document::new("Document 1".to_string());
        let doc2 = Document::new("Document 2".to_string());
        let doc3 = Document::new("Document 3".to_string());
        
        let id1 = doc1.id;
        let id2 = doc2.id;
        let id3 = doc3.id;
        
        storage.save_document(&doc1).await.unwrap();
        storage.save_document(&doc2).await.unwrap();
        storage.save_document(&doc3).await.unwrap();
        
        // Access documents in order with delays
        storage.record_document_access(id1).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        storage.record_document_access(id2).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        storage.record_document_access(id3).await.unwrap();
        
        // Get recently accessed - should be in reverse order (most recent first)
        let recent = storage.get_recently_accessed_documents(10).await.unwrap();
        
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0], id3); // Most recent
        assert_eq!(recent[1], id2);
        assert_eq!(recent[2], id1); // Least recent
    }

    #[tokio::test]
    async fn test_recently_accessed_documents_limit() {
        let storage = create_test_storage().await.unwrap();
        
        // Create 5 documents
        let mut doc_ids = Vec::new();
        for i in 0..5 {
            let doc = Document::new(format!("Document {}", i));
            let id = doc.id;
            storage.save_document(&doc).await.unwrap();
            doc_ids.push(id);
        }
        
        // Access them in order with delays
        for id in &doc_ids {
            storage.record_document_access(*id).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        
        // Request only 3 most recent
        let recent = storage.get_recently_accessed_documents(3).await.unwrap();
        
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0], doc_ids[4]); // Most recent
        assert_eq!(recent[1], doc_ids[3]);
        assert_eq!(recent[2], doc_ids[2]);
    }
}
