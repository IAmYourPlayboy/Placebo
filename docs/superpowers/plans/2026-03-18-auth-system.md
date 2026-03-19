# Auth System Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Full authentication system – register, login, logout, session refresh, password hashing, secure token generation, email verification scaffolding, and demo mode for unauthenticated users.

**Architecture:** Argon2id password hashing → random 256-bit session tokens in Redis (30-day TTL, sliding window refresh) → `AuthUser` extractor already wired. No JWT – Redis sessions are simpler, instantly revocable, and already implemented. OAuth providers deferred to Phase 2. Email verification scaffolded but optional at registration (auto-verified in dev).

**Tech Stack:** axum 0.7, argon2 (password hashing), rand (token generation), lettre (email – scaffolded), sqlx 0.8 (PostgreSQL), deadpool-redis, placebo-shared types.

---

## File Structure

### New Files
| File | Responsibility |
|------|---------------|
| `migrations/007_auth_password.sql` | Add `password_hash`, `email_verified`, `email_verify_token`, `password_reset_token`, `password_reset_expires` to users |
| `src/handlers/auth.rs` | Register, login, logout, refresh, verify-email endpoints |
| `src/services/auth_service.rs` | Password hashing, token generation, credential validation, session lifecycle |
| `crates/placebo-shared/src/auth.rs` | Request/response types for auth endpoints |

### Modified Files
| File | Changes |
|------|---------|
| `Cargo.toml` | Add `argon2`, `rand` dependencies |
| `crates/placebo-shared/Cargo.toml` | No changes needed (types use existing serde) |
| `crates/placebo-shared/src/lib.rs` | Re-export `auth` module |
| `src/repositories/user_repo.rs` | Add `create_user`, `get_password_hash`, `set_email_verified`, `set_password_reset_token`, `update_password`, `get_by_reset_token` |
| `src/handlers/mod.rs` | Register auth routes in `api_router()` |
| `src/redis/session.rs` | Add `delete_all_for_user` (logout everywhere) |
| `src/main.rs` | No changes – dev seeding already works |

---

## Chunk 1: Database & Types

### Task 1: Migration – Add auth fields to users table

**Files:**
- Create: `crates/placebo-api/migrations/007_auth_password.sql`

- [ ] **Step 1: Write the migration**

```sql
-- 007_auth_password.sql
-- Add authentication fields to users table

ALTER TABLE users
    ADD COLUMN password_hash     TEXT,
    ADD COLUMN email_verified    BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN email_verify_token TEXT,
    ADD COLUMN password_reset_token TEXT,
    ADD COLUMN password_reset_expires TIMESTAMPTZ;

-- Index for email verification token lookup
CREATE INDEX idx_users_email_verify_token ON users (email_verify_token) WHERE email_verify_token IS NOT NULL;

-- Index for password reset token lookup
CREATE INDEX idx_users_password_reset_token ON users (password_reset_token) WHERE password_reset_token IS NOT NULL;

-- Mark existing dev users as email-verified (they don't have passwords – dev tokens only)
UPDATE users SET email_verified = TRUE WHERE email LIKE '%@placebo.dev';
```

Note: `password_hash` is nullable – allows OAuth-only users (no password, login via provider only).

- [ ] **Step 2: Run migration**

Run: `cd /Users/notebook/Placebo/crates/placebo-api && cargo sqlx migrate run`
Expected: `Applied 007_auth_password` success message.

- [ ] **Step 3: Verify schema**

Run: `psql -d placebo -c "\d users" | head -25`
Expected: `password_hash`, `email_verified`, `email_verify_token`, `password_reset_token`, `password_reset_expires` columns present.

- [ ] **Step 4: Commit**

```bash
git add crates/placebo-api/migrations/007_auth_password.sql
git commit -m "feat(auth): add password_hash and verification fields to users table"
```

---

### Task 2: Shared auth types

**Files:**
- Create: `crates/placebo-shared/src/auth.rs`
- Modify: `crates/placebo-shared/src/lib.rs`

- [ ] **Step 1: Write auth types with tests**

Create `crates/placebo-shared/src/auth.rs`:

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Requests ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
    /// BCP-47 locale tag, e.g. "ru", "en", "ja"
    pub locale: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetConfirm {
    pub token: String,
    pub new_password: String,
}

// ─── Responses ───────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub is_premium: bool,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    pub message: String,
}

// ─── Validation ──────────────────────────────────────────

impl RegisterRequest {
    /// Validate registration input. Returns list of error messages.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Email: basic format check
        let email = self.email.trim();
        if email.is_empty() {
            errors.push("Email is required".into());
        } else if !email.contains('@') || !email.contains('.') {
            errors.push("Invalid email format".into());
        } else if email.len() > 254 {
            errors.push("Email too long (max 254 chars)".into());
        }

        // Password: min 8 chars
        if self.password.len() < 8 {
            errors.push("Password must be at least 8 characters".into());
        }
        if self.password.len() > 128 {
            errors.push("Password too long (max 128 chars)".into());
        }

        // Display name: 1-50 chars
        let name = self.display_name.trim();
        if name.is_empty() {
            errors.push("Display name is required".into());
        } else if name.len() > 50 {
            errors.push("Display name too long (max 50 chars)".into());
        }

        errors
    }
}

impl LoginRequest {
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.email.trim().is_empty() {
            errors.push("Email is required".into());
        }
        if self.password.is_empty() {
            errors.push("Password is required".into());
        }
        errors
    }
}

impl PasswordResetConfirm {
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.token.trim().is_empty() {
            errors.push("Reset token is required".into());
        }
        if self.new_password.len() < 8 {
            errors.push("Password must be at least 8 characters".into());
        }
        if self.new_password.len() > 128 {
            errors.push("Password too long (max 128 chars)".into());
        }
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_request_validates_email() {
        let req = RegisterRequest {
            email: "bad".into(),
            password: "12345678".into(),
            display_name: "Test".into(),
            locale: None,
        };
        let errs = req.validate();
        assert!(errs.iter().any(|e| e.contains("email")), "should reject bad email");
    }

    #[test]
    fn register_request_validates_short_password() {
        let req = RegisterRequest {
            email: "user@test.com".into(),
            password: "short".into(),
            display_name: "Test".into(),
            locale: None,
        };
        let errs = req.validate();
        assert!(errs.iter().any(|e| e.contains("8 characters")), "should reject short password");
    }

    #[test]
    fn register_request_passes_valid_input() {
        let req = RegisterRequest {
            email: "user@test.com".into(),
            password: "securepassword123".into(),
            display_name: "Test User".into(),
            locale: Some("ru".into()),
        };
        assert!(req.validate().is_empty(), "valid input should pass");
    }

    #[test]
    fn auth_response_serializes_camel_case() {
        let resp = AuthResponse {
            token: "abc".into(),
            user_id: uuid::Uuid::nil(),
            email: "a@b.com".into(),
            display_name: "A".into(),
            is_premium: false,
            expires_in_seconds: 3600,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("userId"), "should use camelCase: {json}");
        assert!(json.contains("expiresInSeconds"), "should use camelCase: {json}");
        assert!(json.contains("displayName"), "should use camelCase: {json}");
        // Ensure no sensitive data leaks
        assert!(!json.contains("password"), "must not contain password");
    }

    #[test]
    fn login_request_validates() {
        let req = LoginRequest {
            email: "".into(),
            password: "".into(),
        };
        let errs = req.validate();
        assert_eq!(errs.len(), 2);
    }
}
```

- [ ] **Step 2: Export auth module from lib.rs**

Add to `crates/placebo-shared/src/lib.rs`:

```rust
pub mod auth;
```

And re-export:

```rust
pub use auth::*;
```

- [ ] **Step 3: Run tests**

Run: `cd /Users/notebook/Placebo/crates/placebo-shared && cargo test`
Expected: All tests pass (existing 21 + new 5 = 26).

- [ ] **Step 4: Commit**

```bash
git add crates/placebo-shared/src/auth.rs crates/placebo-shared/src/lib.rs
git commit -m "feat(shared): add auth request/response types with validation"
```

---

## Chunk 2: Dependencies & Repository Layer

### Task 3: Add auth dependencies

**Files:**
- Modify: `crates/placebo-api/Cargo.toml`

- [ ] **Step 1: Add argon2 and rand to Cargo.toml**

Add under `[dependencies]`:

```toml
argon2 = "0.5"
rand = "0.8"
```

- [ ] **Step 2: Verify compilation**

Run: `cd /Users/notebook/Placebo/crates/placebo-api && cargo check`
Expected: Compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add crates/placebo-api/Cargo.toml
git commit -m "chore(api): add argon2 and rand dependencies for auth"
```

---

### Task 4: Extend user repository

**Files:**
- Modify: `crates/placebo-api/src/repositories/user_repo.rs`

- [ ] **Step 1: Add auth-related queries to user_repo.rs**

Append to the existing file (after existing functions):

```rust
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

/// Get password hash for login verification. Returns None if user not found.
pub async fn get_password_hash(
    pool: &PgPool,
    email: &str,
) -> Result<Option<(Uuid, String, bool)>, sqlx::Error> {
    // Returns (user_id, password_hash, email_verified)
    let row = sqlx::query_as::<_, (Uuid, Option<String>, bool)>(
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
    let row = sqlx::query_as::<_, (Uuid,)>(
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
    let row = sqlx::query_as::<_, (Uuid,)>(
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
    let row = sqlx::query_as::<_, (bool,)>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE LOWER(email) = LOWER($1))",
    )
    .bind(email.trim())
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd /Users/notebook/Placebo/crates/placebo-api && cargo check`
Expected: Compiles. Note: `(Uuid, Option<String>, bool)` tuple needs sqlx FromRow – if not supported, will need a helper struct.

- [ ] **Step 3: Commit**

```bash
git add crates/placebo-api/src/repositories/user_repo.rs
git commit -m "feat(repo): add auth queries – create_user, password lookup, email verify, reset"
```

---

### Task 5: Extend Redis session – delete all for user

**Files:**
- Modify: `crates/placebo-api/src/redis/session.rs`

- [ ] **Step 1: Add user-scoped session tracking**

Append to `session.rs`:

```rust
/// Track which tokens belong to a user (for "logout everywhere").
/// Called on session creation.
pub async fn track_user_session(
    pool: &deadpool_redis::Pool,
    user_id: Uuid,
    token: &str,
    ttl_secs: u64,
) -> Result<(), anyhow::Error> {
    let mut conn = pool.get().await?;
    let key = format!("user_sessions:{user_id}");
    redis::cmd("SADD")
        .arg(&key)
        .arg(token)
        .query_async::<()>(&mut *conn)
        .await?;
    // Set TTL on the set itself (cleanup)
    redis::cmd("EXPIRE")
        .arg(&key)
        .arg(ttl_secs)
        .query_async::<()>(&mut *conn)
        .await?;
    Ok(())
}

/// Delete ALL sessions for a user (logout everywhere).
pub async fn delete_all_for_user(
    pool: &deadpool_redis::Pool,
    user_id: Uuid,
) -> Result<u64, anyhow::Error> {
    let mut conn = pool.get().await?;
    let key = format!("user_sessions:{user_id}");

    // Get all tokens
    let tokens: Vec<String> = redis::cmd("SMEMBERS")
        .arg(&key)
        .query_async(&mut *conn)
        .await?;

    let mut deleted = 0u64;
    for token in &tokens {
        let session_key = format!("session:{token}");
        let removed: u64 = redis::cmd("DEL")
            .arg(&session_key)
            .query_async(&mut *conn)
            .await?;
        deleted += removed;
    }

    // Delete the set itself
    redis::cmd("DEL")
        .arg(&key)
        .query_async::<()>(&mut *conn)
        .await?;

    Ok(deleted)
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`

- [ ] **Step 3: Commit**

```bash
git add crates/placebo-api/src/redis/session.rs
git commit -m "feat(redis): add user session tracking and delete-all for logout everywhere"
```

---

## Chunk 3: Auth Service & Handlers

### Task 6: Auth service – business logic

**Files:**
- Create: `crates/placebo-api/src/services/auth_service.rs`
- Modify: `crates/placebo-api/src/services/mod.rs`

- [ ] **Step 1: Create auth_service.rs**

```rust
use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::redis::session::{self, SessionData};
use crate::repositories::user_repo;
use placebo_shared::auth::{AuthResponse, RegisterRequest, LoginRequest};

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

    // 3. Hash password
    let password_hash = hash_password(&req.password)?;

    // 4. Create user in database
    let locale = req.locale.as_deref().unwrap_or("en");
    let user = user_repo::create_user(
        pool,
        &req.email,
        &req.display_name,
        &password_hash,
        locale,
        auto_verify_email,
    )
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.constraint() == Some("users_email_key") => {
            AppError::Conflict("Email already registered".into())
        }
        _ => AppError::Internal(e.into()),
    })?;

    // 5. If not auto-verified, generate verification token and store it
    if !auto_verify_email {
        let verify_token = generate_token();
        user_repo::set_email_verify_token(pool, user.id, &verify_token)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        // TODO: Send verification email via lettre
        tracing::info!(user_id = %user.id, "Email verification token generated (email sending not yet implemented)");
    }

    // 6. Create session
    let token = generate_token();
    let session = SessionData {
        user_id: user.id,
        email: user.email.clone(),
        is_premium: user.is_premium,
        created_at: chrono::Utc::now().timestamp(),
    };
    session::create(redis, &token, &session, SESSION_TTL)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    session::track_user_session(redis, user.id, &token, SESSION_TTL)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(AuthResponse {
        token,
        user_id: user.id,
        email: user.email,
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
        .map_err(|e| AppError::Internal(e.into()))?;
    session::track_user_session(redis, user.id, &token, SESSION_TTL)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(AuthResponse {
        token,
        user_id: user.id,
        email: user.email,
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
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(())
}

/// Logout everywhere – delete ALL sessions for a user.
pub async fn logout_all(
    redis: &deadpool_redis::Pool,
    user_id: Uuid,
) -> Result<u64, AppError> {
    session::delete_all_for_user(redis, user_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))
}

/// Refresh session – extend TTL (sliding window).
pub async fn refresh_session(
    redis: &deadpool_redis::Pool,
    token: &str,
) -> Result<u64, AppError> {
    // Check session exists
    let _session = session::get(redis, token)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::Unauthorized("Invalid or expired session".into()))?;

    // Extend TTL
    session::refresh(redis, token, SESSION_TTL)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

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
```

- [ ] **Step 2: Add hex dependency** (for token encoding)

Add to `crates/placebo-api/Cargo.toml`:

```toml
hex = "0.4"
```

- [ ] **Step 3: Register module in services/mod.rs**

Add to `crates/placebo-api/src/services/mod.rs`:

```rust
pub mod auth_service;
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check`

- [ ] **Step 5: Commit**

```bash
git add crates/placebo-api/src/services/auth_service.rs crates/placebo-api/src/services/mod.rs crates/placebo-api/Cargo.toml
git commit -m "feat(auth): add auth_service with register, login, logout, password reset"
```

---

### Task 7: Auth handlers – HTTP endpoints

**Files:**
- Create: `crates/placebo-api/src/handlers/auth.rs`
- Modify: `crates/placebo-api/src/handlers/mod.rs`

- [ ] **Step 1: Create auth handler**

Create `crates/placebo-api/src/handlers/auth.rs`:

```rust
use axum::{
    extract::State,
    http::HeaderMap,
    Json,
    Router,
    routing::post,
};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::services::auth_service;
use placebo_shared::auth::{
    AuthResponse, LoginRequest, MessageResponse, PasswordResetConfirm,
    PasswordResetRequest, RefreshRequest, RegisterRequest,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/logout-all", post(logout_all))
        .route("/refresh", post(refresh))
        .route("/password-reset", post(request_password_reset))
        .route("/password-reset/confirm", post(confirm_password_reset))
}

/// POST /api/v1/auth/register
async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // In dev, auto-verify email. In prod, require verification.
    let auto_verify = state.config.environment == crate::config::Environment::Dev;
    let resp = auth_service::register(&state.db, &state.redis, &req, auto_verify).await?;
    Ok(Json(resp))
}

/// POST /api/v1/auth/login
async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let resp = auth_service::login(&state.db, &state.redis, &req).await?;
    Ok(Json(resp))
}

/// POST /api/v1/auth/logout – requires auth token in Authorization header
async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MessageResponse>, AppError> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("Missing token".into()))?;

    auth_service::logout(&state.redis, token).await?;
    Ok(Json(MessageResponse {
        message: "Logged out successfully".into(),
    }))
}

/// POST /api/v1/auth/logout-all – logout from all devices
async fn logout_all(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<MessageResponse>, AppError> {
    let count = auth_service::logout_all(&state.redis, auth.id).await?;
    Ok(Json(MessageResponse {
        message: format!("Logged out from {count} session(s)"),
    }))
}

/// POST /api/v1/auth/refresh – extend session TTL
async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let ttl = auth_service::refresh_session(&state.redis, &req.token).await?;
    Ok(Json(MessageResponse {
        message: format!("Session extended for {ttl} seconds"),
    }))
}

/// POST /api/v1/auth/password-reset – request reset email
async fn request_password_reset(
    State(state): State<AppState>,
    Json(req): Json<PasswordResetRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    auth_service::request_password_reset(&state.db, &req.email).await?;
    // Always return success – don't reveal if email exists
    Ok(Json(MessageResponse {
        message: "If an account with that email exists, a reset link has been sent".into(),
    }))
}

/// POST /api/v1/auth/password-reset/confirm – set new password
async fn confirm_password_reset(
    State(state): State<AppState>,
    Json(req): Json<PasswordResetConfirm>,
) -> Result<Json<MessageResponse>, AppError> {
    let errors = req.validate();
    if !errors.is_empty() {
        return Err(AppError::Validation(errors.join("; ")));
    }
    auth_service::confirm_password_reset(&state.db, &state.redis, &req.token, &req.new_password)
        .await?;
    Ok(Json(MessageResponse {
        message: "Password updated successfully. Please login with your new password.".into(),
    }))
}
```

- [ ] **Step 2: Register auth routes in api_router**

Modify `crates/placebo-api/src/handlers/mod.rs` – add `pub mod auth;` and nest the auth router:

In the `api_router()` function, add:

```rust
.nest("/auth", auth::router())
```

alongside existing `.nest("/cameras", ...)`, `.nest("/rooms", ...)`, etc.

- [ ] **Step 3: Verify compilation**

Run: `cargo check`

- [ ] **Step 4: Commit**

```bash
git add crates/placebo-api/src/handlers/auth.rs crates/placebo-api/src/handlers/mod.rs
git commit -m "feat(auth): add register, login, logout, refresh, password-reset endpoints"
```

---

## Chunk 4: Testing & Verification

### Task 8: Integration smoke test

- [ ] **Step 1: Ensure API compiles and starts**

Run: `cd /Users/notebook/Placebo/crates/placebo-api && cargo build`
Expected: Compiles without errors.

- [ ] **Step 2: Start API server**

Run: `cd /Users/notebook/Placebo/crates/placebo-api && cargo run &`
Wait for: `listening on 0.0.0.0:3001`

- [ ] **Step 3: Test registration via curl**

```bash
curl -s -X POST http://localhost:3001/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"email":"test@example.com","password":"testpass123","displayName":"Test User"}' | jq .
```

Expected: JSON with `token`, `userId`, `email`, `displayName`, `isPremium`, `expiresInSeconds`.

- [ ] **Step 4: Test login via curl**

```bash
curl -s -X POST http://localhost:3001/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"test@example.com","password":"testpass123"}' | jq .
```

Expected: JSON with new `token` (different from registration token).

- [ ] **Step 5: Test auth with token**

```bash
TOKEN=$(curl -s -X POST http://localhost:3001/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"test@example.com","password":"testpass123"}' | jq -r .token)

curl -s http://localhost:3001/api/v1/users/me \
  -H "Authorization: Bearer $TOKEN" | jq .
```

Expected: Full user profile with email, displayName, etc.

- [ ] **Step 6: Test duplicate email rejection**

```bash
curl -s -X POST http://localhost:3001/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"email":"test@example.com","password":"other123456","displayName":"Duplicate"}' | jq .
```

Expected: 409 Conflict with "Email already registered".

- [ ] **Step 7: Test bad password rejection**

```bash
curl -s -X POST http://localhost:3001/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"test@example.com","password":"wrongpassword"}' | jq .
```

Expected: 401 Unauthorized with "Invalid email or password".

- [ ] **Step 8: Test validation**

```bash
curl -s -X POST http://localhost:3001/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"email":"bad","password":"short","displayName":""}' | jq .
```

Expected: 400 Validation error listing all issues.

- [ ] **Step 9: Test logout**

```bash
curl -s -X POST http://localhost:3001/api/v1/auth/logout \
  -H "Authorization: Bearer $TOKEN" | jq .
```

Expected: `{"message": "Logged out successfully"}`

Then verify token is invalid:

```bash
curl -s http://localhost:3001/api/v1/users/me \
  -H "Authorization: Bearer $TOKEN" | jq .
```

Expected: 401 Unauthorized.

- [ ] **Step 10: Run all tests**

```bash
cd /Users/notebook/Placebo && cargo test --workspace
```

Expected: All existing tests still pass + new shared auth tests pass.

- [ ] **Step 11: Final commit**

```bash
git add -A
git commit -m "test(auth): verify registration, login, logout, validation flows"
```

---

## Security Notes

**What's protected:**
- Argon2id hashing (OWASP recommended, memory-hard, resistant to GPU attacks)
- 256-bit random session tokens (cryptographically secure via `OsRng`)
- Generic error messages on login (don't reveal if email exists)
- Password reset always returns success (don't reveal if email exists)
- All sessions invalidated on password change
- Session TTL 30 days with refresh capability
- Email normalization (lowercase, trimmed)
- Password length limits (8–128 chars)
- Nullable `password_hash` for future OAuth-only users

**Deferred to future:**
- OAuth providers (Google, Apple, VK, Telegram, Discord, X, Facebook)
- Email verification sending (scaffolded, tokens stored, sending not implemented)
- Rate limiting on auth endpoints (rate_limit.rs exists, needs to be wired)
- CSRF protection (not needed for API-only, tokens in Authorization header)
- Account lockout after N failed attempts
- User roles / permission hierarchy (user said "поговорим позже")
