use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Database models

#[derive(Debug, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub created_at: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
}

#[derive(Debug, FromRow)]
pub struct KeyPackageRow {
    pub id: i64,
    pub user_id: i64,
    pub key_package_data: Vec<u8>,
    pub key_package_hash: Vec<u8>,
    pub used: i64,
}

#[derive(Debug, FromRow)]
pub struct GroupRow {
    pub id: i64,
    pub group_id: String,
    pub name: String,
    pub is_channel: i64,
    pub created_by: i64,
    pub created_at: String,
}

#[derive(Debug, FromRow)]
pub struct GroupMemberRow {
    pub id: i64,
    pub group_id: String,
    pub user_id: i64,
    pub is_admin: i64,
    pub joined_at: String,
}

#[derive(Debug, FromRow)]
pub struct MlsMessageRow {
    pub id: i64,
    pub group_id: String,
    pub sender_id: Option<i64>,
    pub message_type: String,
    pub message_data: Vec<u8>,
    pub created_at: String,
}

#[derive(Debug, FromRow)]
pub struct PendingWelcomeRow {
    pub id: i64,
    pub user_id: i64,
    pub group_id: String,
    pub welcome_data: Vec<u8>,
    pub group_info_data: Option<Vec<u8>>,
    pub inviter_id: i64,
    pub created_at: String,
    pub group_name: String,
    pub inviter_name: String,
}

#[derive(Debug, FromRow)]
pub struct PendingMessageRow {
    pub id: i64,
    pub user_id: i64,
    pub group_id: String,
    pub message_id: i64,
    pub delivered: i64,
    pub created_at: String,
    pub message_type: String,
    pub message_data: Vec<u8>,
    pub sender_id: Option<i64>,
    pub sender_name: Option<String>,
}

// API request/response models

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub user_id: Option<i64>,
    pub username: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GenericResponse {
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct UploadKeyPackagesRequest {
    pub username: String,
    pub key_packages: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct KeyPackageResponse {
    pub key_package: String,
}

#[derive(Debug, Deserialize)]
pub struct UsernameQuery {
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct PollParams {
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct PendingWelcome {
    pub id: i64,
    pub group_id: String,
    pub group_name: String,
    pub welcome_data: String,
    pub group_info_data: Option<String>,
    pub inviter_name: String,
}

#[derive(Debug, Serialize)]
pub struct PendingMessage {
    pub id: i64,
    pub group_id: String,
    pub message_type: String,
    pub message_data: String,
    pub sender_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PollResponse {
    pub welcomes: Vec<PendingWelcome>,
    pub messages: Vec<PendingMessage>,
}

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub exclude: String,
}

// Group-related models

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub username: String,
    pub name: String,
    pub is_channel: bool,
}

#[derive(Debug, Serialize)]
pub struct CreateGroupResponse {
    pub success: bool,
    pub group_id: String,
}

#[derive(Debug, Deserialize)]
pub struct InviteMemberRequest {
    pub username: String,
    pub invite_username: String,
    pub welcome_data: String,
    pub commit_data: String,
}

#[derive(Debug, Deserialize)]
pub struct JoinGroupRequest {
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub username: String,
    pub message_data: String,
    pub message_type: String,
}

#[derive(Debug, Serialize)]
pub struct GroupInfo {
    pub group_id: String,
    pub name: String,
    pub is_channel: bool,
    pub is_admin: bool,
    pub member_count: i64,
}

#[derive(Debug, Serialize)]
pub struct GroupMessage {
    pub id: i64,
    pub message_type: String,
    pub message_data: String,
    pub sender_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct GetMessagesQuery {
    pub username: String,
    pub since_id: Option<i64>,
}

#[derive(Debug, FromRow)]
pub struct GroupInfoRow {
    pub group_id: String,
    pub name: String,
    pub is_channel: i64,
    pub is_admin: i64,
    pub member_count: i64,
}

#[derive(Debug, FromRow)]
pub struct GroupMessageRow {
    pub id: i64,
    pub message_type: String,
    pub message_data: Vec<u8>,
    pub sender_name: Option<String>,
    pub created_at: String,
}
