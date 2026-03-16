use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecordingSegment {
    pub id: String,
    pub camera_id: String,
    pub start_time: String,
    pub end_time: String,
    pub duration_seconds: i32,
    pub storage_tier: String,
    pub codec: String,
    pub file_path: String,
    pub file_size_bytes: i64,
    pub resolution: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClipRequest {
    pub id: String,
    pub camera_id: String,
    pub user_id: String,
    pub start_time: String,
    pub end_time: String,
    pub status: String,
    pub output_url: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

// ─── RecordingSegment CRUD ──────────────────────────────────────────────────

pub async fn insert_segment(
    pool: &SqlitePool,
    segment: &RecordingSegment,
) -> Result<RecordingSegment, sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO recording_segments (id, camera_id, start_time, end_time, duration_seconds, storage_tier, codec, file_path, file_size_bytes, resolution, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
    )
    .bind(&segment.id)
    .bind(&segment.camera_id)
    .bind(&segment.start_time)
    .bind(&segment.end_time)
    .bind(segment.duration_seconds)
    .bind(&segment.storage_tier)
    .bind(&segment.codec)
    .bind(&segment.file_path)
    .bind(segment.file_size_bytes)
    .bind(&segment.resolution)
    .bind(&segment.created_at)
    .execute(pool)
    .await?;

    Ok(segment.clone())
}

pub async fn get_segments_by_camera(
    pool: &SqlitePool,
    camera_id: &str,
) -> Result<Vec<RecordingSegment>, sqlx::Error> {
    sqlx::query_as::<_, RecordingSegment>(
        "SELECT * FROM recording_segments WHERE camera_id = ? ORDER BY start_time DESC",
    )
    .bind(camera_id)
    .fetch_all(pool)
    .await
}

pub async fn get_segments_by_time_range(
    pool: &SqlitePool,
    camera_id: &str,
    start: &str,
    end: &str,
) -> Result<Vec<RecordingSegment>, sqlx::Error> {
    sqlx::query_as::<_, RecordingSegment>(
        "SELECT * FROM recording_segments WHERE camera_id = ? AND start_time >= ? AND end_time <= ? ORDER BY start_time",
    )
    .bind(camera_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
}

pub async fn delete_expired_segments(
    pool: &SqlitePool,
    before: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM recording_segments WHERE end_time < ?")
        .bind(before)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

// ─── ClipRequest CRUD ───────────────────────────────────────────────────────

pub async fn create_clip_request(
    pool: &SqlitePool,
    clip: &ClipRequest,
) -> Result<ClipRequest, sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO clip_requests (id, camera_id, user_id, start_time, end_time, status, output_url, created_at, completed_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
    )
    .bind(&clip.id)
    .bind(&clip.camera_id)
    .bind(&clip.user_id)
    .bind(&clip.start_time)
    .bind(&clip.end_time)
    .bind(&clip.status)
    .bind(&clip.output_url)
    .bind(&clip.created_at)
    .bind(&clip.completed_at)
    .execute(pool)
    .await?;

    Ok(clip.clone())
}

pub async fn update_clip_status(
    pool: &SqlitePool,
    id: &str,
    status: &str,
    output_url: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let completed_at = if status == "ready" || status == "failed" {
        Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
    } else {
        None
    };

    let result = sqlx::query(
        "UPDATE clip_requests SET status = ?, output_url = ?, completed_at = ? WHERE id = ?",
    )
    .bind(status)
    .bind(output_url)
    .bind(&completed_at)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_clips_by_user(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<ClipRequest>, sqlx::Error> {
    sqlx::query_as::<_, ClipRequest>(
        "SELECT * FROM clip_requests WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn get_clips_by_camera(
    pool: &SqlitePool,
    camera_id: &str,
) -> Result<Vec<ClipRequest>, sqlx::Error> {
    sqlx::query_as::<_, ClipRequest>(
        "SELECT * FROM clip_requests WHERE camera_id = ? ORDER BY created_at DESC",
    )
    .bind(camera_id)
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::camera;
    use crate::db::init_test_db;

    async fn setup_camera(pool: &SqlitePool) -> String {
        let cam = camera::new_camera(
            "cam-rec-test",
            "Test Cam",
            "test-cam-rec",
            35.68,
            139.69,
            "rtsp://test/live",
        );
        camera::insert(pool, &cam).await.unwrap();
        cam.id
    }

    fn test_segment(camera_id: &str, suffix: &str) -> RecordingSegment {
        RecordingSegment {
            id: format!("seg-{}", suffix),
            camera_id: camera_id.to_string(),
            start_time: "2026-03-15 10:00:00".to_string(),
            end_time: "2026-03-15 10:05:00".to_string(),
            duration_seconds: 300,
            storage_tier: "hot".to_string(),
            codec: "h264".to_string(),
            file_path: format!("/recordings/{}.ts", suffix),
            file_size_bytes: 15_000_000,
            resolution: Some("1920x1080".to_string()),
            created_at: "2026-03-15 10:05:00".to_string(),
        }
    }

    fn test_clip(camera_id: &str, suffix: &str) -> ClipRequest {
        ClipRequest {
            id: format!("clip-{}", suffix),
            camera_id: camera_id.to_string(),
            user_id: "user-1".to_string(),
            start_time: "2026-03-15 10:00:00".to_string(),
            end_time: "2026-03-15 10:02:00".to_string(),
            status: "pending".to_string(),
            output_url: None,
            created_at: "2026-03-15 10:05:00".to_string(),
            completed_at: None,
        }
    }

    #[tokio::test]
    async fn test_insert_and_get_segment() {
        let pool = init_test_db().await.unwrap();
        let cam_id = setup_camera(&pool).await;
        let seg = test_segment(&cam_id, "1");
        insert_segment(&pool, &seg).await.unwrap();

        let segments = get_segments_by_camera(&pool, &cam_id).await.unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].duration_seconds, 300);
    }

    #[tokio::test]
    async fn test_get_segments_by_time_range() {
        let pool = init_test_db().await.unwrap();
        let cam_id = setup_camera(&pool).await;

        let mut seg1 = test_segment(&cam_id, "range1");
        seg1.start_time = "2026-03-15 10:00:00".to_string();
        seg1.end_time = "2026-03-15 10:05:00".to_string();
        insert_segment(&pool, &seg1).await.unwrap();

        let mut seg2 = test_segment(&cam_id, "range2");
        seg2.start_time = "2026-03-15 11:00:00".to_string();
        seg2.end_time = "2026-03-15 11:05:00".to_string();
        insert_segment(&pool, &seg2).await.unwrap();

        let results = get_segments_by_time_range(
            &pool, &cam_id, "2026-03-15 09:00:00", "2026-03-15 10:30:00",
        ).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "seg-range1");
    }

    #[tokio::test]
    async fn test_delete_expired_segments() {
        let pool = init_test_db().await.unwrap();
        let cam_id = setup_camera(&pool).await;

        let mut seg = test_segment(&cam_id, "expired");
        seg.end_time = "2026-01-01 00:00:00".to_string();
        insert_segment(&pool, &seg).await.unwrap();

        let deleted = delete_expired_segments(&pool, "2026-03-01 00:00:00").await.unwrap();
        assert_eq!(deleted, 1);

        let segments = get_segments_by_camera(&pool, &cam_id).await.unwrap();
        assert!(segments.is_empty());
    }

    #[tokio::test]
    async fn test_create_and_get_clip() {
        let pool = init_test_db().await.unwrap();
        let cam_id = setup_camera(&pool).await;
        let clip = test_clip(&cam_id, "1");
        create_clip_request(&pool, &clip).await.unwrap();

        let clips = get_clips_by_camera(&pool, &cam_id).await.unwrap();
        assert_eq!(clips.len(), 1);
        assert_eq!(clips[0].status, "pending");
    }

    #[tokio::test]
    async fn test_update_clip_status() {
        let pool = init_test_db().await.unwrap();
        let cam_id = setup_camera(&pool).await;
        let clip = test_clip(&cam_id, "status");
        create_clip_request(&pool, &clip).await.unwrap();

        let updated = update_clip_status(
            &pool, "clip-status", "ready", Some("https://r2.placebo.tv/clips/test.mp4"),
        ).await.unwrap();
        assert!(updated);

        let clips = get_clips_by_camera(&pool, &cam_id).await.unwrap();
        assert_eq!(clips[0].status, "ready");
        assert!(clips[0].completed_at.is_some());
        assert_eq!(clips[0].output_url.as_deref(), Some("https://r2.placebo.tv/clips/test.mp4"));
    }

    #[tokio::test]
    async fn test_get_clips_by_user() {
        let pool = init_test_db().await.unwrap();
        let cam_id = setup_camera(&pool).await;

        let clip1 = test_clip(&cam_id, "user1");
        create_clip_request(&pool, &clip1).await.unwrap();

        let mut clip2 = test_clip(&cam_id, "user2");
        clip2.user_id = "user-2".to_string();
        create_clip_request(&pool, &clip2).await.unwrap();

        let user1_clips = get_clips_by_user(&pool, "user-1").await.unwrap();
        assert_eq!(user1_clips.len(), 1);

        let user2_clips = get_clips_by_user(&pool, "user-2").await.unwrap();
        assert_eq!(user2_clips.len(), 1);
    }

    #[tokio::test]
    async fn test_update_nonexistent_clip() {
        let pool = init_test_db().await.unwrap();
        let updated = update_clip_status(&pool, "nonexistent", "ready", None).await.unwrap();
        assert!(!updated);
    }

    #[tokio::test]
    async fn test_empty_segments_for_camera() {
        let pool = init_test_db().await.unwrap();
        let segments = get_segments_by_camera(&pool, "nonexistent").await.unwrap();
        assert!(segments.is_empty());
    }

    #[tokio::test]
    async fn test_empty_clips_for_user() {
        let pool = init_test_db().await.unwrap();
        let clips = get_clips_by_user(&pool, "nonexistent").await.unwrap();
        assert!(clips.is_empty());
    }

    #[tokio::test]
    async fn test_clip_processing_lifecycle() {
        let pool = init_test_db().await.unwrap();
        let cam_id = setup_camera(&pool).await;
        let clip = test_clip(&cam_id, "lifecycle");
        create_clip_request(&pool, &clip).await.unwrap();

        // pending → processing
        update_clip_status(&pool, "clip-lifecycle", "processing", None).await.unwrap();
        let clips = get_clips_by_camera(&pool, &cam_id).await.unwrap();
        assert_eq!(clips[0].status, "processing");
        assert!(clips[0].completed_at.is_none());

        // processing → ready
        update_clip_status(&pool, "clip-lifecycle", "ready", Some("https://output.mp4")).await.unwrap();
        let clips = get_clips_by_camera(&pool, &cam_id).await.unwrap();
        assert_eq!(clips[0].status, "ready");
        assert!(clips[0].completed_at.is_some());
    }
}
