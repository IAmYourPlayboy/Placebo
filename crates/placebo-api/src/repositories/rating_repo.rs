use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Row types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RatingRow {
    pub id: Uuid,
    pub camera_id: Uuid,
    pub user_id: Uuid,
    pub score: i16,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RatingStats {
    pub avg_score: Option<f64>,
    pub count: i64,
}

// ---------------------------------------------------------------------------
// Repository functions
// ---------------------------------------------------------------------------

/// Insert or update a rating (unique per camera+user).
pub async fn upsert(
    pool: &PgPool,
    camera_id: Uuid,
    user_id: Uuid,
    score: i16,
) -> Result<RatingRow, sqlx::Error> {
    sqlx::query_as::<_, RatingRow>(
        r#"INSERT INTO ratings (camera_id, user_id, score)
        VALUES ($1, $2, $3)
        ON CONFLICT (camera_id, user_id) DO UPDATE SET score = $3, created_at = NOW()
        RETURNING *"#,
    )
    .bind(camera_id)
    .bind(user_id)
    .bind(score)
    .fetch_one(pool)
    .await
}

/// Get aggregate rating stats for a camera.
pub async fn get_stats(pool: &PgPool, camera_id: Uuid) -> Result<RatingStats, sqlx::Error> {
    sqlx::query_as::<_, RatingStats>(
        "SELECT AVG(score::float8) as avg_score, COUNT(*) as count FROM ratings WHERE camera_id = $1",
    )
    .bind(camera_id)
    .fetch_one(pool)
    .await
}

/// Get a specific user's rating for a camera.
pub async fn get_user_rating(
    pool: &PgPool,
    camera_id: Uuid,
    user_id: Uuid,
) -> Result<Option<i16>, sqlx::Error> {
    let row: Option<(i16,)> = sqlx::query_as(
        "SELECT score FROM ratings WHERE camera_id = $1 AND user_id = $2",
    )
    .bind(camera_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

/// Delete a user's rating for a camera.
pub async fn delete(
    pool: &PgPool,
    camera_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM ratings WHERE camera_id = $1 AND user_id = $2",
    )
    .bind(camera_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
