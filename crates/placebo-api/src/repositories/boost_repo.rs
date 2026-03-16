use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Row types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BoostRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub camera_id: Uuid,
    pub days_added: i16,
    pub expires_at: DateTime<Utc>,
    pub applied_at: DateTime<Utc>,
}

/// Boost row with camera name for user's boost history.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BoostWithCameraRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub camera_id: Uuid,
    pub camera_name: String,
    pub days_added: i16,
    pub expires_at: DateTime<Utc>,
    pub applied_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Repository functions
// ---------------------------------------------------------------------------

/// Apply a boost token to a camera.
pub async fn apply(
    pool: &PgPool,
    user_id: Uuid,
    camera_id: Uuid,
    days: i16,
) -> Result<BoostRow, sqlx::Error> {
    let expires_at = Utc::now() + Duration::days(days as i64);
    sqlx::query_as::<_, BoostRow>(
        r#"INSERT INTO boost_tokens (user_id, camera_id, days_added, expires_at)
        VALUES ($1, $2, $3, $4)
        RETURNING *"#,
    )
    .bind(user_id)
    .bind(camera_id)
    .bind(days)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

/// Get active (unexpired) boosts for a camera.
pub async fn get_active_by_camera(
    pool: &PgPool,
    camera_id: Uuid,
) -> Result<Vec<BoostRow>, sqlx::Error> {
    sqlx::query_as::<_, BoostRow>(
        "SELECT * FROM boost_tokens WHERE camera_id = $1 AND expires_at > NOW() ORDER BY applied_at DESC",
    )
    .bind(camera_id)
    .fetch_all(pool)
    .await
}

/// Get all boosts by a user (with camera name), ordered by most recent.
pub async fn get_by_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<BoostWithCameraRow>, sqlx::Error> {
    sqlx::query_as::<_, BoostWithCameraRow>(
        r#"SELECT bt.id, bt.user_id, bt.camera_id, c.name as camera_name,
            bt.days_added, bt.expires_at, bt.applied_at
        FROM boost_tokens bt
        JOIN cameras c ON c.id = bt.camera_id
        WHERE bt.user_id = $1
        ORDER BY bt.applied_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Total active boost days for a camera (sum of unexpired).
pub async fn total_active_days(
    pool: &PgPool,
    camera_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let row: (Option<i64>,) = sqlx::query_as(
        "SELECT SUM(days_added::bigint) FROM boost_tokens WHERE camera_id = $1 AND expires_at > NOW()",
    )
    .bind(camera_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0.unwrap_or(0))
}
