use sqlx::PgPool;
use uuid::Uuid;

use placebo_shared::user::BoostTokenInfo;

use crate::error::AppError;
use crate::repositories::boost_repo::{self, BoostWithCameraRow};

// ---------------------------------------------------------------------------
// Mapping
// ---------------------------------------------------------------------------

fn to_boost_info(row: &BoostWithCameraRow) -> BoostTokenInfo {
    BoostTokenInfo {
        camera_id: row.camera_id,
        camera_name: row.camera_name.clone(),
        days_added: row.days_added,
        applied_at: row.applied_at,
        expires_at: row.expires_at,
    }
}

// ---------------------------------------------------------------------------
// Service functions
// ---------------------------------------------------------------------------

pub async fn apply_boost(
    pool: &PgPool,
    user_id: Uuid,
    camera_id: Uuid,
    days: i16,
) -> Result<(), AppError> {
    if days < 1 || days > 365 {
        return Err(AppError::Validation(
            "Boost days must be between 1 and 365".into(),
        ));
    }

    boost_repo::apply(pool, user_id, camera_id, days).await?;
    Ok(())
}

pub async fn get_user_boosts(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<BoostTokenInfo>, AppError> {
    let rows = boost_repo::get_by_user(pool, user_id).await?;
    Ok(rows.iter().map(to_boost_info).collect())
}

pub async fn get_camera_boost_days(
    pool: &PgPool,
    camera_id: Uuid,
) -> Result<i64, AppError> {
    let days = boost_repo::total_active_days(pool, camera_id).await?;
    Ok(days)
}
