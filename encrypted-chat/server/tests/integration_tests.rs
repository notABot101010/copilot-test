/**
 * Integration tests for the encrypted chat server
 * 
 * Run these tests with the server running:
 * 1. Start the server: cargo run -p encrypted-chat-server
 * 2. Run tests: cargo test -p encrypted-chat-server --test integration_tests
 */

use reqwest::Client;
use serde_json::json;
use std::time::Duration;

const BASE_URL: &str = "http://localhost:4001";

async fn wait_for_server() {
    let client = Client::new();
    for _ in 0..30 {
        if client.get(format!("{}/api/users", BASE_URL))
            .timeout(Duration::from_secs(1))
            .send()
            .await
            .is_ok() 
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("Server did not start in time");
}

#[tokio::test]
async fn test_user_registration() {
    wait_for_server().await;
    
    let client = Client::new();
    
    // Register a user
    let response = client.post(format!("{}/api/register", BASE_URL))
        .json(&json!({
            "username": "test_user_1",
            "identity_public_key": "dGVzdF9wdWJsaWNfa2V5",
            "salt": "dGVzdF9zYWx0",
            "encrypted_identity_private_key": "dGVzdF9wcml2YXRlX2tleQ==",
            "identity_key_iv": "dGVzdF9pdg=="
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success() || response.status() == 409);
}

#[tokio::test]
async fn test_user_login() {
    wait_for_server().await;
    
    let client = Client::new();
    
    // First register
    let _ = client.post(format!("{}/api/register", BASE_URL))
        .json(&json!({
            "username": "test_user_login",
            "identity_public_key": "dGVzdF9wdWJsaWNfa2V5",
            "salt": "dGVzdF9zYWx0",
            "encrypted_identity_private_key": "dGVzdF9wcml2YXRlX2tleQ==",
            "identity_key_iv": "dGVzdF9pdg=="
        }))
        .send()
        .await;
    
    // Then login
    let response = client.post(format!("{}/api/login", BASE_URL))
        .json(&json!({
            "username": "test_user_login"
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["username"], "test_user_login");
    assert!(body["identity_public_key"].is_string());
    assert!(body["salt"].is_string());
}

#[tokio::test]
async fn test_prekey_bundle_upload() {
    wait_for_server().await;
    
    let client = Client::new();
    let username = "test_user_prekey";
    
    // First register
    let _ = client.post(format!("{}/api/register", BASE_URL))
        .json(&json!({
            "username": username,
            "identity_public_key": "dGVzdF9wdWJsaWNfa2V5",
            "salt": "dGVzdF9zYWx0",
            "encrypted_identity_private_key": "dGVzdF9wcml2YXRlX2tleQ==",
            "identity_key_iv": "dGVzdF9pdg=="
        }))
        .send()
        .await;
    
    // Upload prekey bundle
    let response = client.post(format!("{}/api/users/{}/prekeys", BASE_URL, username))
        .json(&json!({
            "signed_prekey_public": "c2lnbmVkX3ByZWtleQ==",
            "signed_prekey_signature": "c2lnbmF0dXJl",
            "encrypted_signed_prekey_private": "ZW5jcnlwdGVkX3ByaXZhdGU=",
            "signed_prekey_iv": "cHJla2V5X2l2",
            "one_time_prekeys": ["b3RwazE=", "b3RwazI="],
            "encrypted_one_time_prekey_privates": ["ZW5jX290cGsx", "ZW5jX290cGsy"],
            "one_time_prekey_ivs": ["aXYx", "aXYy"]
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_prekey_bundle_fetch() {
    wait_for_server().await;
    
    let client = Client::new();
    let username = "test_user_prekey_fetch";
    
    // First register
    let _ = client.post(format!("{}/api/register", BASE_URL))
        .json(&json!({
            "username": username,
            "identity_public_key": "dGVzdF9wdWJsaWNfa2V5",
            "salt": "dGVzdF9zYWx0",
            "encrypted_identity_private_key": "dGVzdF9wcml2YXRlX2tleQ==",
            "identity_key_iv": "dGVzdF9pdg=="
        }))
        .send()
        .await;
    
    // Upload prekey bundle
    let _ = client.post(format!("{}/api/users/{}/prekeys", BASE_URL, username))
        .json(&json!({
            "signed_prekey_public": "c2lnbmVkX3ByZWtleQ==",
            "signed_prekey_signature": "c2lnbmF0dXJl",
            "encrypted_signed_prekey_private": "ZW5jcnlwdGVkX3ByaXZhdGU=",
            "signed_prekey_iv": "cHJla2V5X2l2",
            "one_time_prekeys": ["b3RwazE=", "b3RwazI="],
            "encrypted_one_time_prekey_privates": ["ZW5jX290cGsx", "ZW5jX290cGsy"],
            "one_time_prekey_ivs": ["aXYx", "aXYy"]
        }))
        .send()
        .await;
    
    // Fetch prekey bundle
    let response = client.get(format!("{}/api/users/{}/prekeys", BASE_URL, username))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(body["identity_public_key"].is_string());
    assert!(body["signed_prekey_public"].is_string());
    assert!(body["signed_prekey_signature"].is_string());
}

#[tokio::test]
async fn test_send_and_receive_message() {
    wait_for_server().await;
    
    let client = Client::new();
    // Use unique IDs to avoid conflicts with other tests
    let unique_id = uuid::Uuid::new_v4().to_string();
    let sender = format!("test_sender_{}", &unique_id[..8]);
    let recipient = format!("test_recipient_{}", &unique_id[..8]);
    
    // Register both users
    for username in [&sender, &recipient] {
        let _ = client.post(format!("{}/api/register", BASE_URL))
            .json(&json!({
                "username": username,
                "identity_public_key": "dGVzdF9wdWJsaWNfa2V5",
                "salt": "dGVzdF9zYWx0",
                "encrypted_identity_private_key": "dGVzdF9wcml2YXRlX2tleQ==",
                "identity_key_iv": "dGVzdF9pdg=="
            }))
            .send()
            .await;
    }
    
    // Send message
    let response = client.post(format!("{}/api/users/{}/messages", BASE_URL, sender))
        .json(&json!({
            "recipient_username": recipient,
            "sealed_sender_envelope": "{\"test\": \"envelope\"}"
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    let message_id = body["id"].as_str().expect("Missing message id");
    
    // Small delay to let the message be stored
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Poll for message
    let response = client.get(format!("{}/api/users/{}/messages/poll?timeout_secs=1", BASE_URL, recipient))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    let messages = body["messages"].as_array().expect("Missing messages array");
    assert!(!messages.is_empty(), "Should have at least one message");
    
    // Check that our message is in the list
    let has_our_message = messages.iter().any(|m| m["id"].as_str() == Some(message_id));
    assert!(has_our_message, "Should receive our message");
}

#[tokio::test]
async fn test_message_acknowledgment() {
    wait_for_server().await;
    
    let client = Client::new();
    // Use unique IDs to avoid conflicts with other tests
    let unique_id = uuid::Uuid::new_v4().to_string();
    let sender = format!("test_sender_ack_{}", &unique_id[..8]);
    let recipient = format!("test_recipient_ack_{}", &unique_id[..8]);
    
    // Register both users
    for username in [&sender, &recipient] {
        let response = client.post(format!("{}/api/register", BASE_URL))
            .json(&json!({
                "username": username,
                "identity_public_key": "dGVzdF9wdWJsaWNfa2V5",
                "salt": "dGVzdF9zYWx0",
                "encrypted_identity_private_key": "dGVzdF9wcml2YXRlX2tleQ==",
                "identity_key_iv": "dGVzdF9pdg=="
            }))
            .send()
            .await
            .expect("Failed to register user");
        assert!(response.status().is_success() || response.status() == 409);
    }
    
    // Send message
    let response = client.post(format!("{}/api/users/{}/messages", BASE_URL, sender))
        .json(&json!({
            "recipient_username": recipient,
            "sealed_sender_envelope": "{\"test\": \"envelope\"}"
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    let message_id = body["id"].as_str().expect("Missing message id").to_string();
    
    // Small delay to let the message be stored
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Poll and get message
    let poll_response = client.get(format!("{}/api/users/{}/messages/poll?timeout_secs=1", BASE_URL, recipient))
        .send()
        .await
        .expect("Failed to poll");
    
    assert!(poll_response.status().is_success());
    let poll_body: serde_json::Value = poll_response.json().await.expect("Failed to parse poll response");
    let messages = poll_body["messages"].as_array().expect("Missing messages array");
    
    // Verify our message was received (may have other messages too)
    let has_our_message = messages.iter().any(|m| m["id"].as_str() == Some(&message_id));
    assert!(has_our_message, "Should receive our message");
    
    // Acknowledge message
    let response = client.post(format!("{}/api/users/{}/messages/ack", BASE_URL, recipient))
        .json(&json!({
            "message_ids": [&message_id]
        }))
        .send()
        .await
        .expect("Failed to send ack request");
    
    assert!(response.status().is_success(), "Acknowledgment should succeed");
}

#[tokio::test]
async fn test_list_users() {
    wait_for_server().await;
    
    let client = Client::new();
    
    // Register a user
    let _ = client.post(format!("{}/api/register", BASE_URL))
        .json(&json!({
            "username": "test_list_user",
            "identity_public_key": "dGVzdF9wdWJsaWNfa2V5",
            "salt": "dGVzdF9zYWx0",
            "encrypted_identity_private_key": "dGVzdF9wcml2YXRlX2tleQ==",
            "identity_key_iv": "dGVzdF9pdg=="
        }))
        .send()
        .await;
    
    // List users
    let response = client.get(format!("{}/api/users", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body: Vec<String> = response.json().await.expect("Failed to parse response");
    assert!(body.contains(&"test_list_user".to_string()));
}

#[tokio::test]
async fn test_duplicate_user_registration() {
    wait_for_server().await;
    
    let client = Client::new();
    let username = "test_duplicate_user";
    
    // First registration
    let _ = client.post(format!("{}/api/register", BASE_URL))
        .json(&json!({
            "username": username,
            "identity_public_key": "dGVzdF9wdWJsaWNfa2V5",
            "salt": "dGVzdF9zYWx0",
            "encrypted_identity_private_key": "dGVzdF9wcml2YXRlX2tleQ==",
            "identity_key_iv": "dGVzdF9pdg=="
        }))
        .send()
        .await;
    
    // Second registration with same username
    let response = client.post(format!("{}/api/register", BASE_URL))
        .json(&json!({
            "username": username,
            "identity_public_key": "ZGlmZmVyZW50X2tleQ==",
            "salt": "ZGlmZmVyZW50X3NhbHQ=",
            "encrypted_identity_private_key": "ZGlmZmVyZW50X3ByaXZhdGU=",
            "identity_key_iv": "ZGlmZmVyZW50X2l2"
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    // Should return conflict
    assert_eq!(response.status(), 409);
}

#[tokio::test]
async fn test_bidirectional_messaging() {
    wait_for_server().await;
    
    let client = Client::new();
    // Use unique IDs to avoid conflicts with other tests
    let unique_id = uuid::Uuid::new_v4().to_string();
    let user1 = format!("user1_{}", &unique_id[..8]);
    let user2 = format!("user2_{}", &unique_id[..8]);
    
    // Register both users
    for username in [&user1, &user2] {
        let response = client.post(format!("{}/api/register", BASE_URL))
            .json(&json!({
                "username": username,
                "identity_public_key": "dGVzdF9wdWJsaWNfa2V5",
                "salt": "dGVzdF9zYWx0",
                "encrypted_identity_private_key": "dGVzdF9wcml2YXRlX2tleQ==",
                "identity_key_iv": "dGVzdF9pdg=="
            }))
            .send()
            .await
            .expect("Failed to register user");
        assert!(response.status().is_success() || response.status() == 409);
    }
    
    // User1 sends message to User2
    let response1 = client.post(format!("{}/api/users/{}/messages", BASE_URL, user1))
        .json(&json!({
            "recipient_username": user2,
            "sealed_sender_envelope": "{\"type\": \"message\", \"content\": \"Hello User2!\"}"
        }))
        .send()
        .await
        .expect("Failed to send message");
    
    assert!(response1.status().is_success());
    let body1: serde_json::Value = response1.json().await.expect("Failed to parse response");
    let msg1_id = body1["id"].as_str().expect("Missing message id").to_string();
    
    // Small delay to let the message be stored
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // User2 polls and receives message from User1
    let poll_response1 = client.get(format!("{}/api/users/{}/messages/poll?timeout_secs=1", BASE_URL, user2))
        .send()
        .await
        .expect("Failed to poll");
    
    assert!(poll_response1.status().is_success());
    let poll_body1: serde_json::Value = poll_response1.json().await.expect("Failed to parse poll response");
    let messages1 = poll_body1["messages"].as_array().expect("Missing messages array");
    
    // Verify User2 received the message from User1
    let has_msg1 = messages1.iter().any(|m| m["id"].as_str() == Some(&msg1_id));
    assert!(has_msg1, "User2 should receive message from User1");
    
    // User2 sends reply to User1
    let response2 = client.post(format!("{}/api/users/{}/messages", BASE_URL, user2))
        .json(&json!({
            "recipient_username": user1,
            "sealed_sender_envelope": "{\"type\": \"message\", \"content\": \"Hello User1! Got your message.\"}"
        }))
        .send()
        .await
        .expect("Failed to send reply");
    
    assert!(response2.status().is_success());
    let body2: serde_json::Value = response2.json().await.expect("Failed to parse response");
    let msg2_id = body2["id"].as_str().expect("Missing message id").to_string();
    
    // Small delay
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // User1 polls and receives reply from User2
    let poll_response2 = client.get(format!("{}/api/users/{}/messages/poll?timeout_secs=1", BASE_URL, user1))
        .send()
        .await
        .expect("Failed to poll");
    
    assert!(poll_response2.status().is_success());
    let poll_body2: serde_json::Value = poll_response2.json().await.expect("Failed to parse poll response");
    let messages2 = poll_body2["messages"].as_array().expect("Missing messages array");
    
    // Verify User1 received the reply from User2
    let has_msg2 = messages2.iter().any(|m| m["id"].as_str() == Some(&msg2_id));
    assert!(has_msg2, "User1 should receive reply from User2");
    
    // User1 sends another message to User2
    let response3 = client.post(format!("{}/api/users/{}/messages", BASE_URL, user1))
        .json(&json!({
            "recipient_username": user2,
            "sealed_sender_envelope": "{\"type\": \"message\", \"content\": \"Thanks for replying!\"}"
        }))
        .send()
        .await
        .expect("Failed to send second message");
    
    assert!(response3.status().is_success());
    let body3: serde_json::Value = response3.json().await.expect("Failed to parse response");
    let msg3_id = body3["id"].as_str().expect("Missing message id").to_string();
    
    // Small delay
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // User2 polls and receives second message from User1
    let poll_response3 = client.get(format!("{}/api/users/{}/messages/poll?timeout_secs=1", BASE_URL, user2))
        .send()
        .await
        .expect("Failed to poll");
    
    assert!(poll_response3.status().is_success());
    let poll_body3: serde_json::Value = poll_response3.json().await.expect("Failed to parse poll response");
    let messages3 = poll_body3["messages"].as_array().expect("Missing messages array");
    
    // Verify User2 received the second message from User1
    let has_msg3 = messages3.iter().any(|m| m["id"].as_str() == Some(&msg3_id));
    assert!(has_msg3, "User2 should receive second message from User1");
}
