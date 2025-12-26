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
