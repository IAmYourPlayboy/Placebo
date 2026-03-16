use sqlx::PgPool;
use uuid::Uuid;

use placebo_shared::user::{UserProfile, UserResponse};

use crate::error::AppError;
use crate::repositories::user_repo::{self, UpdateUser, UserRow};

// ---------------------------------------------------------------------------
// Mapping
// ---------------------------------------------------------------------------

fn to_public_response(row: &UserRow) -> UserResponse {
    UserResponse {
        id: row.id,
        display_name: row.display_name.clone(),
        avatar_url: row.avatar_url.clone(),
        is_premium: row.is_premium,
        created_at: row.created_at,
    }
}

fn to_profile(row: &UserRow) -> UserProfile {
    UserProfile {
        id: row.id,
        email: row.email.clone(),
        display_name: row.display_name.clone(),
        avatar_url: row.avatar_url.clone(),
        locale: row.locale.clone(),
        is_premium: row.is_premium,
        premium_until: row.premium_until,
        cloud_used_bytes: row.cloud_used_bytes,
        cloud_limit_bytes: row.cloud_limit_bytes,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

// ---------------------------------------------------------------------------
// Service functions
// ---------------------------------------------------------------------------

pub async fn get_public_profile(pool: &PgPool, id: Uuid) -> Result<UserResponse, AppError> {
    let row = user_repo::get_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {id} not found")))?;
    Ok(to_public_response(&row))
}

pub async fn get_my_profile(pool: &PgPool, id: Uuid) -> Result<UserProfile, AppError> {
    let row = user_repo::get_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;
    Ok(to_profile(&row))
}

pub async fn update_my_profile(
    pool: &PgPool,
    id: Uuid,
    update: &UpdateUser,
) -> Result<UserProfile, AppError> {
    let row = user_repo::update_profile(pool, id, update)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;
    Ok(to_profile(&row))
}
