use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// UserRow – raw database row
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub locale: String,
    pub is_premium: bool,
    pub premium_until: Option<DateTime<Utc>>,
    pub cloud_used_bytes: i64,
    pub cloud_limit_bytes: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Update struct
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct UpdateUser {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub locale: Option<String>,
}

// ---------------------------------------------------------------------------
// Repository functions
// ---------------------------------------------------------------------------

pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
}

pub async fn update_profile(
    pool: &PgPool,
    id: Uuid,
    update: &UpdateUser,
) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"UPDATE users SET
            display_name = COALESCE($2, display_name),
            avatar_url = COALESCE($3, avatar_url),
            locale = COALESCE($4, locale),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *"#,
    )
    .bind(id)
    .bind(&update.display_name)
    .bind(&update.avatar_url)
    .bind(&update.locale)
    .fetch_optional(pool)
    .await
}
