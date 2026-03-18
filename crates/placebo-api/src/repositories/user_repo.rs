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
    sqlx::query_as::<_, UserRow>(
        "SELECT id, email, display_name, avatar_url, locale, is_premium, premium_until, cloud_used_bytes, cloud_limit_bytes, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        "SELECT id, email, display_name, avatar_url, locale, is_premium, premium_until, cloud_used_bytes, cloud_limit_bytes, created_at, updated_at FROM users WHERE email = $1",
    )
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
        RETURNING id, email, display_name, avatar_url, locale, is_premium, premium_until, cloud_used_bytes, cloud_limit_bytes, created_at, updated_at"#,
    )
    .bind(id)
    .bind(&update.display_name)
    .bind(&update.avatar_url)
    .bind(&update.locale)
    .fetch_optional(pool)
    .await
}

/// Create a new user with email/password. Returns the created UserRow.
pub async fn create_user(
    pool: &PgPool,
    email: &str,
    display_name: &str,
    password_hash: &str,
    locale: &str,
    email_verified: bool,
) -> Result<UserRow, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO users (email, display_name, password_hash, locale, email_verified)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, email, display_name, avatar_url, locale,
                  is_premium, premium_until, cloud_used_bytes,
                  cloud_limit_bytes, created_at, updated_at
        "#,
    )
    .bind(email.trim().to_lowercase())
    .bind(display_name.trim())
    .bind(password_hash)
    .bind(locale)
    .bind(email_verified)
    .fetch_one(pool)
    .await
}

/// Get password hash for login verification. Returns None if user not found or has no password.
pub async fn get_password_hash(
    pool: &PgPool,
    email: &str,
) -> Result<Option<(Uuid, String, bool)>, sqlx::Error> {
    // Returns (user_id, password_hash, email_verified)
    let row: Option<(Uuid, Option<String>, bool)> = sqlx::query_as(
        r#"
        SELECT id, password_hash, email_verified
        FROM users
        WHERE LOWER(email) = LOWER($1)
        "#,
    )
    .bind(email.trim())
    .fetch_optional(pool)
    .await?;

    match row {
        Some((id, Some(hash), verified)) => Ok(Some((id, hash, verified))),
        _ => Ok(None), // user not found or no password (OAuth-only)
    }
}

/// Set email as verified (after clicking verification link).
pub async fn set_email_verified(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        r#"
        UPDATE users
        SET email_verified = TRUE, email_verify_token = NULL, updated_at = NOW()
        WHERE email_verify_token = $1
        RETURNING id
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id,)| id))
}

/// Store an email verification token for a user.
pub async fn set_email_verify_token(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE users SET email_verify_token = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(token)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Store a password reset token with expiry (1 hour).
pub async fn set_password_reset_token(
    pool: &PgPool,
    email: &str,
    token: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE users
        SET password_reset_token = $1,
            password_reset_expires = NOW() + INTERVAL '1 hour',
            updated_at = NOW()
        WHERE LOWER(email) = LOWER($2)
        "#,
    )
    .bind(token)
    .bind(email.trim())
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Consume a password reset token and update the password.
/// Returns user_id if successful.
pub async fn reset_password(
    pool: &PgPool,
    token: &str,
    new_password_hash: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        r#"
        UPDATE users
        SET password_hash = $1,
            password_reset_token = NULL,
            password_reset_expires = NULL,
            updated_at = NOW()
        WHERE password_reset_token = $2
          AND password_reset_expires > NOW()
        RETURNING id
        "#,
    )
    .bind(new_password_hash)
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id,)| id))
}

/// Check if email is already taken.
pub async fn email_exists(pool: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM users WHERE LOWER(email) = LOWER($1))",
    )
    .bind(email.trim())
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}
