use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Row types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ClipRow {
    pub id: Uuid,
    pub camera_id: Uuid,
    pub user_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: String, // clip_status enum as text
    pub output_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Repository functions
// ---------------------------------------------------------------------------

/// Create a new clip request.
pub async fn create(
    pool: &PgPool,
    camera_id: Uuid,
    user_id: Uuid,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<ClipRow, sqlx::Error> {
    sqlx::query_as::<_, ClipRow>(
        r#"INSERT INTO clip_requests (camera_id, user_id, start_time, end_time)
        VALUES ($1, $2, $3, $4)
        RETURNING id, camera_id, user_id, start_time, end_time, status::TEXT, output_url, created_at, completed_at"#,
    )
    .bind(camera_id)
    .bind(user_id)
    .bind(start_time)
    .bind(end_time)
    .fetch_one(pool)
    .await
}

/// Get a clip request by ID.
pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ClipRow>, sqlx::Error> {
    sqlx::query_as::<_, ClipRow>(
        r#"SELECT id, camera_id, user_id, start_time, end_time, status::TEXT, output_url, created_at, completed_at
        FROM clip_requests WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Get all clip requests for a user.
pub async fn get_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<ClipRow>, sqlx::Error> {
    sqlx::query_as::<_, ClipRow>(
        r#"SELECT id, camera_id, user_id, start_time, end_time, status::TEXT, output_url, created_at, completed_at
        FROM clip_requests WHERE user_id = $1 ORDER BY created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Update clip status (and optionally output_url).
pub async fn update_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
    output_url: Option<&str>,
) -> Result<Option<ClipRow>, sqlx::Error> {
    let completed_at: Option<DateTime<Utc>> = if status == "ready" || status == "failed" {
        Some(Utc::now())
    } else {
        None
    };

    sqlx::query_as::<_, ClipRow>(
        r#"UPDATE clip_requests SET
            status = $2::clip_status,
            output_url = COALESCE($3, output_url),
            completed_at = COALESCE($4, completed_at)
        WHERE id = $1
        RETURNING id, camera_id, user_id, start_time, end_time, status::TEXT, output_url, created_at, completed_at"#,
    )
    .bind(id)
    .bind(status)
    .bind(output_url)
    .bind(completed_at)
    .fetch_optional(pool)
    .await
}
