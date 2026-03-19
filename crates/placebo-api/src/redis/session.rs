use deadpool_redis::Pool as RedisPool;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: Uuid,
    pub email: String,
    pub is_premium: bool,
    pub created_at: i64, // unix timestamp
}

/// Store a session with TTL
pub async fn create(
    pool: &RedisPool,
    token: &str,
    data: &SessionData,
    ttl_secs: u64,
) -> Result<(), redis::RedisError> {
    let mut conn = pool.get().await.map_err(|e| {
        redis::RedisError::from((redis::ErrorKind::IoError, "pool error", e.to_string()))
    })?;
    let key = format!("session:{}", token);
    let json = serde_json::to_string(data).map_err(|e| {
        redis::RedisError::from((
            redis::ErrorKind::IoError,
            "serialize error",
            e.to_string(),
        ))
    })?;
    conn.set_ex::<_, _, ()>(&key, &json, ttl_secs).await
}

/// Get session data by token
pub async fn get(
    pool: &RedisPool,
    token: &str,
) -> Result<Option<SessionData>, redis::RedisError> {
    let mut conn = pool.get().await.map_err(|e| {
        redis::RedisError::from((redis::ErrorKind::IoError, "pool error", e.to_string()))
    })?;
    let key = format!("session:{}", token);
    let json: Option<String> = conn.get(&key).await?;
    match json {
        Some(j) => {
            let data: SessionData = serde_json::from_str(&j).map_err(|e| {
                redis::RedisError::from((
                    redis::ErrorKind::IoError,
                    "deserialize error",
                    e.to_string(),
                ))
            })?;
            Ok(Some(data))
        }
        None => Ok(None),
    }
}

/// Delete a session (logout)
pub async fn delete(pool: &RedisPool, token: &str) -> Result<bool, redis::RedisError> {
    let mut conn = pool.get().await.map_err(|e| {
        redis::RedisError::from((redis::ErrorKind::IoError, "pool error", e.to_string()))
    })?;
    let key = format!("session:{}", token);
    let deleted: i64 = conn.del(&key).await?;
    Ok(deleted > 0)
}

/// Refresh session TTL (extend expiry on activity)
pub async fn refresh(
    pool: &RedisPool,
    token: &str,
    ttl_secs: u64,
) -> Result<bool, redis::RedisError> {
    let mut conn = pool.get().await.map_err(|e| {
        redis::RedisError::from((redis::ErrorKind::IoError, "pool error", e.to_string()))
    })?;
    let key = format!("session:{}", token);
    conn.expire(&key, ttl_secs as i64).await
}

/// Track which tokens belong to a user (for "logout everywhere").
/// Called on session creation.
pub async fn track_user_session(
    pool: &RedisPool,
    user_id: Uuid,
    token: &str,
    ttl_secs: u64,
) -> Result<(), redis::RedisError> {
    let mut conn = pool.get().await.map_err(|e| {
        redis::RedisError::from((redis::ErrorKind::IoError, "pool error", e.to_string()))
    })?;
    let key = format!("user_sessions:{user_id}");
    redis::cmd("SADD")
        .arg(&key)
        .arg(token)
        .query_async::<()>(&mut *conn)
        .await?;
    // Set TTL on the set itself (cleanup)
    redis::cmd("EXPIRE")
        .arg(&key)
        .arg(ttl_secs as i64)
        .query_async::<()>(&mut *conn)
        .await?;
    Ok(())
}

/// Delete ALL sessions for a user (logout everywhere).
pub async fn delete_all_for_user(
    pool: &RedisPool,
    user_id: Uuid,
) -> Result<u64, redis::RedisError> {
    let mut conn = pool.get().await.map_err(|e| {
        redis::RedisError::from((redis::ErrorKind::IoError, "pool error", e.to_string()))
    })?;
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
