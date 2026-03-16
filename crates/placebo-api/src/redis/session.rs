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
