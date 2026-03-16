use deadpool_redis::Pool as RedisPool;
use redis::AsyncCommands;
use uuid::Uuid;

/// Increment viewer count for a camera (when user starts watching)
pub async fn increment(pool: &RedisPool, camera_id: Uuid) -> Result<i64, redis::RedisError> {
    let mut conn = pool.get().await.map_err(|e| {
        redis::RedisError::from((redis::ErrorKind::IoError, "pool error", e.to_string()))
    })?;
    let key = format!("camera:{}:viewers", camera_id);
    conn.incr(&key, 1i64).await
}

/// Decrement viewer count (when user stops watching)
pub async fn decrement(pool: &RedisPool, camera_id: Uuid) -> Result<i64, redis::RedisError> {
    let mut conn = pool.get().await.map_err(|e| {
        redis::RedisError::from((redis::ErrorKind::IoError, "pool error", e.to_string()))
    })?;
    let key = format!("camera:{}:viewers", camera_id);
    let count: i64 = conn.decr(&key, 1i64).await?;
    // Don't go below 0
    if count < 0 {
        conn.set::<_, _, ()>(&key, 0i64).await?;
        return Ok(0);
    }
    Ok(count)
}

/// Get current viewer count
pub async fn get(pool: &RedisPool, camera_id: Uuid) -> Result<i64, redis::RedisError> {
    let mut conn = pool.get().await.map_err(|e| {
        redis::RedisError::from((redis::ErrorKind::IoError, "pool error", e.to_string()))
    })?;
    let key = format!("camera:{}:viewers", camera_id);
    let count: Option<i64> = conn.get(&key).await?;
    Ok(count.unwrap_or(0))
}

/// Get viewer counts for multiple cameras (batch)
pub async fn get_many(
    pool: &RedisPool,
    camera_ids: &[Uuid],
) -> Result<Vec<(Uuid, i64)>, redis::RedisError> {
    if camera_ids.is_empty() {
        return Ok(vec![]);
    }
    let mut conn = pool.get().await.map_err(|e| {
        redis::RedisError::from((redis::ErrorKind::IoError, "pool error", e.to_string()))
    })?;
    let keys: Vec<String> = camera_ids
        .iter()
        .map(|id| format!("camera:{}:viewers", id))
        .collect();
    let counts: Vec<Option<i64>> = redis::cmd("MGET")
        .arg(&keys)
        .query_async(&mut conn)
        .await?;

    Ok(camera_ids
        .iter()
        .zip(counts.iter())
        .map(|(id, count)| (*id, count.unwrap_or(0)))
        .collect())
}

/// Set camera online status with TTL (heartbeat pattern)
pub async fn set_online(
    pool: &RedisPool,
    camera_id: Uuid,
    ttl_secs: u64,
) -> Result<(), redis::RedisError> {
    let mut conn = pool.get().await.map_err(|e| {
        redis::RedisError::from((redis::ErrorKind::IoError, "pool error", e.to_string()))
    })?;
    let key = format!("camera:{}:online", camera_id);
    conn.set_ex::<_, _, ()>(&key, "1", ttl_secs).await
}

/// Check if camera is online
pub async fn is_online(pool: &RedisPool, camera_id: Uuid) -> Result<bool, redis::RedisError> {
    let mut conn = pool.get().await.map_err(|e| {
        redis::RedisError::from((redis::ErrorKind::IoError, "pool error", e.to_string()))
    })?;
    let key = format!("camera:{}:online", camera_id);
    let exists: bool = conn.exists(&key).await?;
    Ok(exists)
}
