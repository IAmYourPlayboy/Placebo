use sqlx::PgPool;
use uuid::Uuid;

use placebo_shared::camera::{
    CameraResponse, CameraType, Category, RetentionTier, StreamProtocol, StreamSourceType,
    StreamType, VideoCodec,
};
use placebo_shared::pagination::{PaginatedResponse, PaginationMeta};

use crate::error::AppError;
use crate::repositories::camera_repo::{self, CameraRow};

// ---------------------------------------------------------------------------
// CameraRow -> CameraResponse mapping (strips sensitive fields)
// ---------------------------------------------------------------------------

pub fn to_response(row: &CameraRow) -> CameraResponse {
    let camera_type = row
        .camera_type
        .parse::<CameraType>()
        .unwrap_or(CameraType::Public);

    let stream_type = row
        .stream_type
        .as_deref()
        .and_then(|s| s.parse::<StreamType>().ok());

    let stream_protocol = row
        .stream_protocol
        .as_deref()
        .and_then(|s| s.parse::<StreamProtocol>().ok());

    let codec = row
        .codec
        .as_deref()
        .and_then(|s| s.parse::<VideoCodec>().ok());

    let category = row
        .category
        .parse::<Category>()
        .unwrap_or(Category::City);

    let retention_tier = row
        .retention_tier
        .parse::<RetentionTier>()
        .unwrap_or(RetentionTier::Tier1);

    let recording_codec = row
        .recording_codec
        .parse::<VideoCodec>()
        .unwrap_or(VideoCodec::H264);

    let stream_source_type = row
        .stream_source_type
        .as_deref()
        .and_then(|s| s.parse::<StreamSourceType>().ok())
        .unwrap_or(StreamSourceType::Rtsp);

    // Public manifest URL: only present for source types we actually proxy.
    let proxy_manifest_url = match stream_source_type {
        StreamSourceType::YoutubeLive
        | StreamSourceType::DirectHls
        | StreamSourceType::LoopMp4 => Some(format!("/api/v1/hls-proxy/{}", row.slug)),
        StreamSourceType::Rtsp => None,
    };

    let available_qualities: Vec<String> = match &row.available_qualities {
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => vec![],
    };

    let tags: Vec<String> = match &row.tags {
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => vec![],
    };

    CameraResponse {
        id: row.id,
        name: row.name.clone(),
        slug: row.slug.clone(),
        camera_type,

        country: row.country.clone(),
        country_code: row.country_code.clone(),
        region: row.region.clone(),
        city: row.city.clone(),
        district: row.district.clone(),
        address: row.address.clone(),
        custom_label: row.custom_label.clone(),

        lat: row.lat,
        lng: row.lng,
        timezone: row.timezone.clone(),

        // Sensitive fields (stream_url, backup_url, external_id, frame_rate) are NOT included
        stream_type,
        stream_protocol,
        stream_source_type,
        proxy_manifest_url,
        stream_quality_default: row.stream_quality_default.clone(),
        available_qualities,

        bitrate_kbps: row.bitrate_kbps,
        codec,
        resolution_w: row.resolution_w.map(|v| v as i32),
        resolution_h: row.resolution_h.map(|v| v as i32),
        latency_ms: row.latency_ms.map(|v| v as i32),

        has_audio: row.has_audio,
        has_night_vision: row.has_night_vision,
        is_underwater: row.is_underwater,

        category,
        subcategory: row.subcategory.clone(),
        tags,

        description_en: row.description_en.clone(),
        thumbnail_url: row.thumbnail_url.clone(),
        source_url: row.source_url.clone(),
        attribution: row.attribution.clone(),

        recording_enabled: row.recording_enabled,
        retention_tier,
        recording_retention_days: row.recording_retention_days as i32,
        recording_codec,

        height_above_ground: row.height_above_ground.map(|v| v as f64),
        camera_azimuth: row.camera_azimuth.map(|v| v as f64),
        camera_elevation: row.camera_elevation.map(|v| v as f64),
        fov_horizontal: row.fov_horizontal.map(|v| v as f64),
        fov_vertical: row.fov_vertical.map(|v| v as f64),

        manufacturer: row.manufacturer.clone(),
        camera_model: row.camera_model.clone(),

        added_to_placebo_at: row.added_to_placebo_at,

        is_partner_camera: row.is_partner_camera,
        owner_name: row.owner_name.clone(),

        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

// ---------------------------------------------------------------------------
// Service functions
// ---------------------------------------------------------------------------

/// Paginated camera list with optional filters.
pub async fn list_cameras(
    pool: &PgPool,
    page: u32,
    per_page: u32,
    category: Option<&str>,
    camera_type: Option<&str>,
) -> Result<PaginatedResponse<CameraResponse>, AppError> {
    let per_page = per_page.min(200);
    let offset = (page.saturating_sub(1) as i64) * (per_page as i64);
    let limit = per_page as i64;

    let total = camera_repo::get_count_filtered(pool, category, camera_type).await?;
    let rows = camera_repo::get_all(pool, limit, offset).await?;

    let total_pages = if total == 0 {
        0
    } else {
        ((total as f64) / (per_page as f64)).ceil() as u32
    };

    Ok(PaginatedResponse {
        data: rows.iter().map(to_response).collect(),
        meta: PaginationMeta {
            page,
            per_page,
            total,
            total_pages,
        },
    })
}

/// Single camera by UUID.
pub async fn get_camera(pool: &PgPool, id: Uuid) -> Result<CameraResponse, AppError> {
    let row = camera_repo::get_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Camera {id} not found")))?;
    Ok(to_response(&row))
}

/// Nearby cameras via PostGIS.
pub async fn get_nearby(
    pool: &PgPool,
    lat: f64,
    lng: f64,
    radius_m: f64,
    limit: i64,
) -> Result<Vec<CameraResponse>, AppError> {
    let rows = camera_repo::get_nearby(pool, lat, lng, radius_m, limit).await?;
    Ok(rows.iter().map(to_response).collect())
}

/// Cameras within bounding box via PostGIS.
pub async fn get_in_bbox(
    pool: &PgPool,
    sw_lat: f64,
    sw_lng: f64,
    ne_lat: f64,
    ne_lng: f64,
    limit: i64,
) -> Result<Vec<CameraResponse>, AppError> {
    let rows = camera_repo::get_in_bbox(pool, sw_lat, sw_lng, ne_lat, ne_lng, limit).await?;
    Ok(rows.iter().map(to_response).collect())
}

/// Fuzzy search cameras.
pub async fn search_cameras(
    pool: &PgPool,
    query: &str,
    limit: i64,
) -> Result<Vec<CameraResponse>, AppError> {
    let limit = limit.min(200);
    let rows = camera_repo::search(pool, query, limit).await?;
    Ok(rows.iter().map(to_response).collect())
}

/// All distinct categories currently in use.
pub async fn get_categories(pool: &PgPool) -> Result<Vec<String>, AppError> {
    let cats = camera_repo::get_categories(pool).await?;
    Ok(cats)
}

/// Camera count with optional filters.
pub async fn get_count(
    pool: &PgPool,
    category: Option<&str>,
    camera_type: Option<&str>,
) -> Result<i64, AppError> {
    let count = camera_repo::get_count_filtered(pool, category, camera_type).await?;
    Ok(count)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Build a minimal CameraRow with the given stream_source_type and slug.
    /// All other fields get sensible defaults so to_response() can map cleanly.
    fn make_row(stream_source_type: Option<&str>, slug: &str) -> CameraRow {
        CameraRow {
            id: Uuid::new_v4(),
            name: "Test Camera".to_string(),
            slug: slug.to_string(),
            camera_type: "public".to_string(),
            external_id: None,

            country: None,
            country_code: None,
            region: None,
            city: None,
            district: None,
            address: None,
            custom_label: None,
            lat: 0.0,
            lng: 0.0,
            timezone: None,

            stream_url: "rtsp://example/foo".to_string(),
            backup_url: None,
            stream_type: None,
            stream_protocol: None,
            stream_quality_default: None,
            available_qualities: serde_json::json!([]),
            frame_rate: None,
            bitrate_kbps: None,
            codec: None,
            resolution_w: None,
            resolution_h: None,
            latency_ms: None,

            has_audio: false,
            has_night_vision: false,
            is_underwater: false,

            category: "city".to_string(),
            subcategory: None,
            tags: serde_json::json!([]),
            description_en: None,
            thumbnail_url: None,
            source_url: None,
            attribution: None,

            recording_enabled: false,
            retention_tier: "tier5".to_string(),
            recording_retention_days: 0,
            recording_codec: "h264".to_string(),

            height_above_ground: None,
            camera_azimuth: None,
            camera_elevation: None,
            fov_horizontal: None,
            fov_vertical: None,

            manufacturer: None,
            camera_model: None,
            added_to_placebo_at: None,
            is_partner_camera: false,
            owner_name: None,

            stream_source_type: stream_source_type.map(String::from),
            stream_source_config: serde_json::json!({}),

            created_at: Utc::now(),
            updated_at: None,

            distance_m: None,
        }
    }

    #[test]
    fn to_response_youtube_live_sets_proxy_manifest_url() {
        let row = make_row(Some("youtube_live"), "yt-cam");
        let resp = to_response(&row);
        assert_eq!(resp.stream_source_type, StreamSourceType::YoutubeLive);
        assert_eq!(
            resp.proxy_manifest_url,
            Some("/api/v1/hls-proxy/yt-cam".to_string())
        );
    }

    #[test]
    fn to_response_direct_hls_sets_proxy_manifest_url() {
        let row = make_row(Some("direct_hls"), "hls-cam");
        let resp = to_response(&row);
        assert_eq!(resp.stream_source_type, StreamSourceType::DirectHls);
        assert_eq!(
            resp.proxy_manifest_url,
            Some("/api/v1/hls-proxy/hls-cam".to_string())
        );
    }

    #[test]
    fn to_response_loop_mp4_sets_proxy_manifest_url() {
        let row = make_row(Some("loop_mp4"), "loop-cam");
        let resp = to_response(&row);
        assert_eq!(resp.stream_source_type, StreamSourceType::LoopMp4);
        assert_eq!(
            resp.proxy_manifest_url,
            Some("/api/v1/hls-proxy/loop-cam".to_string())
        );
    }

    #[test]
    fn to_response_rtsp_has_no_proxy_manifest_url() {
        let row = make_row(Some("rtsp"), "rtsp-cam");
        let resp = to_response(&row);
        assert_eq!(resp.stream_source_type, StreamSourceType::Rtsp);
        assert_eq!(resp.proxy_manifest_url, None);
    }

    #[test]
    fn to_response_null_stream_source_type_falls_back_to_rtsp() {
        let row = make_row(None, "null-cam");
        let resp = to_response(&row);
        assert_eq!(resp.stream_source_type, StreamSourceType::Rtsp);
        assert_eq!(resp.proxy_manifest_url, None);
    }

    #[test]
    fn to_response_unparseable_stream_source_type_falls_back_to_rtsp() {
        let row = make_row(Some("bogus"), "bogus-cam");
        let resp = to_response(&row);
        assert_eq!(resp.stream_source_type, StreamSourceType::Rtsp);
        assert_eq!(resp.proxy_manifest_url, None);
    }

    #[test]
    fn to_response_strips_sensitive_fields_from_json() {
        let row = make_row(Some("youtube_live"), "sensitive-cam");
        let resp = to_response(&row);
        let json = serde_json::to_string(&resp).unwrap();

        // CameraResponse must not leak sensitive DB fields, in any case.
        assert!(!json.contains("streamUrl"), "streamUrl leaked: {json}");
        assert!(!json.contains("stream_url"), "stream_url leaked: {json}");
        assert!(!json.contains("backupUrl"), "backupUrl leaked: {json}");
        assert!(!json.contains("backup_url"), "backup_url leaked: {json}");
        assert!(!json.contains("externalId"), "externalId leaked: {json}");
        assert!(!json.contains("external_id"), "external_id leaked: {json}");
        assert!(!json.contains("frameRate"), "frameRate leaked: {json}");
        assert!(!json.contains("frame_rate"), "frame_rate leaked: {json}");

        // And the public fields we just tested must be present.
        assert!(json.contains("\"streamSourceType\":\"youtube_live\""));
        assert!(json.contains("\"proxyManifestUrl\":\"/api/v1/hls-proxy/sensitive-cam\""));
    }
}
