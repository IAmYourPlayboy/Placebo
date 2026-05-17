use chrono::{DateTime, NaiveDate, Utc};
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// UserRow – raw database row
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    /// Backfilled NOT NULL at the application layer; legacy DB column is nullable, hence Option.
    /// Migration 008 backfills all existing rows so this is effectively never None in practice.
    pub username: Option<String>,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub locale: String,
    pub is_premium: bool,
    pub premium_until: Option<DateTime<Utc>>,
    pub cloud_used_bytes: i64,
    pub cloud_limit_bytes: i64,
    pub date_of_birth: Option<NaiveDate>,
    pub date_of_birth_hidden: bool,
    pub email_verified: bool,
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

/// Arguments for `create_user`. A struct rather than a positional signature so the
/// users table can grow new columns (M2 added username/dob; later milestones will add more)
/// without an explosion of overloads or unwieldy parameter lists.
#[derive(Debug)]
pub struct CreateUserArgs<'a> {
    pub email: &'a str,
    pub display_name: &'a str,
    pub username: &'a str,
    pub password_hash: &'a str,
    pub locale: &'a str,
    pub date_of_birth: Option<NaiveDate>,
    pub date_of_birth_hidden: bool,
    pub email_verified: bool,
}

// ---------------------------------------------------------------------------
// SELECT helper – the canonical column list for UserRow.
// Keep in one place so we don't drift between get_by_id, get_by_email, RETURNING, etc.
// ---------------------------------------------------------------------------

const USER_COLS: &str = "id, email, username, display_name, avatar_url, locale, \
     is_premium, premium_until, cloud_used_bytes, cloud_limit_bytes, \
     date_of_birth, date_of_birth_hidden, email_verified, created_at, updated_at";

// ---------------------------------------------------------------------------
// Repository functions
// ---------------------------------------------------------------------------

pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
    let sql = format!("SELECT {USER_COLS} FROM users WHERE id = $1");
    sqlx::query_as::<_, UserRow>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRow>, sqlx::Error> {
    let sql = format!("SELECT {USER_COLS} FROM users WHERE email = $1");
    sqlx::query_as::<_, UserRow>(&sql)
        .bind(email)
        .fetch_optional(pool)
        .await
}

pub async fn update_profile(
    pool: &PgPool,
    id: Uuid,
    update: &UpdateUser,
) -> Result<Option<UserRow>, sqlx::Error> {
    let sql = format!(
        r#"UPDATE users SET
            display_name = COALESCE($2, display_name),
            avatar_url = COALESCE($3, avatar_url),
            locale = COALESCE($4, locale),
            updated_at = NOW()
        WHERE id = $1
        RETURNING {USER_COLS}"#
    );
    sqlx::query_as::<_, UserRow>(&sql)
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
    args: CreateUserArgs<'_>,
) -> Result<UserRow, sqlx::Error> {
    let sql = format!(
        r#"
        INSERT INTO users
            (email, display_name, username, password_hash, locale,
             date_of_birth, date_of_birth_hidden, email_verified)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING {USER_COLS}
        "#
    );
    sqlx::query_as::<_, UserRow>(&sql)
        .bind(args.email.trim().to_lowercase())
        .bind(args.display_name.trim())
        .bind(args.username)
        .bind(args.password_hash)
        .bind(args.locale)
        .bind(args.date_of_birth)
        .bind(args.date_of_birth_hidden)
        .bind(args.email_verified)
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

/// Check if a username is available (case-insensitive).
pub async fn username_available(pool: &PgPool, username: &str) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        "SELECT NOT EXISTS(SELECT 1 FROM users WHERE username_normalized = lower($1))",
    )
    .bind(username)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Generate a candidate username derived from `hint` (typically display_name or email),
/// suffixed with a random 4-digit number. Tries up to 10 times before falling back to a
/// timestamp-based suffix that is functionally guaranteed unique.
pub async fn generate_unique_username(pool: &PgPool, hint: &str) -> Result<String, sqlx::Error> {
    // Reduce hint to ASCII alnum + underscore, lowercased, max 12 chars.
    let base: String = hint
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .take(12)
        .collect();
    let base = if base.len() < 3 {
        "user".to_string()
    } else {
        // Strip leading/trailing underscores so we don't violate the validation rule when the
        // hint accidentally starts/ends with one.
        base.trim_matches('_').to_string()
    };
    let base = if base.len() < 3 { "user".to_string() } else { base };

    for _ in 0..10 {
        let suffix: u32 = rand::thread_rng().gen_range(0..10_000);
        let candidate = format!("{base}_{suffix:04}");
        if username_available(pool, &candidate).await? {
            return Ok(candidate);
        }
    }
    // Last-resort: millisecond timestamp – effectively collision-free.
    let ts = chrono::Utc::now().timestamp_millis();
    Ok(format!("{base}_{ts}"))
}
