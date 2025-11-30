use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub encrypted_identity_key: String,
    pub identity_public_key: String,
    pub prekey_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: i64,
    pub from_user: String,
    pub to_user: String,
    pub encrypted_content: String,
    pub ephemeral_public_key: String,
    #[sqlx(default)]
    pub sender_identity_key: Option<String>,
    #[sqlx(default)]
    pub sender_signature: Option<String>,
    pub message_number: i64,
    pub previous_chain_length: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Prekey {
    pub id: i64,
    pub user_id: i64,
    pub public_key: String,
    pub key_id: i64,
    pub used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RatchetState {
    pub user_id: i64,
    pub peer_username: String,
    pub root_key: String,
    pub chain_key_send: Option<String>,
    pub chain_key_receive: Option<String>,
    pub sending_chain_length: i64,
    pub receiving_chain_length: i64,
    pub previous_sending_chain_length: i64,
    pub public_key_send: Option<String>,
    pub public_key_receive: Option<String>,
}

// Request/Response types
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub encrypted_identity_key: String,
    pub identity_public_key: String,
    pub prekey_signature: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub encrypted_identity_key: String,
    pub identity_public_key: String,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub from_user: String,
    pub to_user: String,
    pub encrypted_content: String,
    pub ephemeral_public_key: String,
    pub sender_identity_key: Option<String>,
    pub sender_signature: Option<String>,
    pub message_number: i64,
    pub previous_chain_length: i64,
}

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub success: bool,
    pub message_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct PollParams {
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct GetMessagesParams {
    pub current_user: String,
}

#[derive(Debug, Serialize)]
pub struct UserKeysResponse {
    pub identity_public_key: String,
    pub prekey_signature: String,
}

#[derive(Debug, Deserialize)]
pub struct PrekeyData {
    pub public_key: String,
    pub key_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct UploadPrekeysRequest {
    pub username: String,
    pub prekeys: Vec<PrekeyData>,
}

#[derive(Debug, Serialize)]
pub struct PrekeyResponse {
    pub public_key: String,
    pub key_id: i64,
}

#[derive(Debug, Serialize)]
pub struct GenericResponse {
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct SaveRatchetStateRequest {
    pub username: String,
    pub peer_username: String,
    pub root_key: String,
    pub chain_key_send: Option<String>,
    pub chain_key_receive: Option<String>,
    pub sending_chain_length: i64,
    pub receiving_chain_length: i64,
    pub previous_sending_chain_length: i64,
    pub public_key_send: Option<String>,
    pub public_key_receive: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetRatchetParams {
    pub username: String,
}
