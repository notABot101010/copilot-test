//! Integration tests for TVflix server

use reqwest::Client;
use serde_json::json;
use std::net::TcpListener;
use tempfile::TempDir;

async fn spawn_server() -> (String, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_path = temp_dir.path().join("data");
    let db_path = temp_dir.path().join("tvflix_test.db");

    std::fs::create_dir_all(&data_path).expect("Failed to create data dir");

    // Find an available port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port");
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    // Spawn server in background
    let data_path_clone = data_path.clone();
    let db_path_clone = db_path.clone();
    tokio::spawn(async move {
        use tvflix_server::*;
        
        let db_url = format!("sqlite:{}?mode=rwc", db_path_clone.display());
        let db = database::Database::connect(&db_url).await.expect("DB connect failed");
        db.init().await.expect("DB init failed");

        let storage = storage::Storage::new(data_path_clone);

        let state = AppState {
            db: std::sync::Arc::new(db),
            storage: std::sync::Arc::new(storage),
        };

        let app = handlers::create_router(state);
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind");
        axum::serve(listener, app).await.expect("Server failed");
    });

    // Wait for server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    (format!("http://127.0.0.1:{}", port), temp_dir)
}

#[tokio::test]
async fn test_register_and_login() {
    let (base_url, _temp_dir) = spawn_server().await;
    let client = Client::new();

    // Register
    let response = client
        .post(format!("{}/api/auth/register", base_url))
        .json(&json!({
            "username": "testuser",
            "password": "testpass123"
        }))
        .send()
        .await
        .expect("Failed to send register request");

    assert!(response.status().is_success(), "Register failed: {:?}", response.status());

    let auth_response: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(auth_response.get("token").is_some());
    assert_eq!(auth_response["user"]["username"], "testuser");

    // Login
    let response = client
        .post(format!("{}/api/auth/login", base_url))
        .json(&json!({
            "username": "testuser",
            "password": "testpass123"
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert!(response.status().is_success(), "Login failed");

    let auth_response: serde_json::Value = response.json().await.expect("Failed to parse response");
    let token = auth_response["token"].as_str().unwrap();

    // Get current user
    let response = client
        .get(format!("{}/api/auth/me", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to get user");

    assert!(response.status().is_success());
    let user: serde_json::Value = response.json().await.expect("Failed to parse user");
    assert_eq!(user["username"], "testuser");
}

#[tokio::test]
async fn test_upload_and_list_media() {
    let (base_url, _temp_dir) = spawn_server().await;
    let client = Client::new();

    // Register and get token
    let response = client
        .post(format!("{}/api/auth/register", base_url))
        .json(&json!({
            "username": "mediauser",
            "password": "testpass123"
        }))
        .send()
        .await
        .expect("Failed to register");

    let auth_response: serde_json::Value = response.json().await.unwrap();
    let token = auth_response["token"].as_str().unwrap();

    // Upload a file
    let file_content = b"fake video content for testing";
    let form = reqwest::multipart::Form::new()
        .text("title", "Test Video")
        .part(
            "file",
            reqwest::multipart::Part::bytes(file_content.to_vec())
                .file_name("test.mp4")
                .mime_str("video/mp4")
                .unwrap(),
        );

    let response = client
        .post(format!("{}/api/media", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload");

    assert!(response.status().is_success(), "Upload failed: {:?}", response.status());

    let media: serde_json::Value = response.json().await.unwrap();
    assert_eq!(media["title"], "Test Video");
    assert_eq!(media["media_type"], "video");
    let media_id = media["id"].as_i64().unwrap();

    // List media
    let response = client
        .get(format!("{}/api/media", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to list media");

    assert!(response.status().is_success());
    let media_list: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(media_list.len(), 1);
    assert_eq!(media_list[0]["title"], "Test Video");

    // Stream media
    let response = client
        .get(format!("{}/api/media/{}/stream", base_url, media_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to stream");

    assert!(response.status().is_success());
    let body = response.bytes().await.unwrap();
    assert_eq!(body.as_ref(), file_content);

    // Delete media
    let response = client
        .delete(format!("{}/api/media/{}", base_url, media_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to delete");

    assert!(response.status().is_success());

    // Verify deleted
    let response = client
        .get(format!("{}/api/media", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to list media");

    let media_list: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(media_list.is_empty());
}

#[tokio::test]
async fn test_playlists() {
    let (base_url, _temp_dir) = spawn_server().await;
    let client = Client::new();

    // Register and get token
    let response = client
        .post(format!("{}/api/auth/register", base_url))
        .json(&json!({
            "username": "playlistuser",
            "password": "testpass123"
        }))
        .send()
        .await
        .expect("Failed to register");

    let auth_response: serde_json::Value = response.json().await.unwrap();
    let token = auth_response["token"].as_str().unwrap();

    // Create playlist
    let response = client
        .post(format!("{}/api/playlists", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({ "name": "My Playlist" }))
        .send()
        .await
        .expect("Failed to create playlist");

    assert!(response.status().is_success());
    let playlist: serde_json::Value = response.json().await.unwrap();
    assert_eq!(playlist["name"], "My Playlist");
    let playlist_id = playlist["id"].as_i64().unwrap();

    // List playlists
    let response = client
        .get(format!("{}/api/playlists", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to list playlists");

    assert!(response.status().is_success());
    let playlists: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(playlists.len(), 1);

    // Delete playlist
    let response = client
        .delete(format!("{}/api/playlists/{}", base_url, playlist_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to delete playlist");

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_albums() {
    let (base_url, _temp_dir) = spawn_server().await;
    let client = Client::new();

    // Register and get token
    let response = client
        .post(format!("{}/api/auth/register", base_url))
        .json(&json!({
            "username": "albumuser",
            "password": "testpass123"
        }))
        .send()
        .await
        .expect("Failed to register");

    let auth_response: serde_json::Value = response.json().await.unwrap();
    let token = auth_response["token"].as_str().unwrap();

    // Create album
    let response = client
        .post(format!("{}/api/albums", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({ "name": "My Album" }))
        .send()
        .await
        .expect("Failed to create album");

    assert!(response.status().is_success());
    let album: serde_json::Value = response.json().await.unwrap();
    assert_eq!(album["name"], "My Album");
    let album_id = album["id"].as_i64().unwrap();

    // List albums
    let response = client
        .get(format!("{}/api/albums", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to list albums");

    assert!(response.status().is_success());
    let albums: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(albums.len(), 1);

    // Delete album
    let response = client
        .delete(format!("{}/api/albums/{}", base_url, album_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to delete album");

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_unauthorized_access() {
    let (base_url, _temp_dir) = spawn_server().await;
    let client = Client::new();

    // Try to access protected endpoint without auth
    let response = client
        .get(format!("{}/api/media", base_url))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_range_request() {
    let (base_url, _temp_dir) = spawn_server().await;
    let client = Client::new();

    // Register and get token
    let response = client
        .post(format!("{}/api/auth/register", base_url))
        .json(&json!({
            "username": "rangeuser",
            "password": "testpass123"
        }))
        .send()
        .await
        .expect("Failed to register");

    let auth_response: serde_json::Value = response.json().await.unwrap();
    let token = auth_response["token"].as_str().unwrap();

    // Upload a file
    let file_content = b"0123456789ABCDEFGHIJ";
    let form = reqwest::multipart::Form::new()
        .text("title", "Range Test")
        .part(
            "file",
            reqwest::multipart::Part::bytes(file_content.to_vec())
                .file_name("test.mp4")
                .mime_str("video/mp4")
                .unwrap(),
        );

    let response = client
        .post(format!("{}/api/media", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload");

    let media: serde_json::Value = response.json().await.unwrap();
    let media_id = media["id"].as_i64().unwrap();

    // Request partial content
    let response = client
        .get(format!("{}/api/media/{}/stream", base_url, media_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Range", "bytes=0-4")
        .send()
        .await
        .expect("Failed to stream with range");

    assert_eq!(response.status(), 206);
    let body = response.bytes().await.unwrap();
    assert_eq!(body.as_ref(), b"01234");
}
