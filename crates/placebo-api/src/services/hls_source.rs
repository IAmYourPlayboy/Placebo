//! HLS source resolver for the camera proxy.
//!
//! Given a camera slug, returns the upstream the proxy should fetch:
//! a YouTube live HLS URL (resolved via yt-dlp, cached in Redis),
//! a directly-configured HLS URL, a static loop served by our own
//! ServeDir, or an explicitly unsupported source (e.g. RTSP) which
//! the proxy converts into a 404 in the alpha.

use anyhow::{anyhow, Context, Result};
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::Pool as RedisPool;
use serde_json::Value;
use sqlx::PgPool;
use tokio::process::Command;

use crate::repositories::camera_repo;

const CACHE_TTL_SECS: u64 = 30 * 60;

/// Resolved upstream the proxy should fetch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedSource {
    /// Absolute m3u8 URL the proxy should fetch and rewrite.
    Hls(String),
    /// Relative path served by our own ServeDir handler.
    StaticLoop(String),
    /// Slug exists but the source type is intentionally unsupported in the alpha (e.g. rtsp).
    Unsupported,
    /// Slug does not exist.
    NotFound,
}

pub async fn resolve(
    pg: &PgPool,
    redis: &RedisPool,
    slug: &str,
) -> Result<ResolvedSource> {
    let row = camera_repo::stream_source_for_slug(pg, slug)
        .await
        .context("db lookup failed")?;
    let (kind, cfg) = match row {
        Some(p) => p,
        None => return Ok(ResolvedSource::NotFound),
    };

    match kind.as_str() {
        "youtube_live" => {
            let video_id = cfg
                .get("videoId")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("youtube_live config missing videoId"))?;
            let url = resolve_youtube_cached(redis, slug, video_id).await?;
            Ok(ResolvedSource::Hls(url))
        }
        "direct_hls" => {
            let url = cfg
                .get("url")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("direct_hls config missing url"))?;
            Ok(ResolvedSource::Hls(url.to_string()))
        }
        "loop_mp4" => {
            let asset = cfg
                .get("asset")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("loop_mp4 config missing asset"))?;
            Ok(ResolvedSource::StaticLoop(format!(
                "/static/demo/{asset}/index.m3u8"
            )))
        }
        "rtsp" => Ok(ResolvedSource::Unsupported),
        other => Err(anyhow!("unknown stream_source_type: {other}")),
    }
}

fn cache_key(slug: &str) -> String {
    format!("hls:src:{slug}")
}

async fn resolve_youtube_cached(
    redis: &RedisPool,
    slug: &str,
    video_id: &str,
) -> Result<String> {
    let key = cache_key(slug);

    // Best-effort cache read. Redis being unavailable should never block playback.
    if let Ok(mut conn) = redis.get().await {
        if let Ok(Some(cached)) = conn.get::<_, Option<String>>(&key).await {
            tracing::debug!(slug, "hls source cache hit");
            return Ok(cached);
        }
    }

    let url = resolve_youtube(video_id).await?;

    // Best-effort cache write – proxy should still work if redis is down.
    if let Ok(mut conn) = redis.get().await {
        let _: redis::RedisResult<()> = conn.set_ex(&key, &url, CACHE_TTL_SECS).await;
    }
    Ok(url)
}

async fn resolve_youtube(video_id: &str) -> Result<String> {
    let target = format!("https://www.youtube.com/watch?v={video_id}");
    let output = Command::new("yt-dlp")
        .args(["-f", "best[vcodec^=avc1]", "--no-warnings", "-g", &target])
        .output()
        .await
        .context("failed to spawn yt-dlp (is it installed and on PATH?)")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "yt-dlp failed (status={:?}): {}",
            output.status.code(),
            stderr.trim()
        ));
    }
    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        return Err(anyhow!("yt-dlp returned empty stdout"));
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_format_is_stable() {
        // External scripts (e.g. ops dashboards) may inspect Redis with this prefix.
        assert_eq!(cache_key("yt-shibuya-crossing"), "hls:src:yt-shibuya-crossing");
        assert_eq!(cache_key("demo-tokyo-alley"), "hls:src:demo-tokyo-alley");
    }
}
