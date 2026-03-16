use deadpool_redis::Pool as RedisPool;
use std::time::{SystemTime, UNIX_EPOCH};

/// Check rate limit and consume one request.
/// Returns Ok(remaining) if allowed, or Err with retry_after seconds if exceeded.
pub async fn check_rate_limit(
    pool: &RedisPool,
    key: &str, // e.g., "rate:192.168.1.1:/api/v1/cameras"
    max_requests: u32,
    window_secs: u64,
) -> Result<u32, u64> {
    let mut conn = pool.get().await.map_err(|_| 1u64)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as f64;

    let window_ms = (window_secs * 1000) as f64;
    let window_start = now - window_ms;

    // Use pipeline for atomicity
    let mut pipe = redis::pipe();
    pipe.atomic();

    // Remove expired entries
    pipe.zrembyscore(key, 0f64, window_start);

    // Count current entries
    pipe.zcard(key);

    // Add current request
    pipe.zadd(key, now, now.to_string());

    // Set TTL on the key
    pipe.expire(key, window_secs as i64 + 1);

    let results: Vec<i64> = pipe.query_async(&mut conn).await.map_err(|_| 1u64)?;
    let current_count = results[1] as u32;

    if current_count >= max_requests {
        // Rate limited – remove the request we just added since it's over limit
        let _: Result<(), _> = redis::cmd("ZREM")
            .arg(key)
            .arg(now.to_string())
            .query_async(&mut conn)
            .await;

        let retry_after = window_secs; // simplified: wait for full window
        Err(retry_after)
    } else {
        Ok(max_requests - current_count - 1)
    }
}

/// Helper to build rate limit key from IP and endpoint
pub fn rate_key(ip: &str, endpoint: &str) -> String {
    format!("rate:{}:{}", ip, endpoint)
}
