use sqlx::PgPool;
use uuid::Uuid;

use placebo_shared::room::{RoomMemberResponse, RoomResponse};

use crate::error::AppError;
use crate::repositories::room_repo::{self, NewRoom, RoomMemberRow, RoomRow, UpdateRoom};

// ---------------------------------------------------------------------------
// Mapping
// ---------------------------------------------------------------------------

fn to_response(row: &RoomRow) -> RoomResponse {
    RoomResponse {
        id: row.id,
        name: row.name.clone(),
        camera_id: row.camera_id,
        owner_id: row.owner_id,
        is_private: row.is_private,
        max_members: row.max_members,
        created_at: row.created_at,
    }
}

fn to_member_response(row: &RoomMemberRow) -> RoomMemberResponse {
    RoomMemberResponse {
        user_id: row.user_id,
        display_name: row.display_name.clone(),
        avatar_url: row.avatar_url.clone(),
        joined_at: row.joined_at,
    }
}

// ---------------------------------------------------------------------------
// Service functions
// ---------------------------------------------------------------------------

pub async fn create_room(
    pool: &PgPool,
    owner_id: Uuid,
    name: String,
    camera_id: Option<Uuid>,
    is_private: bool,
    max_members: Option<i16>,
) -> Result<RoomResponse, AppError> {
    let room = NewRoom {
        name,
        camera_id,
        owner_id,
        is_private,
        max_members: max_members.unwrap_or(4).min(50),
    };
    let row = room_repo::create(pool, &room).await?;

    // Owner auto-joins as member
    room_repo::add_member(pool, row.id, owner_id).await?;

    Ok(to_response(&row))
}

pub async fn get_room(pool: &PgPool, id: Uuid) -> Result<RoomResponse, AppError> {
    let row = room_repo::get_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Room {id} not found")))?;
    Ok(to_response(&row))
}

pub async fn get_user_rooms(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<RoomResponse>, AppError> {
    let rows = room_repo::get_user_rooms(pool, user_id).await?;
    Ok(rows.iter().map(to_response).collect())
}

pub async fn update_room(
    pool: &PgPool,
    room_id: Uuid,
    caller_id: Uuid,
    update: UpdateRoom,
) -> Result<RoomResponse, AppError> {
    let room = room_repo::get_by_id(pool, room_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Room {room_id} not found")))?;

    if room.owner_id != caller_id {
        return Err(AppError::Forbidden("Only room owner can update".into()));
    }

    let row = room_repo::update(pool, room_id, &update)
        .await?
        .ok_or_else(|| AppError::NotFound("Room not found".into()))?;
    Ok(to_response(&row))
}

pub async fn delete_room(
    pool: &PgPool,
    room_id: Uuid,
    caller_id: Uuid,
) -> Result<(), AppError> {
    let room = room_repo::get_by_id(pool, room_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Room {room_id} not found")))?;

    if room.owner_id != caller_id {
        return Err(AppError::Forbidden("Only room owner can delete".into()));
    }

    room_repo::delete(pool, room_id).await?;
    Ok(())
}

pub async fn join_room(
    pool: &PgPool,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let room = room_repo::get_by_id(pool, room_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Room {room_id} not found")))?;

    // Check member limit
    let count = room_repo::member_count(pool, room_id).await?;
    if count >= room.max_members as i64 {
        return Err(AppError::Validation("Room is full".into()));
    }

    room_repo::add_member(pool, room_id, user_id).await?;
    Ok(())
}

pub async fn leave_room(
    pool: &PgPool,
    room_id: Uuid,
    user_id: Uuid,
    caller_id: Uuid,
) -> Result<(), AppError> {
    let room = room_repo::get_by_id(pool, room_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Room {room_id} not found")))?;

    // Owner can kick anyone; members can only remove themselves
    if caller_id != user_id && caller_id != room.owner_id {
        return Err(AppError::Forbidden(
            "Only room owner can remove other members".into(),
        ));
    }

    // Owner can't leave their own room (must delete it)
    if user_id == room.owner_id {
        return Err(AppError::Validation(
            "Owner cannot leave. Delete the room instead.".into(),
        ));
    }

    room_repo::remove_member(pool, room_id, user_id).await?;
    Ok(())
}

pub async fn get_members(
    pool: &PgPool,
    room_id: Uuid,
) -> Result<Vec<RoomMemberResponse>, AppError> {
    // Verify room exists
    room_repo::get_by_id(pool, room_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Room {room_id} not found")))?;

    let rows = room_repo::get_members(pool, room_id).await?;
    Ok(rows.iter().map(to_member_response).collect())
}
