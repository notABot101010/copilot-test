//! Database layer for TVflix media center

use sqlx::{sqlite::SqlitePool, FromRow};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("User not found")]
    UserNotFound,
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),
    #[error("Media not found: {0}")]
    MediaNotFound(i64),
    #[error("Playlist not found: {0}")]
    PlaylistNotFound(i64),
    #[error("Album not found: {0}")]
    AlbumNotFound(i64),
    #[error("Invalid session")]
    InvalidSession,
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub created_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Video,
    Music,
    Photo,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Video => write!(f, "video"),
            MediaType::Music => write!(f, "music"),
            MediaType::Photo => write!(f, "photo"),
        }
    }
}

impl std::str::FromStr for MediaType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "video" => Ok(MediaType::Video),
            "music" => Ok(MediaType::Music),
            "photo" => Ok(MediaType::Photo),
            _ => Err(format!("Unknown media type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Media {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub media_type: String,
    pub filename: String,
    pub storage_path: String,
    pub thumbnail_path: Option<String>,
    pub content_type: String,
    pub size: i64,
    pub duration: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Playlist {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct PlaylistItem {
    pub id: i64,
    pub playlist_id: i64,
    pub media_id: i64,
    pub position: i32,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Album {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct AlbumItem {
    pub id: i64,
    pub album_id: i64,
    pub media_id: i64,
    pub position: i32,
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
        // Users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                token TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                expires_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Media table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS media (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                media_type TEXT NOT NULL,
                filename TEXT NOT NULL,
                storage_path TEXT NOT NULL,
                thumbnail_path TEXT,
                content_type TEXT NOT NULL,
                size INTEGER NOT NULL,
                duration INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Playlists table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS playlists (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Playlist items table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS playlist_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                playlist_id INTEGER NOT NULL,
                media_id INTEGER NOT NULL,
                position INTEGER NOT NULL,
                FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
                FOREIGN KEY (media_id) REFERENCES media(id) ON DELETE CASCADE,
                UNIQUE(playlist_id, media_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Albums table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS albums (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Album items table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS album_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                album_id INTEGER NOT NULL,
                media_id INTEGER NOT NULL,
                position INTEGER NOT NULL,
                FOREIGN KEY (album_id) REFERENCES albums(id) ON DELETE CASCADE,
                FOREIGN KEY (media_id) REFERENCES media(id) ON DELETE CASCADE,
                UNIQUE(album_id, media_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indices
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_media_user ON media(user_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_media_type ON media(media_type)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // User operations
    pub async fn create_user(&self, username: &str, password_hash: &str) -> Result<User> {
        // Check if user already exists
        if self.get_user_by_username(username).await?.is_some() {
            return Err(DatabaseError::UserAlreadyExists(username.to_string()));
        }

        let result = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, password_hash)
            VALUES (?, ?)
            RETURNING id, username, password_hash, created_at
            "#,
        )
        .bind(username)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let result = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_user_by_id(&self, id: i64) -> Result<Option<User>> {
        let result = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, created_at FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    // Session operations
    pub async fn create_session(
        &self,
        user_id: i64,
        token: &str,
        expires_at: &str,
    ) -> Result<Session> {
        let result = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (user_id, token, expires_at)
            VALUES (?, ?, ?)
            RETURNING id, user_id, token, created_at, expires_at
            "#,
        )
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_session_by_token(&self, token: &str) -> Result<Option<Session>> {
        let result = sqlx::query_as::<_, Session>(
            "SELECT id, user_id, token, created_at, expires_at FROM sessions WHERE token = ? AND expires_at > datetime('now')",
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_session(&self, token: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE token = ?")
            .bind(token)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        let result = sqlx::query("DELETE FROM sessions WHERE expires_at <= datetime('now')")
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    // Media operations
    pub async fn create_media(
        &self,
        user_id: i64,
        title: &str,
        media_type: MediaType,
        filename: &str,
        storage_path: &str,
        thumbnail_path: Option<&str>,
        content_type: &str,
        size: i64,
        duration: Option<i64>,
    ) -> Result<Media> {
        let result = sqlx::query_as::<_, Media>(
            r#"
            INSERT INTO media (user_id, title, media_type, filename, storage_path, thumbnail_path, content_type, size, duration)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, user_id, title, media_type, filename, storage_path, thumbnail_path, content_type, size, duration, created_at
            "#,
        )
        .bind(user_id)
        .bind(title)
        .bind(media_type.to_string())
        .bind(filename)
        .bind(storage_path)
        .bind(thumbnail_path)
        .bind(content_type)
        .bind(size)
        .bind(duration)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn update_media_thumbnail(&self, id: i64, thumbnail_path: &str) -> Result<()> {
        sqlx::query("UPDATE media SET thumbnail_path = ? WHERE id = ?")
            .bind(thumbnail_path)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_media_by_id(&self, id: i64) -> Result<Option<Media>> {
        let result = sqlx::query_as::<_, Media>(
            "SELECT id, user_id, title, media_type, filename, storage_path, thumbnail_path, content_type, size, duration, created_at FROM media WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_media_by_user(
        &self,
        user_id: i64,
        media_type: Option<MediaType>,
    ) -> Result<Vec<Media>> {
        let result = if let Some(mt) = media_type {
            sqlx::query_as::<_, Media>(
                "SELECT id, user_id, title, media_type, filename, storage_path, thumbnail_path, content_type, size, duration, created_at FROM media WHERE user_id = ? AND media_type = ? ORDER BY created_at DESC",
            )
            .bind(user_id)
            .bind(mt.to_string())
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Media>(
                "SELECT id, user_id, title, media_type, filename, storage_path, thumbnail_path, content_type, size, duration, created_at FROM media WHERE user_id = ? ORDER BY created_at DESC",
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(result)
    }

    pub async fn delete_media(&self, id: i64) -> Result<Option<(String, Option<String>)>> {
        let media = self.get_media_by_id(id).await?;

        if let Some(m) = media {
            sqlx::query("DELETE FROM media WHERE id = ?")
                .bind(id)
                .execute(&self.pool)
                .await?;
            Ok(Some((m.storage_path, m.thumbnail_path)))
        } else {
            Ok(None)
        }
    }

    // Playlist operations
    pub async fn create_playlist(&self, user_id: i64, name: &str) -> Result<Playlist> {
        let result = sqlx::query_as::<_, Playlist>(
            r#"
            INSERT INTO playlists (user_id, name)
            VALUES (?, ?)
            RETURNING id, user_id, name, created_at
            "#,
        )
        .bind(user_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_playlist_by_id(&self, id: i64) -> Result<Option<Playlist>> {
        let result = sqlx::query_as::<_, Playlist>(
            "SELECT id, user_id, name, created_at FROM playlists WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_playlists_by_user(&self, user_id: i64) -> Result<Vec<Playlist>> {
        let result = sqlx::query_as::<_, Playlist>(
            "SELECT id, user_id, name, created_at FROM playlists WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_playlist(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM playlists WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn add_to_playlist(&self, playlist_id: i64, media_id: i64) -> Result<PlaylistItem> {
        // Get next position
        let max_pos: Option<i32> =
            sqlx::query_scalar("SELECT MAX(position) FROM playlist_items WHERE playlist_id = ?")
                .bind(playlist_id)
                .fetch_one(&self.pool)
                .await?;

        let position = max_pos.unwrap_or(0) + 1;

        let result = sqlx::query_as::<_, PlaylistItem>(
            r#"
            INSERT INTO playlist_items (playlist_id, media_id, position)
            VALUES (?, ?, ?)
            RETURNING id, playlist_id, media_id, position
            "#,
        )
        .bind(playlist_id)
        .bind(media_id)
        .bind(position)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn remove_from_playlist(&self, playlist_id: i64, media_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM playlist_items WHERE playlist_id = ? AND media_id = ?")
            .bind(playlist_id)
            .bind(media_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_playlist_media(&self, playlist_id: i64) -> Result<Vec<Media>> {
        let result = sqlx::query_as::<_, Media>(
            r#"
            SELECT m.id, m.user_id, m.title, m.media_type, m.filename, m.storage_path, m.thumbnail_path, m.content_type, m.size, m.duration, m.created_at
            FROM media m
            JOIN playlist_items pi ON m.id = pi.media_id
            WHERE pi.playlist_id = ?
            ORDER BY pi.position
            "#,
        )
        .bind(playlist_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    // Album operations
    pub async fn create_album(&self, user_id: i64, name: &str) -> Result<Album> {
        let result = sqlx::query_as::<_, Album>(
            r#"
            INSERT INTO albums (user_id, name)
            VALUES (?, ?)
            RETURNING id, user_id, name, created_at
            "#,
        )
        .bind(user_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_album_by_id(&self, id: i64) -> Result<Option<Album>> {
        let result = sqlx::query_as::<_, Album>(
            "SELECT id, user_id, name, created_at FROM albums WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_albums_by_user(&self, user_id: i64) -> Result<Vec<Album>> {
        let result = sqlx::query_as::<_, Album>(
            "SELECT id, user_id, name, created_at FROM albums WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_album(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM albums WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn add_to_album(&self, album_id: i64, media_id: i64) -> Result<AlbumItem> {
        // Get next position
        let max_pos: Option<i32> =
            sqlx::query_scalar("SELECT MAX(position) FROM album_items WHERE album_id = ?")
                .bind(album_id)
                .fetch_one(&self.pool)
                .await?;

        let position = max_pos.unwrap_or(0) + 1;

        let result = sqlx::query_as::<_, AlbumItem>(
            r#"
            INSERT INTO album_items (album_id, media_id, position)
            VALUES (?, ?, ?)
            RETURNING id, album_id, media_id, position
            "#,
        )
        .bind(album_id)
        .bind(media_id)
        .bind(position)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn remove_from_album(&self, album_id: i64, media_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM album_items WHERE album_id = ? AND media_id = ?")
            .bind(album_id)
            .bind(media_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_album_media(&self, album_id: i64) -> Result<Vec<Media>> {
        let result = sqlx::query_as::<_, Media>(
            r#"
            SELECT m.id, m.user_id, m.title, m.media_type, m.filename, m.storage_path, m.thumbnail_path, m.content_type, m.size, m.duration, m.created_at
            FROM media m
            JOIN album_items ai ON m.id = ai.media_id
            WHERE ai.album_id = ?
            ORDER BY ai.position
            "#,
        )
        .bind(album_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }
}
