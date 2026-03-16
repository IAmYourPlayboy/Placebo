use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use placebo_shared::recording::{ClipResponse, ClipStatus};

use crate::error::AppError;
use crate::repositories::clip_repo::{self, ClipRow};

// ---------------------------------------------------------------------------
// Mapping
// ---------------------------------------------------------------------------

fn to_response(row: &ClipRow) -> ClipResponse {
    let status = row
        .status
        .parse::<ClipStatus>()
        .unwrap_or(ClipStatus::Pending);

    ClipResponse {
        id: row.id,
        camera_id: row.camera_id,
        start_time: row.start_time,
        end_time: row.end_time,
        status,
        output_url: row.output_url.clone(),
        created_at: row.created_at,
        completed_at: row.completed_at,
    }
}

// ---------------------------------------------------------------------------
// Service functions
// ---------------------------------------------------------------------------

pub async fn request_clip(
    pool: &PgPool,
    camera_id: Uuid,
    user_id: Uuid,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<ClipResponse, AppError> {
    if end_time <= start_time {
        return Err(AppError::Validation(
            "end_time must be after start_time".into(),
        ));
    }

    let duration = end_time - start_time;
    if duration.num_minutes() > 30 {
        return Err(AppError::Validation(
            "Clip duration cannot exceed 30 minutes".into(),
        ));
    }

    let row = clip_repo::create(pool, camera_id, user_id, start_time, end_time).await?;
    Ok(to_response(&row))
}

pub async fn get_clip(pool: &PgPool, id: Uuid) -> Result<ClipResponse, AppError> {
    let row = clip_repo::get_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Clip {id} not found")))?;
    Ok(to_response(&row))
}

pub async fn get_user_clips(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ClipResponse>, AppError> {
    let rows = clip_repo::get_by_user(pool, user_id).await?;
    Ok(rows.iter().map(to_response).collect())
}
