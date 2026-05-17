use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::redis::session::{self, SessionData};
use crate::repositories::user_repo::{self, CreateUserArgs};
use placebo_shared::auth::{AuthResponse, LoginRequest, RegisterRequest};

/// Session TTL: 30 days in seconds.
const SESSION_TTL: u64 = 30 * 24 * 60 * 60;

/// Generate a cryptographically secure random token (64 hex chars = 256 bits).
fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);
    hex::encode(bytes)
}

/// Hash a password with Argon2id.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // Argon2id with safe defaults
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Password hashing failed: {e}")))?;
    Ok(hash.to_string())
}

/// Verify a password against a stored hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid password hash format: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// Register a new user.
pub async fn register(
    pool: &PgPool,
    redis: &deadpool_redis::Pool,
    req: &RegisterRequest,
    auto_verify_email: bool,
) -> Result<AuthResponse, AppError> {
    // 1. Validate input
    let errors = req.validate();
    if !errors.is_empty() {
        return Err(AppError::Validation(errors.join("; ")));
    }

    // 2. Check if email already taken
    let exists = user_repo::email_exists(pool, &req.email)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    if exists {
        return Err(AppError::Conflict("Email already registered".into()));
    }

    // 3. Resolve username – either honour the requested one (if free) or generate from
    //    display_name. On conflict we surface 3 alternatives so the client can render chips.
    let username = match req.username.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(requested) => {
            let free = user_repo::username_available(pool, requested)
                .await
                .map_err(|e| AppError::Internal(e.into()))?;
            if !free {
                let mut suggestions = Vec::with_capacity(3);
                for _ in 0..3 {
                    suggestions.push(
                        user_repo::generate_unique_username(pool, requested)
                            .await
                            .map_err(|e| AppError::Internal(e.into()))?,
                    );
                }
                return Err(AppError::UsernameTaken { suggestions });
            }
            requested.to_string()
        }
        None => user_repo::generate_unique_username(pool, &req.display_name)
            .await
            .map_err(|e| AppError::Internal(e.into()))?,
    };

    // 4. Hash password
    let password_hash = hash_password(&req.password)?;

    // 5. Create user in database
    let locale = req.locale.as_deref().unwrap_or("en");
    let user = user_repo::create_user(
        pool,
        CreateUserArgs {
            email: &req.email,
            display_name: &req.display_name,
            username: &username,
            password_hash: &password_hash,
            locale,
            date_of_birth: req.date_of_birth,
            // Privacy-first default: hide DOB unless the user explicitly opts in.
            date_of_birth_hidden: req.date_of_birth_hidden.unwrap_or(true),
            email_verified: auto_verify_email,
        },
    )
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.constraint() == Some("users_email_key") => {
            AppError::Conflict("Email already registered".into())
        }
        sqlx::Error::Database(db_err)
            if db_err.constraint() == Some("idx_users_username_normalized") =>
        {
            // Race: username became taken between our check and the insert. Best-effort
            // suggestions for the client.
            AppError::UsernameTaken {
                suggestions: Vec::new(),
            }
        }
        _ => AppError::Internal(e.into()),
    })?;

    // 6. If not auto-verified, generate verification token and store it
    if !auto_verify_email {
        let verify_token = generate_token();
        user_repo::set_email_verify_token(pool, user.id, &verify_token)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        // TODO: Send verification email via lettre
        tracing::info!(user_id = %user.id, "Email verification token generated (email sending not yet implemented)");
    }

    // 7. Create session
    let token = generate_token();
    let session = SessionData {
        user_id: user.id,
        email: user.email.clone(),
        is_premium: user.is_premium,
        created_at: chrono::Utc::now().timestamp(),
    };
    session::create(redis, &token, &session, SESSION_TTL)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis session create failed: {e}")))?;
    session::track_user_session(redis, user.id, &token, SESSION_TTL)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis session track failed: {e}")))?;

    Ok(AuthResponse {
        token,
        user_id: user.id,
        email: user.email,
        username: username.clone(),
        display_name: user.display_name,
        is_premium: user.is_premium,
        expires_in_seconds: SESSION_TTL,
    })
}

/// Login with email/password.
pub async fn login(
    pool: &PgPool,
    redis: &deadpool_redis::Pool,
    req: &LoginRequest,
) -> Result<AuthResponse, AppError> {
    // 1. Validate input
    let errors = req.validate();
    if !errors.is_empty() {
        return Err(AppError::Validation(errors.join("; ")));
    }

    // 2. Look up user and password hash
    // Generic error message – don't reveal whether email exists
    let generic_err = || AppError::Unauthorized("Invalid email or password".into());

    let (user_id, hash, _email_verified) =
        user_repo::get_password_hash(pool, &req.email)
            .await
            .map_err(|e| AppError::Internal(e.into()))?
            .ok_or_else(generic_err)?;

    // 3. Verify password
    if !verify_password(&req.password, &hash)? {
        return Err(generic_err());
    }

    // 4. Load full user for response
    let user = user_repo::get_by_id(pool, user_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("User found in auth but not in DB")))?;

    // 5. Create session
    let token = generate_token();
    let session = SessionData {
        user_id: user.id,
        email: user.email.clone(),
        is_premium: user.is_premium,
        created_at: chrono::Utc::now().timestamp(),
    };
    session::create(redis, &token, &session, SESSION_TTL)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis session create failed: {e}")))?;
    session::track_user_session(redis, user.id, &token, SESSION_TTL)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis session track failed: {e}")))?;

    Ok(AuthResponse {
        token,
        user_id: user.id,
        email: user.email,
        // Migration 008 backfills every existing row, so an Option here always resolves;
        // the empty string fallback is a defensive measure that should never fire in practice.
        username: user.username.unwrap_or_default(),
        display_name: user.display_name,
        is_premium: user.is_premium,
        expires_in_seconds: SESSION_TTL,
    })
}

/// Logout – delete current session.
pub async fn logout(
    redis: &deadpool_redis::Pool,
    token: &str,
) -> Result<(), AppError> {
    session::delete(redis, token)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis session delete failed: {e}")))?;
    Ok(())
}

/// Logout everywhere – delete ALL sessions for a user.
pub async fn logout_all(
    redis: &deadpool_redis::Pool,
    user_id: Uuid,
) -> Result<u64, AppError> {
    session::delete_all_for_user(redis, user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis delete all sessions failed: {e}")))
}

/// Refresh session – extend TTL (sliding window).
pub async fn refresh_session(
    redis: &deadpool_redis::Pool,
    token: &str,
) -> Result<u64, AppError> {
    // Check session exists
    let _session = session::get(redis, token)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis session get failed: {e}")))?
        .ok_or_else(|| AppError::Unauthorized("Invalid or expired session".into()))?;

    // Extend TTL
    session::refresh(redis, token, SESSION_TTL)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis session refresh failed: {e}")))?;

    Ok(SESSION_TTL)
}

/// Request password reset – generates token, stores in DB.
/// Always returns success (don't reveal if email exists).
pub async fn request_password_reset(
    pool: &PgPool,
    email: &str,
) -> Result<(), AppError> {
    let token = generate_token();
    let found = user_repo::set_password_reset_token(pool, email, &token)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if found {
        // TODO: Send reset email via lettre
        tracing::info!("Password reset token generated for {email} (email sending not yet implemented)");
    }
    // Always return OK – don't reveal whether email exists
    Ok(())
}

/// Confirm password reset – validates token, updates password.
pub async fn confirm_password_reset(
    pool: &PgPool,
    redis: &deadpool_redis::Pool,
    token: &str,
    new_password: &str,
) -> Result<(), AppError> {
    if new_password.len() < 8 {
        return Err(AppError::Validation("Password must be at least 8 characters".into()));
    }

    let new_hash = hash_password(new_password)?;
    let user_id = user_repo::reset_password(pool, token, &new_hash)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::Validation("Invalid or expired reset token".into()))?;

    // Invalidate ALL existing sessions (security: password changed)
    let _ = session::delete_all_for_user(redis, user_id).await;

    Ok(())
}
