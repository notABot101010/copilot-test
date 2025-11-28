use std::path::Path;

use sqlx::{Row, SqlitePool};

/// Database manager for git repositories
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Connect to the SQLite database
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Database { pool })
    }

    /// Initialize the database schema
    pub async fn init(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS repositories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                path TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Create a new repository entry
    pub async fn create_repository(&self, name: &str, path: &str) -> Result<i64, sqlx::Error> {
        let result = sqlx::query("INSERT INTO repositories (name, path) VALUES (?, ?)")
            .bind(name)
            .bind(path)
            .execute(&self.pool)
            .await?;
        Ok(result.last_insert_rowid())
    }

    /// Get repository by name
    pub async fn get_repository(&self, name: &str) -> Result<Option<Repository>, sqlx::Error> {
        let row = sqlx::query("SELECT id, name, path FROM repositories WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Repository {
            id: r.get("id"),
            name: r.get("name"),
            path: r.get("path"),
        }))
    }

    /// List all repositories
    #[allow(dead_code)]
    pub async fn list_repositories(&self) -> Result<Vec<Repository>, sqlx::Error> {
        let rows = sqlx::query("SELECT id, name, path FROM repositories")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| Repository {
                id: r.get("id"),
                name: r.get("name"),
                path: r.get("path"),
            })
            .collect())
    }
}

/// Repository model
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Repository {
    pub id: i64,
    pub name: String,
    pub path: String,
}

/// Initialize a bare git repository on disk
pub async fn init_bare_repo(path: &Path) -> Result<(), std::io::Error> {
    let status = tokio::process::Command::new("git")
        .args(["init", "--bare"])
        .arg(path)
        .status()
        .await?;

    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to initialize bare git repository",
        ));
    }

    Ok(())
}
