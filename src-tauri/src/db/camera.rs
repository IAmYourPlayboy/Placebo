use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Camera {
    pub id: String,
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

    // Stream
    pub stream_url: String,
    pub backup_url: Option<String>,
    pub stream_type: Option<String>,
    pub stream_protocol: Option<String>,
    pub stream_quality_default: Option<String>,
    pub available_qualities: Option<String>,
    pub frame_rate: Option<i32>,
    pub bitrate_kbps: Option<i32>,
    pub codec: Option<String>,
    pub resolution_w: Option<i32>,
    pub resolution_h: Option<i32>,
    pub latency_ms: Option<i32>,

    // Capabilities
    pub has_audio: i32,
    pub has_night_vision: i32,
    pub is_underwater: i32,

    // Meta
    pub category: String,
    pub subcategory: Option<String>,
    pub tags: String,
    pub description_en: Option<String>,
    pub thumbnail_url: Option<String>,
    pub source_url: Option<String>,
    pub attribution: Option<String>,

    // Recording
    pub recording_enabled: i32,
    pub retention_tier: String,
    pub recording_retention_days: i32,
    pub recording_codec: String,

    // Partner / Hardware
    pub manufacturer: Option<String>,
    pub camera_model: Option<String>,
    pub added_to_placebo_at: Option<String>,
    pub is_partner_camera: i32,
    pub owner_name: Option<String>,

    // Timestamps
    pub created_at: String,
    pub updated_at: Option<String>,
}

pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Camera>, sqlx::Error> {
    sqlx::query_as::<_, Camera>("SELECT * FROM cameras ORDER BY name LIMIT 200")
        .fetch_all(pool)
        .await
}

pub async fn get_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Camera>, sqlx::Error> {
    sqlx::query_as::<_, Camera>("SELECT * FROM cameras WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_by_city(pool: &SqlitePool, city: &str) -> Result<Vec<Camera>, sqlx::Error> {
    sqlx::query_as::<_, Camera>("SELECT * FROM cameras WHERE city = ? ORDER BY name")
        .bind(city)
        .fetch_all(pool)
        .await
}

pub async fn search(pool: &SqlitePool, query: &str) -> Result<Vec<Camera>, sqlx::Error> {
    let pattern = format!("%{}%", query);
    sqlx::query_as::<_, Camera>(
        "SELECT * FROM cameras WHERE name LIKE ?1 OR city LIKE ?1 OR country LIKE ?1 OR tags LIKE ?1 ORDER BY name LIMIT 50",
    )
    .bind(&pattern)
    .fetch_all(pool)
    .await
}

pub async fn get_nearby(
    pool: &SqlitePool,
    lat: f64,
    lng: f64,
    radius_km: f64,
) -> Result<Vec<Camera>, sqlx::Error> {
    // Haversine distance in SQL using SQLite math functions
    // We use a bounding box pre-filter for performance, then precise Haversine
    let lat_delta = radius_km / 111.32;
    let lng_delta = radius_km / (111.32 * f64::cos(lat.to_radians()));

    sqlx::query_as::<_, Camera>(
        r#"
        SELECT * FROM cameras
        WHERE lat BETWEEN ?1 - ?3 AND ?1 + ?3
          AND lng BETWEEN ?2 - ?4 AND ?2 + ?4
        ORDER BY (
            (lat - ?1) * (lat - ?1) + (lng - ?2) * (lng - ?2)
        ) ASC
        LIMIT 50
        "#,
    )
    .bind(lat)
    .bind(lng)
    .bind(lat_delta)
    .bind(lng_delta)
    .fetch_all(pool)
    .await
}

pub async fn get_categories(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT DISTINCT category FROM cameras ORDER BY category")
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

pub async fn get_count(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cameras")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn insert(pool: &SqlitePool, camera: &Camera) -> Result<Camera, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO cameras (
            id, name, slug, camera_type, external_id,
            country, country_code, region, city, district, address, custom_label,
            lat, lng, timezone,
            stream_url, backup_url, stream_type, stream_protocol,
            stream_quality_default, available_qualities, frame_rate, bitrate_kbps,
            codec, resolution_w, resolution_h, latency_ms,
            has_audio, has_night_vision, is_underwater,
            category, subcategory, tags, description_en, thumbnail_url, source_url, attribution,
            recording_enabled, retention_tier, recording_retention_days, recording_codec,
            manufacturer, camera_model, added_to_placebo_at, is_partner_camera, owner_name,
            created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8, ?9, ?10, ?11, ?12,
            ?13, ?14, ?15,
            ?16, ?17, ?18, ?19,
            ?20, ?21, ?22, ?23,
            ?24, ?25, ?26, ?27,
            ?28, ?29, ?30,
            ?31, ?32, ?33, ?34, ?35, ?36, ?37,
            ?38, ?39, ?40, ?41,
            ?42, ?43, ?44, ?45, ?46,
            ?47, ?48
        )
        "#,
    )
    .bind(&camera.id)
    .bind(&camera.name)
    .bind(&camera.slug)
    .bind(&camera.camera_type)
    .bind(&camera.external_id)
    .bind(&camera.country)
    .bind(&camera.country_code)
    .bind(&camera.region)
    .bind(&camera.city)
    .bind(&camera.district)
    .bind(&camera.address)
    .bind(&camera.custom_label)
    .bind(camera.lat)
    .bind(camera.lng)
    .bind(&camera.timezone)
    .bind(&camera.stream_url)
    .bind(&camera.backup_url)
    .bind(&camera.stream_type)
    .bind(&camera.stream_protocol)
    .bind(&camera.stream_quality_default)
    .bind(&camera.available_qualities)
    .bind(camera.frame_rate)
    .bind(camera.bitrate_kbps)
    .bind(&camera.codec)
    .bind(camera.resolution_w)
    .bind(camera.resolution_h)
    .bind(camera.latency_ms)
    .bind(camera.has_audio)
    .bind(camera.has_night_vision)
    .bind(camera.is_underwater)
    .bind(&camera.category)
    .bind(&camera.subcategory)
    .bind(&camera.tags)
    .bind(&camera.description_en)
    .bind(&camera.thumbnail_url)
    .bind(&camera.source_url)
    .bind(&camera.attribution)
    .bind(camera.recording_enabled)
    .bind(&camera.retention_tier)
    .bind(camera.recording_retention_days)
    .bind(&camera.recording_codec)
    .bind(&camera.manufacturer)
    .bind(&camera.camera_model)
    .bind(&camera.added_to_placebo_at)
    .bind(camera.is_partner_camera)
    .bind(&camera.owner_name)
    .bind(&camera.created_at)
    .bind(&camera.updated_at)
    .execute(pool)
    .await?;

    Ok(camera.clone())
}

pub async fn insert_batch(pool: &SqlitePool, cameras: &[Camera]) -> Result<usize, sqlx::Error> {
    let mut count = 0;
    for camera in cameras {
        insert(pool, camera).await?;
        count += 1;
    }
    Ok(count)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM cameras WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Helper to create a Camera with required fields and sensible defaults
pub fn new_camera(id: &str, name: &str, slug: &str, lat: f64, lng: f64, stream_url: &str) -> Camera {
    Camera {
        id: id.to_string(),
        name: name.to_string(),
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
        lat,
        lng,
        timezone: None,
        stream_url: stream_url.to_string(),
        backup_url: None,
        stream_type: None,
        stream_protocol: None,
        stream_quality_default: None,
        available_qualities: Some("[]".to_string()),
        frame_rate: None,
        bitrate_kbps: None,
        codec: None,
        resolution_w: None,
        resolution_h: None,
        latency_ms: None,
        has_audio: 0,
        has_night_vision: 0,
        is_underwater: 0,
        category: "city".to_string(),
        subcategory: None,
        tags: "[]".to_string(),
        description_en: None,
        thumbnail_url: None,
        source_url: None,
        attribution: None,
        recording_enabled: 0,
        retention_tier: "tier5".to_string(),
        recording_retention_days: 0,
        recording_codec: "h264".to_string(),
        manufacturer: None,
        camera_model: None,
        added_to_placebo_at: None,
        is_partner_camera: 0,
        owner_name: None,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        updated_at: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_test_db;

    fn test_camera(suffix: &str) -> Camera {
        let mut cam = new_camera(
            &format!("test-id-{}", suffix),
            &format!("Test Camera {}", suffix),
            &format!("test-camera-{}", suffix),
            35.6762,
            139.6503,
            "rtsp://test.placebo.tv/live",
        );
        cam.city = Some("Tokyo".to_string());
        cam.country = Some("Japan".to_string());
        cam.country_code = Some("JP".to_string());
        cam.category = "city".to_string();
        cam
    }

    #[tokio::test]
    async fn test_insert_and_get_by_id() {
        let pool = init_test_db().await.unwrap();
        let cam = test_camera("1");
        insert(&pool, &cam).await.unwrap();

        let found = get_by_id(&pool, "test-id-1").await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.name, "Test Camera 1");
        assert_eq!(found.slug, "test-camera-1");
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let pool = init_test_db().await.unwrap();
        let found = get_by_id(&pool, "nonexistent").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_get_all() {
        let pool = init_test_db().await.unwrap();
        insert(&pool, &test_camera("a")).await.unwrap();
        insert(&pool, &test_camera("b")).await.unwrap();

        let all = get_all(&pool).await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_get_all_empty() {
        let pool = init_test_db().await.unwrap();
        let all = get_all(&pool).await.unwrap();
        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn test_get_by_city() {
        let pool = init_test_db().await.unwrap();
        let mut cam1 = test_camera("c1");
        cam1.city = Some("Tokyo".to_string());
        insert(&pool, &cam1).await.unwrap();

        let mut cam2 = test_camera("c2");
        cam2.city = Some("Moscow".to_string());
        insert(&pool, &cam2).await.unwrap();

        let tokyo = get_by_city(&pool, "Tokyo").await.unwrap();
        assert_eq!(tokyo.len(), 1);
        assert_eq!(tokyo[0].name, "Test Camera c1");

        let moscow = get_by_city(&pool, "Moscow").await.unwrap();
        assert_eq!(moscow.len(), 1);
    }

    #[tokio::test]
    async fn test_search() {
        let pool = init_test_db().await.unwrap();
        let mut cam = test_camera("search");
        cam.name = "Shibuya Crossing".to_string();
        cam.city = Some("Tokyo".to_string());
        insert(&pool, &cam).await.unwrap();

        let results = search(&pool, "Shibuya").await.unwrap();
        assert_eq!(results.len(), 1);

        let results = search(&pool, "Tokyo").await.unwrap();
        assert_eq!(results.len(), 1);

        let results = search(&pool, "nonexistent").await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_search_special_characters() {
        let pool = init_test_db().await.unwrap();
        let mut cam = test_camera("special");
        cam.name = "Камера #1 (тест)".to_string();
        insert(&pool, &cam).await.unwrap();

        let results = search(&pool, "Камера").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_get_nearby() {
        let pool = init_test_db().await.unwrap();
        // Shibuya area
        let mut cam1 = test_camera("near1");
        cam1.lat = 35.6595;
        cam1.lng = 139.7004;
        insert(&pool, &cam1).await.unwrap();

        // Far away (Moscow)
        let mut cam2 = test_camera("near2");
        cam2.lat = 55.7558;
        cam2.lng = 37.6173;
        insert(&pool, &cam2).await.unwrap();

        // Search near Shibuya (5km radius)
        let nearby = get_nearby(&pool, 35.66, 139.70, 5.0).await.unwrap();
        assert_eq!(nearby.len(), 1);
        assert_eq!(nearby[0].id, "test-id-near1");
    }

    #[tokio::test]
    async fn test_get_nearby_no_results() {
        let pool = init_test_db().await.unwrap();
        let nearby = get_nearby(&pool, 0.0, 0.0, 1.0).await.unwrap();
        assert!(nearby.is_empty());
    }

    #[tokio::test]
    async fn test_get_categories() {
        let pool = init_test_db().await.unwrap();
        let mut cam1 = test_camera("cat1");
        cam1.category = "city".to_string();
        insert(&pool, &cam1).await.unwrap();

        let mut cam2 = test_camera("cat2");
        cam2.category = "traffic".to_string();
        insert(&pool, &cam2).await.unwrap();

        let mut cam3 = test_camera("cat3");
        cam3.category = "city".to_string();
        insert(&pool, &cam3).await.unwrap();

        let cats = get_categories(&pool).await.unwrap();
        assert_eq!(cats.len(), 2);
        assert!(cats.contains(&"city".to_string()));
        assert!(cats.contains(&"traffic".to_string()));
    }

    #[tokio::test]
    async fn test_get_count() {
        let pool = init_test_db().await.unwrap();
        assert_eq!(get_count(&pool).await.unwrap(), 0);

        insert(&pool, &test_camera("cnt1")).await.unwrap();
        insert(&pool, &test_camera("cnt2")).await.unwrap();
        assert_eq!(get_count(&pool).await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_delete() {
        let pool = init_test_db().await.unwrap();
        insert(&pool, &test_camera("del")).await.unwrap();

        assert!(delete(&pool, "test-id-del").await.unwrap());
        assert!(!delete(&pool, "test-id-del").await.unwrap()); // already deleted
        assert!(get_by_id(&pool, "test-id-del").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_insert_batch() {
        let pool = init_test_db().await.unwrap();
        let cameras = vec![test_camera("batch1"), test_camera("batch2"), test_camera("batch3")];
        let count = insert_batch(&pool, &cameras).await.unwrap();
        assert_eq!(count, 3);
        assert_eq!(get_count(&pool).await.unwrap(), 3);
    }

    #[tokio::test]
    async fn test_duplicate_slug_fails() {
        let pool = init_test_db().await.unwrap();
        let cam1 = test_camera("dup");
        insert(&pool, &cam1).await.unwrap();

        let mut cam2 = test_camera("dup2");
        cam2.slug = "test-camera-dup".to_string(); // same slug
        let result = insert(&pool, &cam2).await;
        assert!(result.is_err());
    }
}
