use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::repositories::rating_repo;

// ---------------------------------------------------------------------------
// Response types (inline, not shared – ratings are API-only)
// ---------------------------------------------------------------------------

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingStatsResponse {
    pub avg_score: Option<f64>,
    pub count: i64,
    pub user_score: Option<i16>,
}

// ---------------------------------------------------------------------------
// Service functions
// ---------------------------------------------------------------------------

pub async fn rate_camera(
    pool: &PgPool,
    camera_id: Uuid,
    user_id: Uuid,
    score: i16,
) -> Result<(), AppError> {
    if !(1..=5).contains(&score) {
        return Err(AppError::Validation(
            "Score must be between 1 and 5".into(),
        ));
    }

    rating_repo::upsert(pool, camera_id, user_id, score).await?;
    Ok(())
}

pub async fn get_camera_ratings(
    pool: &PgPool,
    camera_id: Uuid,
    user_id: Option<Uuid>,
) -> Result<RatingStatsResponse, AppError> {
    let stats = rating_repo::get_stats(pool, camera_id).await?;

    let user_score = if let Some(uid) = user_id {
        rating_repo::get_user_rating(pool, camera_id, uid).await?
    } else {
        None
    };

    Ok(RatingStatsResponse {
        avg_score: stats.avg_score,
        count: stats.count,
        user_score,
    })
}

pub async fn delete_rating(
    pool: &PgPool,
    camera_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let deleted = rating_repo::delete(pool, camera_id, user_id).await?;
    if !deleted {
        return Err(AppError::NotFound("Rating not found".into()));
    }
    Ok(())
}
