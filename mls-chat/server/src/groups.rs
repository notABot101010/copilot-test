use axum::{
    extract::{Path, Query, State},
    Json,
};
use tracing::info;
use uuid::Uuid;

use crate::models::*;
use crate::{AppError, AppState};

pub async fn create_group(
    State(state): State<AppState>,
    Json(req): Json<CreateGroupRequest>,
) -> Result<Json<CreateGroupResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let group_id = Uuid::new_v4().to_string();
    let is_channel = if req.is_channel { 1 } else { 0 };

    // Create the group
    sqlx::query("INSERT INTO groups (group_id, name, is_channel, created_by) VALUES (?, ?, ?, ?)")
        .bind(&group_id)
        .bind(&req.name)
        .bind(is_channel)
        .bind(user.id)
        .execute(&state.db)
        .await?;

    // Add creator as admin member
    sqlx::query("INSERT INTO group_members (group_id, user_id, is_admin) VALUES (?, ?, 1)")
        .bind(&group_id)
        .bind(user.id)
        .execute(&state.db)
        .await?;

    info!("Created group {} by user {}", group_id, req.username);

    Ok(Json(CreateGroupResponse {
        success: true,
        group_id,
    }))
}

pub async fn list_groups(
    State(state): State<AppState>,
    Query(params): Query<UsernameQuery>,
) -> Result<Json<Vec<GroupInfo>>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&params.username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let groups = sqlx::query_as::<_, GroupInfoRow>(
        r#"SELECT g.group_id, g.name, g.is_channel, gm.is_admin,
                  (SELECT COUNT(*) FROM group_members WHERE group_id = g.group_id) as member_count
           FROM groups g
           JOIN group_members gm ON g.group_id = gm.group_id AND gm.user_id = ?
           ORDER BY g.name"#,
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;

    let result: Vec<GroupInfo> = groups
        .into_iter()
        .map(|g| GroupInfo {
            group_id: g.group_id,
            name: g.name,
            is_channel: g.is_channel == 1,
            is_admin: g.is_admin == 1,
            member_count: g.member_count,
        })
        .collect();

    Ok(Json(result))
}

pub async fn get_group(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    Query(params): Query<UsernameQuery>,
) -> Result<Json<GroupInfo>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&params.username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let group = sqlx::query_as::<_, GroupInfoRow>(
        r#"SELECT g.group_id, g.name, g.is_channel, gm.is_admin,
                  (SELECT COUNT(*) FROM group_members WHERE group_id = g.group_id) as member_count
           FROM groups g
           JOIN group_members gm ON g.group_id = gm.group_id AND gm.user_id = ?
           WHERE g.group_id = ?"#,
    )
    .bind(user.id)
    .bind(&group_id)
    .fetch_optional(&state.db)
    .await?;

    let group = group.ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

    Ok(Json(GroupInfo {
        group_id: group.group_id,
        name: group.name,
        is_channel: group.is_channel == 1,
        is_admin: group.is_admin == 1,
        member_count: group.member_count,
    }))
}

pub async fn invite_member(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    Json(req): Json<InviteMemberRequest>,
) -> Result<Json<GenericResponse>, AppError> {
    // Get inviter
    let inviter = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await?;

    let inviter = inviter.ok_or_else(|| AppError::NotFound("Inviter not found".to_string()))?;

    // Check inviter is admin of the group
    let membership = sqlx::query_as::<_, GroupMemberRow>(
        "SELECT id, group_id, user_id, is_admin, joined_at FROM group_members WHERE group_id = ? AND user_id = ?",
    )
    .bind(&group_id)
    .bind(inviter.id)
    .fetch_optional(&state.db)
    .await?;

    let membership = membership
        .ok_or_else(|| AppError::Unauthorized("Not a member of this group".to_string()))?;

    if membership.is_admin == 0 {
        return Err(AppError::Unauthorized(
            "Only admins can invite members".to_string(),
        ));
    }

    // Get invitee
    let invitee = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&req.invite_username)
    .fetch_optional(&state.db)
    .await?;

    let invitee =
        invitee.ok_or_else(|| AppError::NotFound("User to invite not found".to_string()))?;

    // Check if already a member
    let existing = sqlx::query_as::<_, GroupMemberRow>(
        "SELECT id, group_id, user_id, is_admin, joined_at FROM group_members WHERE group_id = ? AND user_id = ?",
    )
    .bind(&group_id)
    .bind(invitee.id)
    .fetch_optional(&state.db)
    .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("User is already a member".to_string()));
    }

    // Decode welcome data
    let welcome_data = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &req.welcome_data,
    )
    .map_err(|_| AppError::BadRequest("Invalid welcome data".to_string()))?;

    // Decode commit data
    let commit_data =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &req.commit_data)
            .map_err(|_| AppError::BadRequest("Invalid commit data".to_string()))?;

    // Store pending welcome
    sqlx::query("INSERT INTO pending_welcomes (user_id, group_id, welcome_data, inviter_id) VALUES (?, ?, ?, ?)")
        .bind(invitee.id)
        .bind(&group_id)
        .bind(&welcome_data)
        .bind(inviter.id)
        .execute(&state.db)
        .await?;

    // Store commit message for all other group members
    let result = sqlx::query(
        "INSERT INTO mls_messages (group_id, sender_id, message_type, message_data) VALUES (?, ?, 'commit', ?)",
    )
    .bind(&group_id)
    .bind(inviter.id)
    .bind(&commit_data)
    .execute(&state.db)
    .await?;

    let message_id = result.last_insert_rowid();

    // Create pending messages for all current members except inviter
    let members = sqlx::query_as::<_, GroupMemberRow>(
        "SELECT id, group_id, user_id, is_admin, joined_at FROM group_members WHERE group_id = ? AND user_id != ?",
    )
    .bind(&group_id)
    .bind(inviter.id)
    .fetch_all(&state.db)
    .await?;

    for member in &members {
        sqlx::query(
            "INSERT INTO pending_messages (user_id, group_id, message_id) VALUES (?, ?, ?)",
        )
        .bind(member.user_id)
        .bind(&group_id)
        .bind(message_id)
        .execute(&state.db)
        .await?;
    }

    // Notify users
    let mut poll_manager = state.poll_manager.write().await;
    poll_manager.notify(&req.invite_username);
    for member in &members {
        // Get username for notification
        if let Ok(Some(u)) = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, created_at FROM users WHERE id = ?",
        )
        .bind(member.user_id)
        .fetch_optional(&state.db)
        .await
        {
            poll_manager.notify(&u.username);
        }
    }

    info!(
        "User {} invited {} to group {}",
        req.username, req.invite_username, group_id
    );

    Ok(Json(GenericResponse { success: true }))
}

pub async fn join_group(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    Json(req): Json<JoinGroupRequest>,
) -> Result<Json<GenericResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Check if there's a pending welcome
    let welcome = sqlx::query_as::<_, PendingWelcomeRow>(
        "SELECT pw.id, pw.user_id, pw.group_id, pw.welcome_data, pw.group_info_data, pw.inviter_id, pw.created_at, g.name as group_name, u.username as inviter_name
         FROM pending_welcomes pw
         JOIN groups g ON g.group_id = pw.group_id
         JOIN users u ON u.id = pw.inviter_id
         WHERE pw.user_id = ? AND pw.group_id = ?",
    )
    .bind(user.id)
    .bind(&group_id)
    .fetch_optional(&state.db)
    .await?;

    if welcome.is_none() {
        return Err(AppError::NotFound(
            "No pending invitation found".to_string(),
        ));
    }

    // Add user as member
    sqlx::query(
        "INSERT OR IGNORE INTO group_members (group_id, user_id, is_admin) VALUES (?, ?, 0)",
    )
    .bind(&group_id)
    .bind(user.id)
    .execute(&state.db)
    .await?;

    // Delete the welcome
    sqlx::query("DELETE FROM pending_welcomes WHERE user_id = ? AND group_id = ?")
        .bind(user.id)
        .bind(&group_id)
        .execute(&state.db)
        .await?;

    info!("User {} joined group {}", req.username, group_id);

    Ok(Json(GenericResponse { success: true }))
}

pub async fn send_message(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<GenericResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Check membership
    let membership = sqlx::query_as::<_, GroupMemberRow>(
        "SELECT id, group_id, user_id, is_admin, joined_at FROM group_members WHERE group_id = ? AND user_id = ?",
    )
    .bind(&group_id)
    .bind(user.id)
    .fetch_optional(&state.db)
    .await?;

    let membership = membership
        .ok_or_else(|| AppError::Unauthorized("Not a member of this group".to_string()))?;

    // Check if it's a channel and user is not admin
    let group = sqlx::query_as::<_, GroupRow>(
        "SELECT id, group_id, name, is_channel, created_by, created_at FROM groups WHERE group_id = ?",
    )
    .bind(&group_id)
    .fetch_optional(&state.db)
    .await?;

    let group = group.ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

    if group.is_channel == 1 && membership.is_admin == 0 {
        return Err(AppError::Unauthorized(
            "Only admins can post in channels".to_string(),
        ));
    }

    // Decode message data
    let message_data = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &req.message_data,
    )
    .map_err(|_| AppError::BadRequest("Invalid message data".to_string()))?;

    // Store message
    let result = sqlx::query(
        "INSERT INTO mls_messages (group_id, sender_id, message_type, message_data) VALUES (?, ?, ?, ?)",
    )
    .bind(&group_id)
    .bind(user.id)
    .bind(&req.message_type)
    .bind(&message_data)
    .execute(&state.db)
    .await?;

    let message_id = result.last_insert_rowid();

    // Create pending messages for all members except sender
    let members = sqlx::query_as::<_, GroupMemberRow>(
        "SELECT id, group_id, user_id, is_admin, joined_at FROM group_members WHERE group_id = ? AND user_id != ?",
    )
    .bind(&group_id)
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;

    for member in &members {
        sqlx::query(
            "INSERT INTO pending_messages (user_id, group_id, message_id) VALUES (?, ?, ?)",
        )
        .bind(member.user_id)
        .bind(&group_id)
        .bind(message_id)
        .execute(&state.db)
        .await?;
    }

    // Notify users
    let mut poll_manager = state.poll_manager.write().await;
    for member in &members {
        if let Ok(Some(u)) = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, created_at FROM users WHERE id = ?",
        )
        .bind(member.user_id)
        .fetch_optional(&state.db)
        .await
        {
            poll_manager.notify(&u.username);
        }
    }

    Ok(Json(GenericResponse { success: true }))
}

pub async fn get_messages(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    Query(params): Query<GetMessagesQuery>,
) -> Result<Json<Vec<GroupMessage>>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&params.username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Check membership
    let membership = sqlx::query_as::<_, GroupMemberRow>(
        "SELECT id, group_id, user_id, is_admin, joined_at FROM group_members WHERE group_id = ? AND user_id = ?",
    )
    .bind(&group_id)
    .bind(user.id)
    .fetch_optional(&state.db)
    .await?;

    if membership.is_none() {
        return Err(AppError::Unauthorized(
            "Not a member of this group".to_string(),
        ));
    }

    let since_id = params.since_id.unwrap_or(0);

    let messages = sqlx::query_as::<_, GroupMessageRow>(
        r#"SELECT m.id, m.message_type, m.message_data, u.username as sender_name, m.created_at
           FROM mls_messages m
           LEFT JOIN users u ON u.id = m.sender_id
           WHERE m.group_id = ? AND m.id > ?
           ORDER BY m.created_at ASC"#,
    )
    .bind(&group_id)
    .bind(since_id)
    .fetch_all(&state.db)
    .await?;

    let result: Vec<GroupMessage> = messages
        .into_iter()
        .map(|m| GroupMessage {
            id: m.id,
            message_type: m.message_type,
            message_data: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &m.message_data,
            ),
            sender_name: m.sender_name,
            created_at: m.created_at,
        })
        .collect();

    Ok(Json(result))
}

pub async fn list_channels(
    State(state): State<AppState>,
) -> Result<Json<Vec<GroupInfo>>, AppError> {
    let channels = sqlx::query_as::<_, GroupInfoRow>(
        r#"SELECT g.group_id, g.name, g.is_channel, 0 as is_admin,
                  (SELECT COUNT(*) FROM group_members WHERE group_id = g.group_id) as member_count
           FROM groups g
           WHERE g.is_channel = 1
           ORDER BY g.name"#,
    )
    .fetch_all(&state.db)
    .await?;

    let result: Vec<GroupInfo> = channels
        .into_iter()
        .map(|g| GroupInfo {
            group_id: g.group_id,
            name: g.name,
            is_channel: true,
            is_admin: false,
            member_count: g.member_count,
        })
        .collect();

    Ok(Json(result))
}

pub async fn subscribe_channel(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    Json(req): Json<JoinGroupRequest>,
) -> Result<Json<GenericResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Check if it's actually a channel
    let group = sqlx::query_as::<_, GroupRow>(
        "SELECT id, group_id, name, is_channel, created_by, created_at FROM groups WHERE group_id = ?",
    )
    .bind(&group_id)
    .fetch_optional(&state.db)
    .await?;

    let group = group.ok_or_else(|| AppError::NotFound("Channel not found".to_string()))?;

    if group.is_channel == 0 {
        return Err(AppError::BadRequest("This is not a channel".to_string()));
    }

    // Add user as subscriber (non-admin member)
    sqlx::query(
        "INSERT OR IGNORE INTO group_members (group_id, user_id, is_admin) VALUES (?, ?, 0)",
    )
    .bind(&group_id)
    .bind(user.id)
    .execute(&state.db)
    .await?;

    info!("User {} subscribed to channel {}", req.username, group_id);

    Ok(Json(GenericResponse { success: true }))
}
