use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// SQL fragment for all SELECT queries – extracts lat/lng from GEOMETRY column
// ---------------------------------------------------------------------------

const CAMERA_SELECT: &str = r#"
    id, name, slug, camera_type::TEXT, external_id,
    country, country_code, region, city, district, address, custom_label,
    ST_Y(location) as lat, ST_X(location) as lng, timezone,
    stream_url, backup_url, stream_type::TEXT, stream_protocol::TEXT,
    stream_quality_default, available_qualities, frame_rate, bitrate_kbps,
    codec::TEXT, resolution_w, resolution_h, latency_ms,
    has_audio, has_night_vision, is_underwater,
    category, subcategory, tags, description_en, thumbnail_url, source_url, attribution,
    recording_enabled, retention_tier::TEXT, recording_retention_days, recording_codec::TEXT,
    height_above_ground, camera_azimuth, camera_elevation, fov_horizontal, fov_vertical,
    manufacturer, camera_model, added_to_placebo_at, is_partner_camera, owner_name,
    stream_source_type::TEXT as stream_source_type,
    stream_source_config,
    created_at, updated_at
"#;

// ---------------------------------------------------------------------------
// CameraRow – raw database row
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CameraRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub camera_type: String,
    pub external_id: Option<String>,

    // Location
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub address: Option<String>,
    pub custom_label: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub timezone: Option<String>,

    // Stream (sensitive)
    pub stream_url: String,
    pub backup_url: Option<String>,
    pub stream_type: Option<String>,
    pub stream_protocol: Option<String>,
    pub stream_quality_default: Option<String>,
    pub available_qualities: serde_json::Value,
    pub frame_rate: Option<i16>,
    pub bitrate_kbps: Option<i32>,
    pub codec: Option<String>,
    pub resolution_w: Option<i16>,
    pub resolution_h: Option<i16>,
    pub latency_ms: Option<i16>,

    // Capabilities
    pub has_audio: bool,
    pub has_night_vision: bool,
    pub is_underwater: bool,

    // Meta
    pub category: String,
    pub subcategory: Option<String>,
    pub tags: serde_json::Value,
    pub description_en: Option<String>,
    pub thumbnail_url: Option<String>,
    pub source_url: Option<String>,
    pub attribution: Option<String>,

    // Recording
    pub recording_enabled: bool,
    pub retention_tier: String,
    pub recording_retention_days: i16,
    pub recording_codec: String,

    // 3D
    pub height_above_ground: Option<f32>,
    pub camera_azimuth: Option<f32>,
    pub camera_elevation: Option<f32>,
    pub fov_horizontal: Option<f32>,
    pub fov_vertical: Option<f32>,

    // Partner
    pub manufacturer: Option<String>,
    pub camera_model: Option<String>,
    pub added_to_placebo_at: Option<DateTime<Utc>>,
    pub is_partner_camera: bool,
    pub owner_name: Option<String>,

    // Stream source descriptor (M3)
    pub stream_source_type: Option<String>,
    pub stream_source_config: serde_json::Value,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,

    // Computed (only in some queries)
    #[sqlx(default)]
    pub distance_m: Option<f64>,
}

// ---------------------------------------------------------------------------
// NewCamera – for INSERT
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct NewCamera {
    pub name: String,
    pub slug: String,
    pub camera_type: String,
    pub external_id: Option<String>,

    pub country: Option<String>,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub address: Option<String>,
    pub custom_label: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub timezone: Option<String>,

    pub stream_url: String,
    pub backup_url: Option<String>,
    pub stream_type: Option<String>,
    pub stream_protocol: Option<String>,
    pub stream_quality_default: Option<String>,
    pub available_qualities: serde_json::Value,
    pub frame_rate: Option<i16>,
    pub bitrate_kbps: Option<i32>,
    pub codec: Option<String>,
    pub resolution_w: Option<i16>,
    pub resolution_h: Option<i16>,
    pub latency_ms: Option<i16>,

    pub has_audio: bool,
    pub has_night_vision: bool,
    pub is_underwater: bool,

    pub category: String,
    pub subcategory: Option<String>,
    pub tags: serde_json::Value,
    pub description_en: Option<String>,
    pub thumbnail_url: Option<String>,
    pub source_url: Option<String>,
    pub attribution: Option<String>,

    pub recording_enabled: bool,
    pub retention_tier: String,
    pub recording_retention_days: i16,
    pub recording_codec: String,

    pub height_above_ground: Option<f32>,
    pub camera_azimuth: Option<f32>,
    pub camera_elevation: Option<f32>,
    pub fov_horizontal: Option<f32>,
    pub fov_vertical: Option<f32>,

    pub manufacturer: Option<String>,
    pub camera_model: Option<String>,
    pub is_partner_camera: bool,
    pub owner_name: Option<String>,

    pub stream_source_type: Option<String>,
    pub stream_source_config: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Repository functions
// ---------------------------------------------------------------------------

/// Paginated list of all cameras.
pub async fn get_all(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<CameraRow>, sqlx::Error> {
    let sql = format!(
        "SELECT {CAMERA_SELECT}, NULL::float8 as distance_m FROM cameras ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    );
    sqlx::query_as::<_, CameraRow>(&sql)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
}

/// Total camera count (no filters).
pub async fn get_count(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cameras")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Camera count with optional category and camera_type filters.
pub async fn get_count_filtered(
    pool: &PgPool,
    category: Option<&str>,
    camera_type: Option<&str>,
) -> Result<i64, sqlx::Error> {
    // Build dynamic WHERE clause based on which filters are present
    match (category, camera_type) {
        (None, None) => get_count(pool).await,
        (Some(cat), None) => {
            let row: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM cameras WHERE category = $1",
            )
            .bind(cat)
            .fetch_one(pool)
            .await?;
            Ok(row.0)
        }
        (None, Some(ct)) => {
            let row: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM cameras WHERE camera_type::TEXT = $1",
            )
            .bind(ct)
            .fetch_one(pool)
            .await?;
            Ok(row.0)
        }
        (Some(cat), Some(ct)) => {
            let row: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM cameras WHERE category = $1 AND camera_type::TEXT = $2",
            )
            .bind(cat)
            .bind(ct)
            .fetch_one(pool)
            .await?;
            Ok(row.0)
        }
    }
}

/// Find camera by UUID.
pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<CameraRow>, sqlx::Error> {
    let sql = format!(
        "SELECT {CAMERA_SELECT}, NULL::float8 as distance_m FROM cameras WHERE id = $1"
    );
    sqlx::query_as::<_, CameraRow>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Find camera by slug.
pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<Option<CameraRow>, sqlx::Error> {
    let sql = format!(
        "SELECT {CAMERA_SELECT}, NULL::float8 as distance_m FROM cameras WHERE slug = $1"
    );
    sqlx::query_as::<_, CameraRow>(&sql)
        .bind(slug)
        .fetch_optional(pool)
        .await
}

/// Lightweight lookup used only by the HLS proxy. Returns the source
/// type and JSON config for a camera identified by slug. Never exposes
/// `stream_url` itself – the proxy resolves the upstream URL via
/// `services::hls_source` from the descriptor instead.
pub async fn stream_source_for_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<(String, serde_json::Value)>, sqlx::Error> {
    let row: Option<(Option<String>, serde_json::Value)> = sqlx::query_as(
        "SELECT stream_source_type::text, stream_source_config FROM cameras WHERE slug = $1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|(t, c)| t.map(|tt| (tt, c))))
}

/// Find cameras by city name (case-insensitive).
pub async fn get_by_city(pool: &PgPool, city: &str) -> Result<Vec<CameraRow>, sqlx::Error> {
    let sql = format!(
        "SELECT {CAMERA_SELECT}, NULL::float8 as distance_m FROM cameras WHERE LOWER(city) = LOWER($1) ORDER BY name"
    );
    sqlx::query_as::<_, CameraRow>(&sql)
        .bind(city)
        .fetch_all(pool)
        .await
}

/// PostGIS: find cameras within `radius_m` meters of a point, ordered by distance.
pub async fn get_nearby(
    pool: &PgPool,
    lat: f64,
    lng: f64,
    radius_m: f64,
    limit: i64,
) -> Result<Vec<CameraRow>, sqlx::Error> {
    let sql = format!(
        r#"SELECT {CAMERA_SELECT},
           ST_Distance(location::geography, ST_Point($2, $1)::geography) as distance_m
           FROM cameras
           WHERE ST_DWithin(location::geography, ST_Point($2, $1)::geography, $3)
           ORDER BY distance_m
           LIMIT $4"#
    );
    sqlx::query_as::<_, CameraRow>(&sql)
        .bind(lat)
        .bind(lng)
        .bind(radius_m)
        .bind(limit)
        .fetch_all(pool)
        .await
}

/// PostGIS: find cameras within a bounding box.
pub async fn get_in_bbox(
    pool: &PgPool,
    sw_lat: f64,
    sw_lng: f64,
    ne_lat: f64,
    ne_lng: f64,
    limit: i64,
) -> Result<Vec<CameraRow>, sqlx::Error> {
    // ST_MakeEnvelope(xmin, ymin, xmax, ymax, srid) = (sw_lng, sw_lat, ne_lng, ne_lat, 4326)
    let sql = format!(
        r#"SELECT {CAMERA_SELECT}, NULL::float8 as distance_m
           FROM cameras
           WHERE location && ST_MakeEnvelope($1, $2, $3, $4, 4326)
           ORDER BY name
           LIMIT $5"#
    );
    sqlx::query_as::<_, CameraRow>(&sql)
        .bind(sw_lng)
        .bind(sw_lat)
        .bind(ne_lng)
        .bind(ne_lat)
        .bind(limit)
        .fetch_all(pool)
        .await
}

/// Fuzzy search using pg_trgm similarity + ILIKE.
pub async fn search(
    pool: &PgPool,
    query: &str,
    limit: i64,
) -> Result<Vec<CameraRow>, sqlx::Error> {
    let pattern = format!("%{query}%");
    let sql = format!(
        r#"SELECT {CAMERA_SELECT}, NULL::float8 as distance_m
           FROM cameras
           WHERE name ILIKE $1
              OR city ILIKE $1
              OR country ILIKE $1
              OR region ILIKE $1
              OR tags::text ILIKE $1
              OR description_en ILIKE $1
           ORDER BY similarity(name, $2) DESC, name
           LIMIT $3"#
    );
    sqlx::query_as::<_, CameraRow>(&sql)
        .bind(&pattern)
        .bind(query)
        .bind(limit)
        .fetch_all(pool)
        .await
}

/// Distinct categories currently in use.
pub async fn get_categories(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT DISTINCT category FROM cameras ORDER BY category")
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// Insert a new camera, returning the full row.
pub async fn insert(pool: &PgPool, c: &NewCamera) -> Result<CameraRow, sqlx::Error> {
    let sql = format!(
        r#"INSERT INTO cameras (
            name, slug, camera_type, external_id,
            country, country_code, region, city, district, address, custom_label,
            location, timezone,
            stream_url, backup_url, stream_type, stream_protocol,
            stream_quality_default, available_qualities, frame_rate, bitrate_kbps,
            codec, resolution_w, resolution_h, latency_ms,
            has_audio, has_night_vision, is_underwater,
            category, subcategory, tags, description_en, thumbnail_url, source_url, attribution,
            recording_enabled, retention_tier, recording_retention_days, recording_codec,
            height_above_ground, camera_azimuth, camera_elevation, fov_horizontal, fov_vertical,
            manufacturer, camera_model, is_partner_camera, owner_name,
            stream_source_type, stream_source_config
        ) VALUES (
            $1, $2, $3::camera_type, $4,
            $5, $6, $7, $8, $9, $10, $11,
            ST_SetSRID(ST_Point($12, $13), 4326), $14,
            $15, $16, $17::stream_type, $18::stream_protocol,
            $19, $20, $21, $22,
            $23::video_codec, $24, $25, $26,
            $27, $28, $29,
            $30, $31, $32, $33, $34, $35, $36,
            $37, $38::retention_tier, $39, $40::video_codec,
            $41, $42, $43, $44, $45,
            $46, $47, $48, $49,
            $50::stream_source_type, $51
        ) RETURNING {CAMERA_SELECT}, NULL::float8 as distance_m"#
    );

    // ST_Point(lng, lat) – longitude first
    sqlx::query_as::<_, CameraRow>(&sql)
        .bind(&c.name)                      // $1
        .bind(&c.slug)                      // $2
        .bind(&c.camera_type)               // $3
        .bind(&c.external_id)               // $4
        .bind(&c.country)                   // $5
        .bind(&c.country_code)              // $6
        .bind(&c.region)                    // $7
        .bind(&c.city)                      // $8
        .bind(&c.district)                  // $9
        .bind(&c.address)                   // $10
        .bind(&c.custom_label)              // $11
        .bind(c.lng)                        // $12 – lng first for ST_Point
        .bind(c.lat)                        // $13
        .bind(&c.timezone)                  // $14
        .bind(&c.stream_url)               // $15
        .bind(&c.backup_url)               // $16
        .bind(&c.stream_type)              // $17
        .bind(&c.stream_protocol)          // $18
        .bind(&c.stream_quality_default)   // $19
        .bind(&c.available_qualities)      // $20
        .bind(c.frame_rate)                // $21
        .bind(c.bitrate_kbps)              // $22
        .bind(&c.codec)                    // $23
        .bind(c.resolution_w)              // $24
        .bind(c.resolution_h)              // $25
        .bind(c.latency_ms)                // $26
        .bind(c.has_audio)                 // $27
        .bind(c.has_night_vision)          // $28
        .bind(c.is_underwater)             // $29
        .bind(&c.category)                 // $30
        .bind(&c.subcategory)              // $31
        .bind(&c.tags)                     // $32
        .bind(&c.description_en)           // $33
        .bind(&c.thumbnail_url)            // $34
        .bind(&c.source_url)               // $35
        .bind(&c.attribution)              // $36
        .bind(c.recording_enabled)         // $37
        .bind(&c.retention_tier)           // $38
        .bind(c.recording_retention_days)  // $39
        .bind(&c.recording_codec)          // $40
        .bind(c.height_above_ground)       // $41
        .bind(c.camera_azimuth)            // $42
        .bind(c.camera_elevation)          // $43
        .bind(c.fov_horizontal)            // $44
        .bind(c.fov_vertical)              // $45
        .bind(&c.manufacturer)             // $46
        .bind(&c.camera_model)             // $47
        .bind(c.is_partner_camera)         // $48
        .bind(&c.owner_name)               // $49
        .bind(&c.stream_source_type)       // $50
        .bind(&c.stream_source_config)     // $51
        .fetch_one(pool)
        .await
}

/// Delete a camera by ID. Returns `true` if a row was deleted.
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM cameras WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
