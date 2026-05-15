# Milestone 3: Cameras Seed + HLS Proxy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Перенести HLS-прокси из Vite-middleware на axum-бэкенд (production-ready), расширить camera-схему полем `stream_source_config`, переписать seed до 10-15 реальных и 3-5 тестовых камер, добавить эндпоинты `GET /cameras` и `GET /cameras/:id` с ts-rs-типами.

**Architecture:**
- **Схема:** добавить `stream_source_type` (enum: `youtube_live`, `direct_hls`, `loop_mp4`, `rtsp`) и `stream_source_config JSONB` в `cameras`. Существующие `stream_url`, `stream_type` остаются для legacy/RTSP, но **не отдаются в API**.
- **HLS-прокси на axum:** `GET /api/v1/hls-proxy/:slug` + `GET /api/v1/hls-proxy/:slug/seg` с Redis-кешем разрешённого upstream-URL (TTL 30 минут). yt-dlp вызываем через `tokio::process::Command`. Сегменты проксируются stream-ом без буферизации.
- **API камер:** `GET /api/v1/cameras` с пагинацией и фильтрами; `GET /api/v1/cameras/:id`. Публичные DTO (`CameraSummary`, `CameraDetail`) никогда не содержат `stream_url`.
- **Seed:** 18-20 строк с новыми полями; реальные HLS-источники + loop-MP4 для демо.
- **Vite middleware** (`hls-proxy` блок) удаляется; прод идёт через axum.

**Tech Stack:** axum, sqlx, deadpool-redis, tokio::process, reqwest (streaming), ts-rs.

**Spec reference:** `docs/superpowers/specs/2026-05-14-alpha-design.md`, разделы 5.1–5.2, 7.4, 9, 2.3.

**Зависимости:** M0 (ts-rs), M2 (auth context для защиты некоторых эндпоинтов).

---

## File Map

### Backend (create/modify)

- Create: `crates/placebo-api/migrations/009_camera_stream_sources.sql`
- Create: `crates/placebo-api/migrations/010_seed_alpha_cameras.sql`
- Modify: `crates/placebo-shared/src/camera.rs` (CameraSummary/Detail/StreamSourceType + ts-rs)
- Modify: `crates/placebo-api/src/repositories/camera_repo.rs`
- Modify: `crates/placebo-api/src/services/camera_service.rs`
- Modify: `crates/placebo-api/src/handlers/cameras.rs`
- Create: `crates/placebo-api/src/handlers/hls_proxy.rs`
- Create: `crates/placebo-api/src/services/hls_source.rs`
- Modify: `crates/placebo-api/Cargo.toml` (reqwest в обычные deps, обеспечить `stream` feature)
- Modify: точка монтирования роутера (вероятно `lib.rs` или `app_state.rs`).

### Frontend

- Modify: `vite.config.ts` (удалить hls-proxy блок)
- Modify: `src/hooks/useNearbyCameras.ts` (URL теперь `/api/v1/hls-proxy/:slug`)
- Create: `src/api/cameras.ts` (обёртка)
- Modify: `src/hooks/useCityTiles.ts` если там хардкод URL – подтянуть из `VITE_API_BASE_URL`.
- Modify: `src/screens/World3DScreen.tsx` при необходимости (не ломая текущую работу; основной рефакторинг в M4).

---

## Task 1: Ветка

- [ ] **Step 1**

```bash
git -C d:/Projects/Placebo checkout main && git -C d:/Projects/Placebo pull
git -C d:/Projects/Placebo checkout -b feat/m3-cameras-hls
```

---

## Task 2: Миграция 009 – stream_source колонки

**Files:** `crates/placebo-api/migrations/009_camera_stream_sources.sql`

- [ ] **Step 1: Миграция**

```sql
-- 009_camera_stream_sources.sql
-- Adds a first-class stream source descriptor for cameras.
-- The existing stream_url column remains for legacy/RTSP ingest but
-- is never exposed via the public API.

CREATE TYPE stream_source_type AS ENUM (
    'youtube_live',
    'direct_hls',
    'loop_mp4',
    'rtsp'
);

ALTER TABLE cameras
    ADD COLUMN stream_source_type    stream_source_type,
    ADD COLUMN stream_source_config  JSONB NOT NULL DEFAULT '{}';

CREATE INDEX idx_cameras_stream_source_type ON cameras (stream_source_type);

-- Back-fill existing rows: assume YouTube for slugs listed in vite.config
-- historic map; fallback to rtsp for others.
UPDATE cameras SET
    stream_source_type = 'youtube_live',
    stream_source_config = jsonb_build_object('videoId', CASE slug
        WHEN 'shibuya-crossing' THEN 'dfVK7ld38Ys'
        WHEN 'shibuya-station' THEN 'DjdUEyjx8GM'
        WHEN 'hachiko-square'  THEN 'ehJbjfH1dIo'
        WHEN 'center-gai'      THEN '6dp-bvQ7RWo'
        ELSE NULL END)
WHERE slug IN ('shibuya-crossing','shibuya-station','hachiko-square','center-gai');

UPDATE cameras
SET stream_source_type = 'rtsp'
WHERE stream_source_type IS NULL;
```

- [ ] **Step 2: Прогнать**

```bash
cd d:/Projects/Placebo/crates/placebo-api
cargo sqlx migrate run
```

- [ ] **Step 3: Commit**

```bash
git add crates/placebo-api/migrations/009_camera_stream_sources.sql
git commit -m "feat(db): 009 adds stream_source_type/config to cameras"
```

---

## Task 3: Миграция 010 – alpha seed

**Files:** `crates/placebo-api/migrations/010_seed_alpha_cameras.sql`

- [ ] **Step 1: Seed**

Реальный отбор источников мы делаем вручную. Для плана фиксирую структуру и 18 конкретных записей (их надо будет верифицировать – YouTube-ID могут измениться, проверка в Task 7).

```sql
-- 010_seed_alpha_cameras.sql
-- Alpha-release camera roster: 13 live + 5 looped demo cameras.
-- The existing 50 dev cameras from 005 remain for migration convenience
-- but are NOT surfaced in the alpha UI (filtered by stream_source_type).

-- LIVE YouTube cameras (13)
INSERT INTO cameras (
    name, slug, camera_type,
    country, country_code, city,
    location, timezone,
    stream_url, stream_type,
    stream_source_type, stream_source_config,
    category, description_en, thumbnail_url,
    resolution_w, resolution_h, codec,
    height_above_ground, camera_azimuth, camera_elevation, fov_horizontal, fov_vertical,
    created_at
) VALUES
    ('Shibuya Crossing – Live',          'yt-shibuya-crossing',  'public', 'Japan', 'JP', 'Tokyo',     ST_SetSRID(ST_Point(139.7004, 35.6595), 4326), 'Asia/Tokyo',     'youtube://dfVK7ld38Ys', 'youtube', 'youtube_live', '{"videoId":"dfVK7ld38Ys"}',   'city',    'Shibuya Crossing live',                NULL, 1920, 1080, 'h264', 8, 180, -20, 80, 50, NOW()),
    ('Times Square – NYC Live',          'yt-times-square',      'public', 'USA',   'US', 'New York',  ST_SetSRID(ST_Point(-73.9857,  40.7580), 4326), 'America/New_York','youtube://u4UZ4UvZXrg', 'youtube', 'youtube_live', '{"videoId":"u4UZ4UvZXrg"}',   'city',    'Times Square live panoramic',          NULL, 1920, 1080, 'h264', 12, 90, -10, 90, 55, NOW()),
    ('Abbey Road Crossing – London',     'yt-abbey-road',        'public', 'UK',    'GB', 'London',    ST_SetSRID(ST_Point(-0.1779,   51.5320), 4326), 'Europe/London',  'youtube://FQWkgr0aHlI', 'youtube', 'youtube_live', '{"videoId":"FQWkgr0aHlI"}',   'city',    'Abbey Road zebra crossing live',       NULL, 1920, 1080, 'h264', 4, 0,  -30, 70, 45, NOW()),
    ('Helsinki Senate Square',           'yt-helsinki-senate',   'public', 'Finland','FI','Helsinki',  ST_SetSRID(ST_Point(24.9525,   60.1699), 4326), 'Europe/Helsinki','youtube://AdUw5RdyZxI', 'youtube', 'youtube_live', '{"videoId":"AdUw5RdyZxI"}',   'city',    'Senate Square & Helsinki Cathedral',   NULL, 1920, 1080, 'h264', 20, 135,-12, 85, 50, NOW()),
    ('Red Square – Moscow',              'yt-red-square',        'public', 'Russia','RU', 'Moscow',    ST_SetSRID(ST_Point(37.6208,   55.7539), 4326), 'Europe/Moscow',  'youtube://h1wly909BYw', 'youtube', 'youtube_live', '{"videoId":"h1wly909BYw"}',   'city',    'Red Square live feed',                 NULL, 1920, 1080, 'h264', 10, 180,-8,  90, 52, NOW()),
    ('Gateway of India – Mumbai',        'yt-gateway-mumbai',    'public', 'India', 'IN', 'Mumbai',    ST_SetSRID(ST_Point(72.8347,   18.9220), 4326), 'Asia/Kolkata',   'youtube://7Bl5p4VTXzQ', 'youtube', 'youtube_live', '{"videoId":"7Bl5p4VTXzQ"}',   'city',    'Gateway of India',                     NULL, 1920, 1080, 'h264', 6, 90, -15, 80, 50, NOW()),
    ('Dubai Marina',                     'yt-dubai-marina',      'public', 'UAE',   'AE', 'Dubai',     ST_SetSRID(ST_Point(55.1403,   25.0777), 4326), 'Asia/Dubai',     'youtube://2L4yhCmGRWg', 'youtube', 'youtube_live', '{"videoId":"2L4yhCmGRWg"}',   'city',    'Dubai Marina skyline',                 NULL, 1920, 1080, 'h264', 40, 45, -20, 85, 50, NOW()),
    ('Eiffel Tower cam',                 'yt-eiffel',            'public', 'France','FR', 'Paris',     ST_SetSRID(ST_Point(2.2945,    48.8584), 4326), 'Europe/Paris',   'youtube://dyWHmEQAVUI', 'youtube', 'youtube_live', '{"videoId":"dyWHmEQAVUI"}',   'city',    'Eiffel Tower live',                    NULL, 1920, 1080, 'h264', 25, 300,-5,  85, 50, NOW()),
    ('Venice – Rialto Bridge',           'yt-rialto-venice',     'public', 'Italy', 'IT', 'Venice',    ST_SetSRID(ST_Point(12.3359,   45.4380), 4326), 'Europe/Rome',    'youtube://qMksIqJv3pI', 'youtube', 'youtube_live', '{"videoId":"qMksIqJv3pI"}',   'city',    'Grand Canal at Rialto',                NULL, 1920, 1080, 'h264', 5, 180,-10, 80, 50, NOW()),
    ('Seoul – Gwanghwamun Square',       'yt-gwanghwamun',       'public', 'Korea', 'KR', 'Seoul',     ST_SetSRID(ST_Point(126.9768,  37.5759), 4326), 'Asia/Seoul',     'youtube://wNmMr_ATI2E', 'youtube', 'youtube_live', '{"videoId":"wNmMr_ATI2E"}',   'city',    'Gwanghwamun Square',                   NULL, 1920, 1080, 'h264', 8, 0,  -12, 85, 50, NOW()),
    ('Barcelona – La Rambla',            'yt-la-rambla',         'public', 'Spain', 'ES', 'Barcelona', ST_SetSRID(ST_Point(2.1724,    41.3809), 4326), 'Europe/Madrid',  'youtube://hSbkw-F7bzY', 'youtube', 'youtube_live', '{"videoId":"hSbkw-F7bzY"}',   'city',    'La Rambla pedestrian street',          NULL, 1920, 1080, 'h264', 6, 90, -15, 80, 50, NOW()),
    ('Bondi Beach – Sydney',             'yt-bondi',             'public', 'Australia','AU','Sydney',  ST_SetSRID(ST_Point(151.2767,  -33.8908),4326), 'Australia/Sydney','youtube://2Te5EvOXNZw','youtube','youtube_live', '{"videoId":"2Te5EvOXNZw"}',   'nature',  'Bondi Beach live',                     NULL, 1920, 1080, 'h264', 10, 60, -8, 90, 55, NOW()),
    ('Amsterdam Canals',                 'yt-amsterdam-canal',   'public', 'Netherlands','NL','Amsterdam',ST_SetSRID(ST_Point(4.8917,  52.3730),4326), 'Europe/Amsterdam','youtube://SkdGPWUUkEw','youtube','youtube_live', '{"videoId":"SkdGPWUUkEw"}',   'city',    'Amsterdam canal loop',                 NULL, 1920, 1080, 'h264', 4, 180, -10, 80, 50, NOW());

-- DEMO loop MP4 cameras (5). URLs point to assets we upload to R2/local
-- static hosting; for the alpha we ship them under /static/demo/.
INSERT INTO cameras (
    name, slug, camera_type,
    country, country_code, city,
    location, timezone,
    stream_url, stream_type,
    stream_source_type, stream_source_config,
    category, description_en,
    resolution_w, resolution_h, codec,
    height_above_ground, camera_azimuth, camera_elevation, fov_horizontal, fov_vertical,
    created_at
) VALUES
    ('Demo – Tokyo Alley',   'demo-tokyo-alley',   'public','Japan','JP','Tokyo',    ST_SetSRID(ST_Point(139.7014,35.6598),4326),'Asia/Tokyo','loop://tokyo-alley.mp4','hls','loop_mp4','{"asset":"tokyo-alley.mp4","duration_s":92}','city','Looped demo – Tokyo alley',1280,720,'h264',6,120,-15,80,50,NOW()),
    ('Demo – Cafe Street',   'demo-cafe-street',   'public','France','FR','Paris',   ST_SetSRID(ST_Point(2.3500, 48.8566),4326),'Europe/Paris','loop://cafe-street.mp4','hls','loop_mp4','{"asset":"cafe-street.mp4","duration_s":120}','city','Looped demo – Parisian cafe street',1280,720,'h264',4,240,-8,80,50,NOW()),
    ('Demo – Beach Sunset',  'demo-beach-sunset',  'public','Spain','ES','Valencia', ST_SetSRID(ST_Point(-0.3700,39.4700),4326),'Europe/Madrid','loop://beach-sunset.mp4','hls','loop_mp4','{"asset":"beach-sunset.mp4","duration_s":180}','nature','Looped demo – sunset at the beach',1280,720,'h264',8,270,-5,90,55,NOW()),
    ('Demo – Rainy Window',  'demo-rainy-window',  'public','Russia','RU','Saint Petersburg', ST_SetSRID(ST_Point(30.3351,59.9343),4326),'Europe/Moscow','loop://rainy-window.mp4','hls','loop_mp4','{"asset":"rainy-window.mp4","duration_s":240}','nature','Looped demo – rainy window',1280,720,'h264',3,0,-20,60,40,NOW()),
    ('Demo – Mountain Pass', 'demo-mountain-pass', 'public','Switzerland','CH','Zermatt', ST_SetSRID(ST_Point(7.7491,46.0207),4326),'Europe/Zurich','loop://mountain-pass.mp4','hls','loop_mp4','{"asset":"mountain-pass.mp4","duration_s":150}','nature','Looped demo – alpine pass',1280,720,'h264',12,45,-10,85,50,NOW());
```

**Предупреждение:** YouTube-ID выше – **кандидаты, их нужно проверить** на этапе Task 7. Если канал сменил ID или стрим закончился – заменить в отдельной миграции (`011_fix_cameras.sql`).

- [ ] **Step 2: Прогнать**

```bash
cargo sqlx migrate run
psql -d placebo_dev -c "SELECT slug, stream_source_type FROM cameras WHERE slug LIKE 'yt-%' OR slug LIKE 'demo-%';"
```

Expected: 18 строк.

- [ ] **Step 3: Commit**

```bash
git add crates/placebo-api/migrations/010_seed_alpha_cameras.sql
git commit -m "feat(db): 010 alpha seed – 13 youtube live + 5 loop demos"
```

---

## Task 4: DTO в placebo-shared

**Files:** `crates/placebo-shared/src/camera.rs`

- [ ] **Step 1: Типы**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[cfg(feature = "export-types")]
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "snake_case")]
pub enum StreamSourceType {
    YoutubeLive,
    DirectHls,
    LoopMp4,
    Rtsp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct CameraSummary {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub city: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub category: String,
    pub thumbnail_url: Option<String>,
    pub has_audio: bool,
    pub has_night_vision: bool,
    pub stream_source_type: StreamSourceType,
    /// Relative path to the proxied manifest (e.g. "/api/v1/hls-proxy/yt-shibuya-crossing")
    pub stream_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct CameraDetail {
    #[serde(flatten)]
    pub summary: CameraSummary,
    pub description_en: Option<String>,
    pub tags: Vec<String>,
    pub timezone: Option<String>,
    pub height_above_ground: Option<f32>,
    pub camera_azimuth: Option<f32>,
    pub camera_elevation: Option<f32>,
    pub fov_horizontal: Option<f32>,
    pub fov_vertical: Option<f32>,
    pub added_to_placebo_at: Option<DateTime<Utc>>,
    pub attribution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct CameraListQuery {
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub category: Option<String>,
    pub q: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct CameraListResponse {
    pub items: Vec<CameraSummary>,
    pub total: u64,
}
```

- [ ] **Step 2: Export types**

```bash
npm run gen-types
ls src/types/api/ | grep -i camera
```

Expected: `CameraSummary.ts`, `CameraDetail.ts`, `StreamSourceType.ts`, `CameraListQuery.ts`, `CameraListResponse.ts`.

- [ ] **Step 3: Commit**

```bash
git add crates/placebo-shared/src/camera.rs src/types/api/
git commit -m "feat(shared): Camera DTOs with StreamSourceType (ts-rs)"
```

---

## Task 5: camera_repo + camera_service

**Files:**
- Modify: `crates/placebo-api/src/repositories/camera_repo.rs`
- Modify: `crates/placebo-api/src/services/camera_service.rs`

- [ ] **Step 1: Row struct**

```rust
// camera_repo.rs
#[derive(sqlx::FromRow)]
pub struct CameraRow {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub city: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub category: String,
    pub thumbnail_url: Option<String>,
    pub has_audio: bool,
    pub has_night_vision: bool,
    pub stream_source_type: String, // enum → fetched as text
    pub description_en: Option<String>,
    pub tags: serde_json::Value,
    pub timezone: Option<String>,
    pub height_above_ground: Option<f32>,
    pub camera_azimuth: Option<f32>,
    pub camera_elevation: Option<f32>,
    pub fov_horizontal: Option<f32>,
    pub fov_vertical: Option<f32>,
    pub added_to_placebo_at: Option<chrono::DateTime<chrono::Utc>>,
    pub attribution: Option<String>,
}

const SELECT_FIELDS: &str = "
    id, slug, name, city, country, country_code,
    ST_Y(location) AS lat, ST_X(location) AS lng,
    category, thumbnail_url, has_audio, has_night_vision,
    stream_source_type::text AS stream_source_type,
    description_en, tags, timezone,
    height_above_ground, camera_azimuth, camera_elevation,
    fov_horizontal, fov_vertical,
    added_to_placebo_at, attribution
";

pub async fn list(
    pool: &PgPool,
    q: &placebo_shared::camera::CameraListQuery,
) -> sqlx::Result<(Vec<CameraRow>, u64)> {
    let limit = q.limit.unwrap_or(50).min(200) as i64;
    let offset = q.offset.unwrap_or(0) as i64;

    // Build WHERE clauses dynamically; use parameters to prevent SQL injection.
    let mut where_clauses: Vec<String> = vec!["stream_source_type IS NOT NULL".into()];
    // Only surface alpha roster by default (yt-* or demo-*)
    where_clauses.push("(slug LIKE 'yt-%' OR slug LIKE 'demo-%')".into());

    let mut args: Vec<String> = Vec::new();
    if let Some(city) = &q.city { where_clauses.push(format!("lower(city) = lower(${})", args.len() + 1)); args.push(city.clone()); }
    if let Some(cc)   = &q.country_code { where_clauses.push(format!("country_code = upper(${})", args.len() + 1)); args.push(cc.clone()); }
    if let Some(cat)  = &q.category { where_clauses.push(format!("category = ${}", args.len() + 1)); args.push(cat.clone()); }
    if let Some(text) = &q.q { where_clauses.push(format!("(name ILIKE '%' || ${} || '%' OR city ILIKE '%' || ${} || '%')", args.len() + 1, args.len() + 1)); args.push(text.clone()); }

    let where_sql = where_clauses.join(" AND ");

    let sql = format!(
        "SELECT {SELECT_FIELDS} FROM cameras WHERE {where_sql} ORDER BY name LIMIT ${} OFFSET ${}",
        args.len() + 1, args.len() + 2
    );

    let mut query = sqlx::query_as::<_, CameraRow>(&sql);
    for a in &args { query = query.bind(a); }
    query = query.bind(limit).bind(offset);
    let rows = query.fetch_all(pool).await?;

    let count_sql = format!("SELECT count(*)::bigint FROM cameras WHERE {where_sql}");
    let mut cq = sqlx::query_scalar::<_, i64>(&count_sql);
    for a in &args { cq = cq.bind(a); }
    let total = cq.fetch_one(pool).await? as u64;

    Ok((rows, total))
}

pub async fn by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<CameraRow>> {
    let sql = format!("SELECT {SELECT_FIELDS} FROM cameras WHERE id = $1");
    sqlx::query_as::<_, CameraRow>(&sql).bind(id).fetch_optional(pool).await
}

pub async fn by_slug(pool: &PgPool, slug: &str) -> sqlx::Result<Option<CameraRow>> {
    let sql = format!("SELECT {SELECT_FIELDS} FROM cameras WHERE slug = $1");
    sqlx::query_as::<_, CameraRow>(&sql).bind(slug).fetch_optional(pool).await
}

/// Raw stream source for the proxy layer. Not exposed to clients.
pub async fn stream_source_for_slug(
    pool: &PgPool,
    slug: &str,
) -> sqlx::Result<Option<(String, serde_json::Value)>> {
    sqlx::query_as::<_, (String, serde_json::Value)>(
        "SELECT stream_source_type::text, stream_source_config FROM cameras WHERE slug = $1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
}
```

- [ ] **Step 2: Сервис-конвертер Row → DTO**

В `camera_service.rs`:

```rust
use placebo_shared::camera::{
    CameraDetail, CameraListQuery, CameraListResponse, CameraSummary, StreamSourceType,
};

fn parse_source_type(s: &str) -> StreamSourceType {
    match s {
        "youtube_live" => StreamSourceType::YoutubeLive,
        "direct_hls"   => StreamSourceType::DirectHls,
        "loop_mp4"     => StreamSourceType::LoopMp4,
        _              => StreamSourceType::Rtsp,
    }
}

fn row_to_summary(r: camera_repo::CameraRow) -> CameraSummary {
    CameraSummary {
        id: r.id,
        slug: r.slug.clone(),
        name: r.name,
        city: r.city,
        country: r.country,
        country_code: r.country_code,
        lat: r.lat,
        lng: r.lng,
        category: r.category,
        thumbnail_url: r.thumbnail_url,
        has_audio: r.has_audio,
        has_night_vision: r.has_night_vision,
        stream_source_type: parse_source_type(&r.stream_source_type),
        stream_url: format!("/api/v1/hls-proxy/{}", r.slug),
    }
}

pub async fn list(
    pool: &PgPool,
    q: CameraListQuery,
) -> Result<CameraListResponse, AppError> {
    let (rows, total) = camera_repo::list(pool, &q).await?;
    Ok(CameraListResponse {
        items: rows.into_iter().map(row_to_summary).collect(),
        total,
    })
}

pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<CameraDetail>, AppError> {
    let row = match camera_repo::by_id(pool, id).await? {
        Some(r) => r,
        None => return Ok(None),
    };
    let tags: Vec<String> = serde_json::from_value(row.tags.clone()).unwrap_or_default();
    let summary = row_to_summary(row.clone());
    Ok(Some(CameraDetail {
        summary,
        description_en: row.description_en,
        tags,
        timezone: row.timezone,
        height_above_ground: row.height_above_ground,
        camera_azimuth: row.camera_azimuth,
        camera_elevation: row.camera_elevation,
        fov_horizontal: row.fov_horizontal,
        fov_vertical: row.fov_vertical,
        added_to_placebo_at: row.added_to_placebo_at,
        attribution: row.attribution,
    }))
}
```

**Примечание**: `CameraRow` должен быть `Clone`. Добавить `#[derive(Clone, sqlx::FromRow)]`.

- [ ] **Step 3: Unit-тест парсинга enum**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn parses_known_variants() {
        assert_eq!(parse_source_type("youtube_live"), StreamSourceType::YoutubeLive);
        assert_eq!(parse_source_type("direct_hls"), StreamSourceType::DirectHls);
        assert_eq!(parse_source_type("loop_mp4"), StreamSourceType::LoopMp4);
        assert_eq!(parse_source_type("weird"), StreamSourceType::Rtsp);
    }
}
```

- [ ] **Step 4: cargo test + commit**

```bash
cargo test -p placebo-api --lib services::camera_service
git add crates/placebo-api/src/
git commit -m "feat(cameras): repo+service return CameraSummary/Detail DTOs"
```

---

## Task 6: REST handlers /api/v1/cameras

**Files:** `crates/placebo-api/src/handlers/cameras.rs`

- [ ] **Step 1: Router**

```rust
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use placebo_shared::camera::{CameraDetail, CameraListQuery, CameraListResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::services::camera_service;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/cameras", get(list))
        .route("/cameras/:id", get(get_one))
}

async fn list(
    State(state): State<AppState>,
    Query(q): Query<CameraListQuery>,
) -> Result<Json<CameraListResponse>, AppError> {
    Ok(Json(camera_service::list(&state.db, q).await?))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CameraDetail>, AppError> {
    let found = camera_service::get_by_id(&state.db, id).await?
        .ok_or(AppError::NotFound("camera".into()))?;
    Ok(Json(found))
}
```

- [ ] **Step 2: Смонтировать**

Найти место сборки router и убедиться, что `.merge(handlers::cameras::router())` в составе `/api/v1`.

- [ ] **Step 3: Smoke**

```bash
cargo run -p placebo-api &
sleep 3
curl -s http://localhost:3001/api/v1/cameras | jq '.total'
curl -s http://localhost:3001/api/v1/cameras?city=Tokyo | jq '.items[0].slug'
kill %1
```

Expected: total >= 18, slug для Tokyo начинается с `yt-shibuya-crossing` или `demo-tokyo-alley`.

- [ ] **Step 4: Commit**

```bash
git add crates/placebo-api/src/handlers/cameras.rs
git commit -m "feat(api): GET /cameras and /cameras/:id with pagination"
```

---

## Task 7: HLS-прокси на axum

**Files:**
- Create: `crates/placebo-api/src/services/hls_source.rs` – резолвер
- Create: `crates/placebo-api/src/handlers/hls_proxy.rs` – handler

### Поведение

- `GET /api/v1/hls-proxy/{slug}` – возвращает m3u8, переписанный так, что сегменты идут через наш же прокси.
- `GET /api/v1/hls-proxy/{slug}/seg?u=<base64url-of-absolute-url>` – стримит сегмент upstream.
- Для `youtube_live`: резолвим upstream через yt-dlp, кешируем в Redis 30 мин.
- Для `direct_hls`: конфиг содержит `{"url": "..."}` – используем напрямую.
- Для `loop_mp4`: возвращаем синтетический HLS из сегментов, заранее подготовленных (M3 в альфе просто отдаёт один .m3u8 из `/static/demo/<asset>/index.m3u8`; сами ассеты создаются вручную FFmpeg-ом, инструкции ниже).
- Для `rtsp`: **не поддерживаем в альфе** → 404.

- [ ] **Step 1: Service**

```rust
// src/services/hls_source.rs
use anyhow::{anyhow, Result};
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::Pool as RedisPool;
use serde_json::Value;
use sqlx::PgPool;
use tokio::process::Command;

use crate::repositories::camera_repo;

const CACHE_TTL_SECS: usize = 30 * 60;

pub enum ResolvedSource {
    /// Absolute m3u8 URL we should fetch and rewrite.
    Hls(String),
    /// Static m3u8 path served by our own static handler.
    StaticLoop(String),
}

pub async fn resolve(
    pg: &PgPool,
    redis: &RedisPool,
    slug: &str,
) -> Result<ResolvedSource> {
    let (kind, cfg) = camera_repo::stream_source_for_slug(pg, slug)
        .await?
        .ok_or_else(|| anyhow!("unknown slug"))?;

    match kind.as_str() {
        "youtube_live" => {
            let video_id = cfg.get("videoId").and_then(Value::as_str).ok_or_else(|| anyhow!("no videoId"))?;
            let cache_key = format!("hls:yt:{slug}");
            let mut conn = redis.get().await?;
            if let Some(cached) = conn.get::<_, Option<String>>(&cache_key).await? {
                return Ok(ResolvedSource::Hls(cached));
            }
            let url = resolve_youtube(video_id).await?;
            let _: () = conn.set_ex(cache_key, &url, CACHE_TTL_SECS).await?;
            Ok(ResolvedSource::Hls(url))
        }
        "direct_hls" => {
            let url = cfg.get("url").and_then(Value::as_str).ok_or_else(|| anyhow!("no url"))?;
            Ok(ResolvedSource::Hls(url.to_string()))
        }
        "loop_mp4" => {
            let asset = cfg.get("asset").and_then(Value::as_str).ok_or_else(|| anyhow!("no asset"))?;
            Ok(ResolvedSource::StaticLoop(format!("/static/demo/{asset}/index.m3u8")))
        }
        other => Err(anyhow!("unsupported stream_source_type: {other}")),
    }
}

async fn resolve_youtube(video_id: &str) -> Result<String> {
    let output = Command::new("yt-dlp")
        .args([
            "-f", "best[vcodec^=avc1]",
            "--no-warnings",
            "-g",
            &format!("https://www.youtube.com/watch?v={video_id}"),
        ])
        .output()
        .await?;
    if !output.status.success() {
        return Err(anyhow!("yt-dlp exit {:?}", output.status.code()));
    }
    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() { return Err(anyhow!("yt-dlp empty output")); }
    Ok(url)
}
```

- [ ] **Step 2: Handler**

```rust
// src/handlers/hls_proxy.rs
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::Url;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::services::hls_source::{self, ResolvedSource};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/hls-proxy/:slug", get(manifest))
        .route("/hls-proxy/:slug/seg", get(segment))
}

#[derive(Deserialize)]
struct SegQuery { u: String }

async fn manifest(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Response, AppError> {
    let source = hls_source::resolve(&state.db, &state.redis, &slug)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    match source {
        ResolvedSource::StaticLoop(rel_path) => Ok((
            StatusCode::FOUND,
            [("location", rel_path.as_str())],
        ).into_response()),
        ResolvedSource::Hls(upstream_url) => {
            let client = reqwest::Client::new();
            let body = client.get(&upstream_url).send().await
                .map_err(|e| AppError::Internal(format!("upstream: {e}")))?
                .text().await
                .map_err(|e| AppError::Internal(format!("upstream body: {e}")))?;

            let base = Url::parse(&upstream_url).map_err(|e| AppError::Internal(e.to_string()))?;
            let rewritten = rewrite_m3u8(&body, &base, &slug);

            Ok((
                [("content-type", "application/vnd.apple.mpegurl"), ("cache-control", "no-cache")],
                rewritten,
            ).into_response())
        }
    }
}

fn rewrite_m3u8(body: &str, base: &Url, slug: &str) -> String {
    let mut out = String::with_capacity(body.len() + 256);
    for line in body.lines() {
        if line.starts_with('#') || line.is_empty() {
            out.push_str(line); out.push('\n'); continue;
        }
        let abs = if line.starts_with("http://") || line.starts_with("https://") {
            line.to_string()
        } else {
            base.join(line).map(|u| u.to_string()).unwrap_or_else(|_| line.to_string())
        };
        let encoded = URL_SAFE_NO_PAD.encode(abs.as_bytes());
        out.push_str(&format!("/api/v1/hls-proxy/{slug}/seg?u={encoded}"));
        out.push('\n');
    }
    out
}

async fn segment(
    Path(_slug): Path<String>,
    Query(q): Query<SegQuery>,
) -> Result<Response, AppError> {
    let decoded = URL_SAFE_NO_PAD.decode(q.u.as_bytes())
        .map_err(|_| AppError::BadRequest("invalid u".into()))?;
    let url = String::from_utf8(decoded)
        .map_err(|_| AppError::BadRequest("invalid u utf8".into()))?;

    let client = reqwest::Client::new();
    let upstream = client.get(&url).send().await
        .map_err(|e| AppError::Internal(format!("upstream: {e}")))?;

    let status = upstream.status();
    let mut headers = HeaderMap::new();
    if let Some(ct) = upstream.headers().get("content-type") { headers.insert("content-type", ct.clone()); }
    headers.insert("access-control-allow-origin", "*".parse().unwrap());
    headers.insert("cache-control", "no-cache".parse().unwrap());

    let stream = upstream.bytes_stream();
    let body = Body::from_stream(stream);
    Ok((status, headers, body).into_response())
}
```

- [ ] **Step 3: Зависимости**

В `crates/placebo-api/Cargo.toml`:

```toml
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json", "stream"] }
base64 = "0.22"
```

- [ ] **Step 4: Маунт**

```rust
.merge(handlers::hls_proxy::router())
```

- [ ] **Step 5: Static-hosting для loop_mp4 (отдельный шаг)**

Для альфы папка `static/demo/<asset>/index.m3u8` + сегменты. Добавить `tower_http::services::ServeDir` на `/static`:

```rust
use tower_http::services::ServeDir;
// при сборке Router:
.nest_service("/static", ServeDir::new("./static"))
```

Как сгенерировать loop HLS из MP4 (ручная операция для 5 demo-ассетов):

```bash
mkdir -p crates/placebo-api/static/demo/tokyo-alley
ffmpeg -i source.mp4 -c:v libx264 -preset veryfast -crf 23 \
  -g 48 -hls_time 4 -hls_playlist_type vod \
  crates/placebo-api/static/demo/tokyo-alley/index.m3u8
```

В альфе: 5 ассетов, каждый ~90-240 секунд. Суммарный размер < 500 MB.

- [ ] **Step 6: cargo check + test + коммит**

```bash
cargo check -p placebo-api
git add crates/placebo-api/src/handlers/hls_proxy.rs crates/placebo-api/src/services/hls_source.rs crates/placebo-api/Cargo.toml
git commit -m "feat(api): /api/v1/hls-proxy manifest+segment with Redis URL cache"
```

---

## Task 8: Валидация YouTube ID кандидатов

- [ ] **Step 1: Скрипт проверки**

Создать `scripts/verify-youtube-seed.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
IDS=(dfVK7ld38Ys u4UZ4UvZXrg FQWkgr0aHlI AdUw5RdyZxI h1wly909BYw 7Bl5p4VTXzQ 2L4yhCmGRWg dyWHmEQAVUI qMksIqJv3pI wNmMr_ATI2E hSbkw-F7bzY 2Te5EvOXNZw SkdGPWUUkEw)
for id in "${IDS[@]}"; do
  echo -n "$id ... "
  if yt-dlp -f 'best[vcodec^=avc1]' --no-warnings -g "https://www.youtube.com/watch?v=$id" >/dev/null 2>&1; then
    echo OK
  else
    echo FAIL
  fi
done
```

- [ ] **Step 2: Прогон**

```bash
chmod +x scripts/verify-youtube-seed.sh
./scripts/verify-youtube-seed.sh
```

Expected: каждая строка OK. Если несколько FAIL – заменить ID в миграции `011_fix_cameras.sql` и повторить.

- [ ] **Step 3: Commit**

```bash
git add scripts/verify-youtube-seed.sh
git commit -m "build: script to verify alpha YouTube seed ids resolve via yt-dlp"
```

---

## Task 9: Frontend – убрать vite middleware, подключить API

- [ ] **Step 1: vite.config.ts**

Удалить блок `{ name: 'hls-proxy', configureServer(...) {...} }`, а также `YOUTUBE_IDS`, `resolveYoutubeHls`, `rewriteM3u8`, `proxyFetch`, `urlCache`. Импорты `execSync`, `httpsGet`, `httpGet`, типы `IncomingMessage`/`ServerResponse` – тоже удалить.

Оставить только базовый vite config с `react()`.

- [ ] **Step 2: Клиент API камер**

`src/api/cameras.ts`:

```ts
import { apiRequest } from "./client";
import type { CameraListResponse } from "../types/api/CameraListResponse";
import type { CameraDetail } from "../types/api/CameraDetail";
import type { CameraListQuery } from "../types/api/CameraListQuery";

export async function listCameras(q: CameraListQuery = {}): Promise<CameraListResponse> {
  const params = new URLSearchParams();
  for (const [k, v] of Object.entries(q)) if (v != null) params.set(k, String(v));
  const qs = params.toString();
  return apiRequest<CameraListResponse>(`/cameras${qs ? `?${qs}` : ""}`, { auth: false });
}

export async function getCamera(id: string): Promise<CameraDetail> {
  return apiRequest<CameraDetail>(`/cameras/${id}`, { auth: false });
}
```

- [ ] **Step 3: Переключить useNearbyCameras**

Заменить mock в `src/hooks/useNearbyCameras.ts` на реальный запрос. Minimum change: если файл построен на моках – добавить новый хук `useCamerasFromApi()` и пометить старый `@deprecated`. Реальный рефакторинг World3D – в M4; в M3 мы просто добавляем источник данных.

```ts
// src/hooks/useCamerasFromApi.ts
import { useEffect, useState } from "react";
import { listCameras } from "../api/cameras";
import type { CameraSummary } from "../types/api/CameraSummary";

export function useCamerasFromApi() {
  const [data, setData] = useState<CameraSummary[] | null>(null);
  const [error, setError] = useState<Error | null>(null);
  useEffect(() => {
    let cancelled = false;
    listCameras({ limit: 50 })
      .then((res) => { if (!cancelled) setData(res.items); })
      .catch((e) => { if (!cancelled) setError(e); });
    return () => { cancelled = true; };
  }, []);
  return { data, error, loading: !data && !error };
}
```

- [ ] **Step 4: Убрать `VITE_GO2RTC_URL` из .env если есть**

```bash
grep -rn "VITE_GO2RTC_URL" d:/Projects/Placebo/ 2>/dev/null | grep -v node_modules | grep -v target
```

Eсли есть – удалить.

- [ ] **Step 5: npm run build**

```bash
cd d:/Projects/Placebo
npm run build
```

Expected: build проходит.

- [ ] **Step 6: Commit**

```bash
git add vite.config.ts src/api/cameras.ts src/hooks/useCamerasFromApi.ts
git commit -m "$(cat <<'EOF'
feat(frontend): drop Vite hls-proxy, use /api/v1/hls-proxy instead

- vite.config.ts stripped to a plain react() setup.
- src/api/cameras.ts wraps /cameras + /cameras/:id.
- useCamerasFromApi() feeds real CameraSummary items from the server.
- M4 rewires World3DScreen to consume this hook; useNearbyCameras mock
  stays in place until then to avoid scope creep.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: End-to-end verification

- [ ] **Step 1: Бэкенд**

```bash
cd d:/Projects/Placebo/crates/placebo-api
cargo run
```

- [ ] **Step 2: Проверка эндпоинтов**

```bash
curl -s http://localhost:3001/api/v1/cameras | jq '.total'
curl -s http://localhost:3001/api/v1/cameras?city=Tokyo | jq '.items[].slug'

# Получить manifest
curl -v http://localhost:3001/api/v1/hls-proxy/yt-shibuya-crossing
# Ожидаем 200 + content-type application/vnd.apple.mpegurl
# Первая non-# строка должна начинаться с /api/v1/hls-proxy/yt-shibuya-crossing/seg?u=...
```

- [ ] **Step 3: Проверка video-воспроизведения в браузере**

С Tauri dev запускать долго: используем отдельный минимальный HTML для изоляции:

```bash
cat > /tmp/hls-test.html <<'EOF'
<!DOCTYPE html><html><body>
<video id="v" controls style="width:100%"></video>
<script src="https://cdn.jsdelivr.net/npm/hls.js@1"></script>
<script>
  const hls = new Hls();
  hls.loadSource('http://localhost:3001/api/v1/hls-proxy/yt-shibuya-crossing');
  hls.attachMedia(document.getElementById('v'));
</script></body></html>
EOF
# Открыть в Chrome
start /tmp/hls-test.html
```

Expected: видео начинает играть через 3-8 секунд.

- [ ] **Step 4: Commit + PR**

```bash
cd d:/Projects/Placebo
git push -u origin feat/m3-cameras-hls
```

PR: `feat/m3-cameras-hls → main`.

---

## Acceptance Criteria для Milestone 3

1. ✅ Миграции 009 и 010 применяются на чистой базе без ошибок.
2. ✅ `GET /api/v1/cameras` возвращает ≥ 18 CameraSummary, без `streamUrl` указывающего на RTSP или внешний HLS.
3. ✅ `GET /api/v1/cameras/:id` возвращает CameraDetail с тегами и 3D-полями.
4. ✅ `GET /api/v1/hls-proxy/:slug` для `yt-*` slug отдаёт m3u8 с сегмент-URL, указывающими на наш же `/seg`.
5. ✅ `GET /api/v1/hls-proxy/:slug/seg?u=...` стримит сегмент и возвращает 200.
6. ✅ Redis содержит `hls:yt:<slug>` TTL 30 min после первого запроса.
7. ✅ Для `loop_mp4` прокси возвращает 302 на `/static/demo/<asset>/index.m3u8`.
8. ✅ Для `rtsp` возвращается ошибка (в альфе не поддерживаем).
9. ✅ `vite.config.ts` больше не содержит hls-proxy middleware и yt-dlp кода.
10. ✅ `scripts/verify-youtube-seed.sh` – все 13 ID резолвятся в yt-dlp.
11. ✅ ts-rs генерирует CameraSummary/Detail/StreamSourceType/CameraListQuery/CameraListResponse.
12. ✅ cargo test всего workspace зелёный; npm run build зелёный.

---

## Что идёт дальше

После approval M3 – `2026-05-14-milestone-4-home-categories-world.md`: переписать Home, Categories, встроить World3DScreen в shell, переключить 3D-мир на реальные данные из API.
