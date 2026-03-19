# OSM Tile Pipeline Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Serve real OSM geometry (roads, water, parks, buildings) from PostGIS through an axum tile API to the React Three Fiber 3D world.

**Architecture:** PostGIS SQL views → Rust repo (raw queries) → service (Redis cache + coordinate conversion) → handler (validation + JSON response) → useCityTiles hook (tile grid fetch) → R3F components (geometry rendering). Server does all heavy lifting; client receives ready-to-render local-meter coordinates.

**Tech Stack:** PostgreSQL 17 + PostGIS 3.6.2, axum 0.7, sqlx 0.8, deadpool-redis, React 18, React Three Fiber, Three.js

**Spec:** `docs/superpowers/specs/2026-03-16-osm-tile-pipeline-design.md`

---

## Chunk 1: SQL Views + Indexes (Database Layer)

### Task 1: Create SQL views and indexes

**Files:**
- Create: `pipeline/sql/03_roads_view.sql`
- Create: `pipeline/sql/04_water_view.sql`
- Create: `pipeline/sql/05_parks_view.sql`
- Create: `pipeline/sql/06_buildings_tile_view.sql`
- Create: `pipeline/sql/07_indexes.sql`

- [ ] **Step 1: Create roads_view**

Write `pipeline/sql/03_roads_view.sql`:
```sql
CREATE OR REPLACE VIEW roads_view AS
SELECT
  osm_id,
  highway,
  name,
  way,
  CASE highway
    WHEN 'motorway' THEN 15
    WHEN 'trunk' THEN 14
    WHEN 'primary' THEN 12
    WHEN 'secondary' THEN 9
    WHEN 'tertiary' THEN 7
    WHEN 'residential' THEN 6
    WHEN 'unclassified' THEN 5
    WHEN 'service' THEN 4
    WHEN 'footway' THEN 2
    WHEN 'cycleway' THEN 2
    WHEN 'steps' THEN 1.5
    WHEN 'pedestrian' THEN 3
    WHEN 'path' THEN 1.5
    ELSE 5
  END AS width_meters
FROM planet_osm_line
WHERE highway IS NOT NULL
  AND highway NOT IN ('proposed', 'construction', 'abandoned', 'platform');
```

- [ ] **Step 2: Create water_view**

Write `pipeline/sql/04_water_view.sql`:
```sql
CREATE OR REPLACE VIEW water_view AS
SELECT osm_id, 'polygon' AS geom_type,
       COALESCE(water, "natural") AS water_type, name, way
FROM planet_osm_polygon
WHERE water IS NOT NULL
   OR "natural" IN ('water', 'wetland')
UNION ALL
SELECT osm_id, 'line' AS geom_type,
       waterway AS water_type, name, way
FROM planet_osm_line
WHERE waterway IN ('river', 'stream', 'canal', 'drain');
```

- [ ] **Step 3: Create parks_view**

Write `pipeline/sql/05_parks_view.sql`:
```sql
CREATE OR REPLACE VIEW parks_view AS
SELECT osm_id,
       COALESCE(leisure, "natural", landuse) AS park_type,
       name, way
FROM planet_osm_polygon
WHERE leisure IN ('park', 'garden', 'playground', 'pitch')
   OR "natural" IN ('wood', 'scrub', 'grassland')
   OR landuse IN ('grass', 'forest', 'recreation_ground', 'cemetery', 'meadow');
```

- [ ] **Step 4: Create buildings_tile_view**

Write `pipeline/sql/06_buildings_tile_view.sql`:
```sql
CREATE OR REPLACE VIEW buildings_tile_view AS
SELECT
  osm_id,
  way,
  CASE
    WHEN height ~ '^\d+(\.\d+)?$' THEN height::float
    WHEN "building:levels" ~ '^\d+$' THEN "building:levels"::int * 3.0
    ELSE 9.0
  END AS height_meters,
  name
FROM planet_osm_polygon
WHERE building IS NOT NULL
  AND building NOT IN ('no', 'entrance')
  AND ST_Area(way::geography) > 10;
```

- [ ] **Step 5: Create partial indexes**

Write `pipeline/sql/07_indexes.sql`:
```sql
CREATE INDEX IF NOT EXISTS idx_line_highway
  ON planet_osm_line (highway) WHERE highway IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_polygon_building
  ON planet_osm_polygon (building) WHERE building IS NOT NULL AND building != 'no';
CREATE INDEX IF NOT EXISTS idx_polygon_water
  ON planet_osm_polygon (water) WHERE water IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_polygon_leisure
  ON planet_osm_polygon (leisure) WHERE leisure IS NOT NULL;
```

- [ ] **Step 6: Run all SQL against PostGIS and verify**

```bash
for f in pipeline/sql/03_roads_view.sql pipeline/sql/04_water_view.sql pipeline/sql/05_parks_view.sql pipeline/sql/06_buildings_tile_view.sql pipeline/sql/07_indexes.sql; do
  psql -d placebo -f "$f"
done
psql -d placebo -c "SELECT 'roads' AS view, count(*) FROM roads_view UNION ALL SELECT 'water', count(*) FROM water_view UNION ALL SELECT 'parks', count(*) FROM parks_view UNION ALL SELECT 'buildings', count(*) FROM buildings_tile_view;"
```
Expected: roads ~200K+, water ~5K+, parks ~10K+, buildings ~500K+

- [ ] **Step 7: Verify bbox query performance (Shibuya)**

```bash
psql -d placebo -c "EXPLAIN ANALYZE SELECT count(*) FROM roads_view WHERE way && ST_MakeEnvelope(139.695, 35.658, 139.710, 35.668, 4326);"
```
Expected: <50ms, index scan on `planet_osm_line_way_idx`

- [ ] **Step 8: Commit**

```bash
git add pipeline/sql/03_roads_view.sql pipeline/sql/04_water_view.sql pipeline/sql/05_parks_view.sql pipeline/sql/06_buildings_tile_view.sql pipeline/sql/07_indexes.sql
git commit -m "feat(sql): add roads, water, parks, buildings views + indexes for tile API"
```

---

## Chunk 2: Rust Backend (Repository + Service + Handler)

### Task 2: Rust repository – types and PostGIS queries

**Files:**
- Create: `crates/placebo-api/src/repositories/world_repo.rs`
- Modify: `crates/placebo-api/src/repositories/mod.rs` (add `pub mod world_repo;`)

- [ ] **Step 1: Add `pub mod world_repo;` to `repositories/mod.rs`**

In `crates/placebo-api/src/repositories/mod.rs`, add line:
```rust
pub mod world_repo;
```

- [ ] **Step 2: Write world_repo.rs with types and helper functions**

Create `crates/placebo-api/src/repositories/world_repo.rs`:

```rust
use sqlx::PgPool;

// ─── Row types (from PostGIS) ───────────────────────────────

#[derive(Debug, sqlx::FromRow)]
pub struct RoadRow {
    pub osm_id: i64,
    pub highway: String,
    pub name: Option<String>,
    pub width_meters: f64,
    pub geojson: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct WaterRow {
    pub osm_id: i64,
    pub geom_type: String,
    pub water_type: String,
    pub name: Option<String>,
    pub geojson: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ParkRow {
    pub osm_id: i64,
    pub park_type: String,
    pub name: Option<String>,
    pub geojson: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct BuildingRow {
    pub osm_id: i64,
    pub height_meters: f64,
    pub name: Option<String>,
    pub geojson: String,
}

// ─── Tile math ──────────────────────────────────────────────

/// Convert Slippy Map tile z/x/y to WGS84 bounding box
/// Returns (west_lng, south_lat, east_lng, north_lat)
pub fn tile_to_bbox(z: u8, x: u32, y: u32) -> (f64, f64, f64, f64) {
    let n = 2f64.powi(z as i32);
    let west = x as f64 / n * 360.0 - 180.0;
    let east = (x + 1) as f64 / n * 360.0 - 180.0;
    let north = (std::f64::consts::PI * (1.0 - 2.0 * y as f64 / n))
        .sinh()
        .atan()
        .to_degrees();
    let south = (std::f64::consts::PI * (1.0 - 2.0 * (y + 1) as f64 / n))
        .sinh()
        .atan()
        .to_degrees();
    (west, south, east, north)
}

pub fn zoom_tolerance(z: u8) -> f64 {
    match z {
        17 => 0.00001,
        16 => 0.00005,
        15 => 0.0001,
        _ => 0.00005,
    }
}

// ─── Queries ────────────────────────────────────────────────

pub async fn get_roads(
    pool: &PgPool,
    west: f64, south: f64, east: f64, north: f64,
    tolerance: f64,
) -> Result<Vec<RoadRow>, sqlx::Error> {
    sqlx::query_as::<_, RoadRow>(
        r#"SELECT osm_id, highway, name, width_meters,
                  ST_AsGeoJSON(ST_SimplifyPreserveTopology(way, $5)) AS geojson
           FROM roads_view
           WHERE way && ST_MakeEnvelope($1, $2, $3, $4, 4326)"#,
    )
    .bind(west).bind(south).bind(east).bind(north).bind(tolerance)
    .fetch_all(pool)
    .await
}

pub async fn get_water(
    pool: &PgPool,
    west: f64, south: f64, east: f64, north: f64,
    tolerance: f64,
) -> Result<Vec<WaterRow>, sqlx::Error> {
    sqlx::query_as::<_, WaterRow>(
        r#"SELECT osm_id, geom_type, water_type, name,
                  ST_AsGeoJSON(ST_SimplifyPreserveTopology(way, $5)) AS geojson
           FROM water_view
           WHERE way && ST_MakeEnvelope($1, $2, $3, $4, 4326)"#,
    )
    .bind(west).bind(south).bind(east).bind(north).bind(tolerance)
    .fetch_all(pool)
    .await
}

pub async fn get_parks(
    pool: &PgPool,
    west: f64, south: f64, east: f64, north: f64,
    tolerance: f64,
) -> Result<Vec<ParkRow>, sqlx::Error> {
    sqlx::query_as::<_, ParkRow>(
        r#"SELECT osm_id, park_type, name,
                  ST_AsGeoJSON(ST_SimplifyPreserveTopology(way, $5)) AS geojson
           FROM parks_view
           WHERE way && ST_MakeEnvelope($1, $2, $3, $4, 4326)"#,
    )
    .bind(west).bind(south).bind(east).bind(north).bind(tolerance)
    .fetch_all(pool)
    .await
}

pub async fn get_buildings(
    pool: &PgPool,
    west: f64, south: f64, east: f64, north: f64,
    tolerance: f64,
) -> Result<Vec<BuildingRow>, sqlx::Error> {
    sqlx::query_as::<_, BuildingRow>(
        r#"SELECT osm_id, height_meters, name,
                  ST_AsGeoJSON(ST_SimplifyPreserveTopology(way, $5)) AS geojson
           FROM buildings_tile_view
           WHERE way && ST_MakeEnvelope($1, $2, $3, $4, 4326)"#,
    )
    .bind(west).bind(south).bind(east).bind(north).bind(tolerance)
    .fetch_all(pool)
    .await
}
```

- [ ] **Step 3: Verify compilation**

```bash
cargo check -p placebo-api
```

- [ ] **Step 4: Unit test tile_to_bbox**

Add at bottom of `world_repo.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_to_bbox_shibuya() {
        let (west, south, east, north) = tile_to_bbox(16, 57483, 25953);
        // Shibuya area: ~139.69-139.71 lng, ~35.65-35.67 lat
        assert!((west - 139.691).abs() < 0.01);
        assert!((east - 139.697).abs() < 0.01);
        assert!((south - 35.659).abs() < 0.01);
        assert!((north - 35.664).abs() < 0.01);
    }

    #[test]
    fn test_zoom_tolerance() {
        assert_eq!(zoom_tolerance(17), 0.00001);
        assert_eq!(zoom_tolerance(16), 0.00005);
        assert_eq!(zoom_tolerance(15), 0.0001);
    }
}
```

Run: `cargo test -p placebo-api -- world_repo`

- [ ] **Step 5: Commit**

```bash
git add crates/placebo-api/src/repositories/world_repo.rs crates/placebo-api/src/repositories/mod.rs
git commit -m "feat(api): add world_repo with PostGIS tile queries and bbox math"
```

### Task 3: Rust service – caching, coordinate conversion, response assembly

**Files:**
- Create: `crates/placebo-api/src/services/world_service.rs`
- Modify: `crates/placebo-api/src/services/mod.rs` (add `pub mod world_service;`)

- [ ] **Step 1: Add `pub mod world_service;` to `services/mod.rs`**

- [ ] **Step 2: Write world_service.rs**

Create `crates/placebo-api/src/services/world_service.rs`:

```rust
use deadpool_redis::Pool as RedisPool;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AppError;
use crate::repositories::world_repo;

// ─── Cache types (stored in Redis as lat/lng) ───────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct TileRawData {
    pub roads: Vec<RawRoad>,
    pub water: Vec<RawWater>,
    pub parks: Vec<RawPark>,
    pub buildings: Vec<RawBuilding>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawRoad {
    pub coords: Vec<[f64; 2]>, // [lng, lat]
    pub highway: String,
    pub name: Option<String>,
    pub width: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawWater {
    pub coords: Vec<[f64; 2]>,
    pub water_type: String,
    pub geom_type: String, // "polygon" | "line"
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawPark {
    pub coords: Vec<[f64; 2]>,
    pub park_type: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawBuilding {
    pub coords: Vec<[f64; 2]>,
    pub height: f64,
}

// ─── API response types ─────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TileResponse {
    pub tile: TileInfo,
    pub roads: Vec<RoadFeature>,
    pub water: Vec<WaterFeature>,
    pub parks: Vec<ParkFeature>,
    pub buildings: Vec<BuildingFeature>,
}

#[derive(Debug, Serialize)]
pub struct TileInfo { pub z: u8, pub x: u32, pub y: u32 }

#[derive(Debug, Serialize)]
pub struct Point2D { pub x: f64, pub z: f64 }

#[derive(Debug, Serialize)]
pub struct RoadFeature {
    pub points: Vec<Point2D>,
    pub highway: String,
    pub name: Option<String>,
    pub width: f64,
}

#[derive(Debug, Serialize)]
pub struct WaterFeature {
    pub points: Vec<Point2D>,
    #[serde(rename = "type")]
    pub water_type: String,
    #[serde(rename = "geomType")]
    pub geom_type: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParkFeature {
    pub points: Vec<Point2D>,
    #[serde(rename = "type")]
    pub park_type: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BuildingFeature {
    pub outline: Vec<Point2D>,
    pub height: f64,
}

// ─── Coordinate conversion ──────────────────────────────────

pub fn geo_to_local(lat: f64, lng: f64, center_lat: f64, center_lng: f64) -> (f64, f64) {
    let x = (lng - center_lng) * center_lat.to_radians().cos() * 111320.0;
    let z = (lat - center_lat) * 111320.0;
    (x, z)
}

// ─── GeoJSON parsing ────────────────────────────────────────

/// Extract coordinate pairs from ST_AsGeoJSON output.
/// Handles LineString, Polygon (outer ring only), MultiPolygon (first polygon outer ring).
fn parse_geojson_coords(geojson: &str) -> Vec<[f64; 2]> {
    let v: serde_json::Value = match serde_json::from_str(geojson) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    let geom_type = v["type"].as_str().unwrap_or("");
    match geom_type {
        "LineString" => {
            v["coordinates"].as_array()
                .map(|arr| arr.iter().filter_map(|c| {
                    let a = c.as_array()?;
                    Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?])
                }).collect())
                .unwrap_or_default()
        }
        "Polygon" => {
            v["coordinates"].as_array()
                .and_then(|rings| rings.first())
                .and_then(|ring| ring.as_array())
                .map(|arr| arr.iter().filter_map(|c| {
                    let a = c.as_array()?;
                    Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?])
                }).collect())
                .unwrap_or_default()
        }
        "MultiPolygon" => {
            v["coordinates"].as_array()
                .and_then(|polys| polys.first())
                .and_then(|poly| poly.as_array())
                .and_then(|rings| rings.first())
                .and_then(|ring| ring.as_array())
                .map(|arr| arr.iter().filter_map(|c| {
                    let a = c.as_array()?;
                    Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?])
                }).collect())
                .unwrap_or_default()
        }
        _ => vec![],
    }
}

// ─── Row → Raw conversion ───────────────────────────────────

fn rows_to_raw(
    roads: Vec<world_repo::RoadRow>,
    water: Vec<world_repo::WaterRow>,
    parks: Vec<world_repo::ParkRow>,
    buildings: Vec<world_repo::BuildingRow>,
) -> TileRawData {
    TileRawData {
        roads: roads.into_iter().map(|r| RawRoad {
            coords: parse_geojson_coords(&r.geojson),
            highway: r.highway,
            name: r.name,
            width: r.width_meters,
        }).collect(),
        water: water.into_iter().map(|w| RawWater {
            coords: parse_geojson_coords(&w.geojson),
            water_type: w.water_type,
            geom_type: w.geom_type,
            name: w.name,
        }).collect(),
        parks: parks.into_iter().map(|p| RawPark {
            coords: parse_geojson_coords(&p.geojson),
            park_type: p.park_type,
            name: p.name,
        }).collect(),
        buildings: buildings.into_iter().map(|b| RawBuilding {
            coords: parse_geojson_coords(&b.geojson),
            height: b.height_meters,
        }).collect(),
    }
}

// ─── Raw → Response conversion (applies center offset) ──────

fn raw_to_response(
    raw: &TileRawData,
    z: u8, x: u32, y: u32,
    center_lat: f64, center_lng: f64,
) -> TileResponse {
    let convert = |coords: &[[f64; 2]]| -> Vec<Point2D> {
        coords.iter().map(|[lng, lat]| {
            let (x, z) = geo_to_local(*lat, *lng, center_lat, center_lng);
            Point2D { x, z }
        }).collect()
    };

    TileResponse {
        tile: TileInfo { z, x, y },
        roads: raw.roads.iter().map(|r| RoadFeature {
            points: convert(&r.coords),
            highway: r.highway.clone(),
            name: r.name.clone(),
            width: r.width,
        }).collect(),
        water: raw.water.iter().map(|w| WaterFeature {
            points: convert(&w.coords),
            water_type: w.water_type.clone(),
            geom_type: w.geom_type.clone(),
            name: w.name.clone(),
        }).collect(),
        parks: raw.parks.iter().map(|p| ParkFeature {
            points: convert(&p.coords),
            park_type: p.park_type.clone(),
            name: p.name.clone(),
        }).collect(),
        buildings: raw.buildings.iter().map(|b| BuildingFeature {
            outline: convert(&b.coords),
            height: b.height,
        }).collect(),
    }
}

// ─── Main entry point ───────────────────────────────────────

const TILE_CACHE_TTL: u64 = 3600; // 1 hour

pub async fn get_tile(
    pool: &PgPool,
    redis: &RedisPool,
    z: u8, x: u32, y: u32,
    center_lat: f64, center_lng: f64,
) -> Result<TileResponse, AppError> {
    let cache_key = format!("tile:{z}:{x}:{y}");

    // Try Redis cache
    if let Ok(mut conn) = redis.get().await {
        if let Ok(cached) = conn.get::<_, Option<String>>(&cache_key).await {
            if let Some(json_str) = cached {
                if let Ok(raw) = serde_json::from_str::<TileRawData>(&json_str) {
                    return Ok(raw_to_response(&raw, z, x, y, center_lat, center_lng));
                }
            }
        }
    }

    // Cache miss – query PostGIS
    let (west, south, east, north) = world_repo::tile_to_bbox(z, x, y);
    let tolerance = world_repo::zoom_tolerance(z);

    let (roads_res, water_res, parks_res, buildings_res) = tokio::join!(
        world_repo::get_roads(pool, west, south, east, north, tolerance),
        world_repo::get_water(pool, west, south, east, north, tolerance),
        world_repo::get_parks(pool, west, south, east, north, tolerance),
        world_repo::get_buildings(pool, west, south, east, north, tolerance),
    );

    let raw = rows_to_raw(
        roads_res.map_err(|e| AppError::Internal(e.into()))?,
        water_res.map_err(|e| AppError::Internal(e.into()))?,
        parks_res.map_err(|e| AppError::Internal(e.into()))?,
        buildings_res.map_err(|e| AppError::Internal(e.into()))?,
    );

    // Cache in Redis (fire-and-forget)
    if let Ok(json_str) = serde_json::to_string(&raw) {
        if let Ok(mut conn) = redis.get().await {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json_str, TILE_CACHE_TTL).await;
        }
    }

    Ok(raw_to_response(&raw, z, x, y, center_lat, center_lng))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_to_local_origin() {
        let (x, z) = geo_to_local(35.6595, 139.7004, 35.6595, 139.7004);
        assert!((x).abs() < 0.01);
        assert!((z).abs() < 0.01);
    }

    #[test]
    fn test_geo_to_local_offset() {
        // ~100m east
        let (x, _z) = geo_to_local(35.6595, 139.7015, 35.6595, 139.7004);
        assert!((x - 100.0).abs() < 5.0); // ±5m tolerance
    }

    #[test]
    fn test_parse_geojson_linestring() {
        let geojson = r#"{"type":"LineString","coordinates":[[139.70,35.66],[139.71,35.67]]}"#;
        let coords = parse_geojson_coords(geojson);
        assert_eq!(coords.len(), 2);
        assert!((coords[0][0] - 139.70).abs() < 0.001);
    }

    #[test]
    fn test_parse_geojson_polygon() {
        let geojson = r#"{"type":"Polygon","coordinates":[[[139.70,35.66],[139.71,35.66],[139.71,35.67],[139.70,35.66]]]}"#;
        let coords = parse_geojson_coords(geojson);
        assert_eq!(coords.len(), 4);
    }
}
```

Note: `AppError::Internal` must exist in `error.rs`. Check the existing `AppError` enum – if it doesn't have an `Internal` variant, use whatever variant maps to 500 (e.g., `AppError::Database` or add `Internal(String)`).

- [ ] **Step 3: Verify compilation and run tests**

```bash
cargo check -p placebo-api
cargo test -p placebo-api -- world_service
```

- [ ] **Step 4: Commit**

```bash
git add crates/placebo-api/src/services/world_service.rs crates/placebo-api/src/services/mod.rs
git commit -m "feat(api): add world_service with tile caching, geo conversion, GeoJSON parsing"
```

### Task 4: Rust handler + router wiring

**Files:**
- Create: `crates/placebo-api/src/handlers/world.rs`
- Modify: `crates/placebo-api/src/handlers/mod.rs` (add `pub mod world;` + nest route)

- [ ] **Step 1: Write handler**

Create `crates/placebo-api/src/handlers/world.rs`:

```rust
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::services::world_service::{self, TileResponse};

#[derive(Debug, Deserialize)]
pub struct TileParams {
    pub z: u8,
    pub x: u32,
    pub y: u32,
    pub center_lat: f64,
    pub center_lng: f64,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/tile", get(get_tile))
}

async fn get_tile(
    State(state): State<AppState>,
    Query(params): Query<TileParams>,
) -> Result<Json<TileResponse>, AppError> {
    if params.z < 15 || params.z > 17 {
        return Err(AppError::Validation(
            "Zoom must be between 15 and 17".into(),
        ));
    }

    let response = world_service::get_tile(
        &state.db,
        &state.redis,
        params.z, params.x, params.y,
        params.center_lat, params.center_lng,
    )
    .await?;

    Ok(Json(response))
}
```

Note: Check `AppError` enum for `Validation` variant. If it doesn't exist, use the appropriate variant or add it.

- [ ] **Step 2: Wire into router**

In `crates/placebo-api/src/handlers/mod.rs`, add `pub mod world;` and update `api_router()`:

```rust
pub fn api_router() -> Router<AppState> {
    Router::new()
        .nest("/cameras", cameras::router()
            .merge(ratings::router())
            .merge(boosts::router())
            .merge(clips::camera_router())
        )
        .nest("/rooms", rooms::router())
        .nest("/users", users::router())
        .nest("/clips", clips::user_router())
        .nest("/world", world::router())
}
```

- [ ] **Step 3: Verify full build**

```bash
cargo build -p placebo-api
```

- [ ] **Step 4: Manual test (requires running API server + PostGIS)**

```bash
# In separate terminal: cargo run -p placebo-api
curl -s "http://localhost:3000/api/v1/world/tile?z=16&x=57483&y=25953&center_lat=35.6595&center_lng=139.7004" | jq '.roads | length, .water | length, .parks | length, .buildings | length'
```
Expected: non-zero counts for roads and buildings at minimum

- [ ] **Step 5: Commit**

```bash
git add crates/placebo-api/src/handlers/world.rs crates/placebo-api/src/handlers/mod.rs
git commit -m "feat(api): add /api/v1/world/tile endpoint for OSM tile data"
```

---

## Chunk 3: Frontend Types + Hook

### Task 5: Add city tile types to world3d.ts

**Files:**
- Modify: `src/types/world3d.ts`

- [ ] **Step 1: Add types at end of file**

Append to `src/types/world3d.ts` (after the `localToGeo` / `geoDistance` functions):

```typescript
// ─── City Tile Types ────────────────────────────────────────

export interface RoadSegment {
  points: { x: number; z: number }[];
  highway: string;
  name: string | null;
  width: number;
}

export const DEFAULT_ROAD_WIDTHS: Record<string, number> = {
  motorway: 15, trunk: 14, primary: 12, secondary: 9,
  tertiary: 7, residential: 6, unclassified: 5, service: 4,
  footway: 2, cycleway: 2, pedestrian: 3, path: 1.5, steps: 1.5,
};

export interface WaterFeature {
  points: { x: number; z: number }[];
  type: string;
  geomType: 'polygon' | 'line';
  name: string | null;
}

export interface ParkFeature {
  points: { x: number; z: number }[];
  type: string;
  name: string | null;
}

export interface BuildingFootprint {
  outline: { x: number; z: number }[];
  height: number;
}
```

- [ ] **Step 2: Update RoadNetwork.tsx import**

In `src/components/world3d/ground/RoadNetwork.tsx`, change the `RoadSegment` import:
- Remove inline `RoadSegment` interface if defined locally
- Add `import { RoadSegment } from '../../../types/world3d';`

- [ ] **Step 3: Update GroundSystem.tsx import**

Same pattern – import `RoadSegment` from `types/world3d` instead of local/hook source.

- [ ] **Step 4: Verify TypeScript compiles**

```bash
npx tsc --noEmit
```

- [ ] **Step 5: Commit**

```bash
git add src/types/world3d.ts src/components/world3d/ground/RoadNetwork.tsx src/components/world3d/ground/GroundSystem.tsx
git commit -m "feat(types): add WaterFeature, ParkFeature, BuildingFootprint types"
```

### Task 6: Create useCityTiles hook

**Files:**
- Create: `src/hooks/useCityTiles.ts`

- [ ] **Step 1: Write the hook**

Create `src/hooks/useCityTiles.ts`:

```typescript
import { useState, useEffect, useRef } from 'react';
import type { RoadSegment, WaterFeature, ParkFeature, BuildingFootprint } from '../types/world3d';

// API base – in dev this is the local axum server
const API_BASE = 'http://localhost:3000';

interface CityTilesResult {
  roads: RoadSegment[];
  water: WaterFeature[];
  parks: ParkFeature[];
  buildings: BuildingFootprint[];
  loading: boolean;
  error: string | null;
}

interface TileCoord { z: number; x: number; y: number }

function latLngToTile(lat: number, lng: number, zoom: number): { x: number; y: number } {
  const n = 2 ** zoom;
  const x = Math.floor(((lng + 180) / 360) * n);
  const latRad = (lat * Math.PI) / 180;
  const y = Math.floor(
    ((1 - Math.log(Math.tan(latRad) + 1 / Math.cos(latRad)) / Math.PI) / 2) * n
  );
  return { x, y };
}

function getVisibleTiles(lat: number, lng: number, zoom: number): TileCoord[] {
  const center = latLngToTile(lat, lng, zoom);
  const tiles: TileCoord[] = [];
  for (let dx = -1; dx <= 1; dx++) {
    for (let dy = -1; dy <= 1; dy++) {
      tiles.push({ z: zoom, x: center.x + dx, y: center.y + dy });
    }
  }
  return tiles;
}

function tileCacheKey(tiles: TileCoord[]): string {
  return tiles.map(t => `${t.z}/${t.x}/${t.y}`).sort().join(',');
}

export function useCityTiles(
  centerLat: number,
  centerLng: number,
  zoom: number = 16,
): CityTilesResult {
  const [roads, setRoads] = useState<RoadSegment[]>([]);
  const [water, setWater] = useState<WaterFeature[]>([]);
  const [parks, setParks] = useState<ParkFeature[]>([]);
  const [buildings, setBuildings] = useState<BuildingFootprint[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const prevTileKey = useRef<string>('');

  useEffect(() => {
    const tiles = getVisibleTiles(centerLat, centerLng, zoom);
    const key = tileCacheKey(tiles);

    // Skip if same tiles
    if (key === prevTileKey.current) return;
    prevTileKey.current = key;

    const controller = new AbortController();
    setLoading(true);
    setError(null);

    const fetchTiles = async () => {
      try {
        const results = await Promise.allSettled(
          tiles.map(t =>
            fetch(
              `${API_BASE}/api/v1/world/tile?z=${t.z}&x=${t.x}&y=${t.y}&center_lat=${centerLat}&center_lng=${centerLng}`,
              { signal: controller.signal }
            ).then(res => {
              if (!res.ok) throw new Error(`Tile ${t.z}/${t.x}/${t.y}: ${res.status}`);
              return res.json();
            })
          )
        );

        const allRoads: RoadSegment[] = [];
        const allWater: WaterFeature[] = [];
        const allParks: ParkFeature[] = [];
        const allBuildings: BuildingFootprint[] = [];

        for (const result of results) {
          if (result.status === 'fulfilled') {
            const data = result.value;
            if (data.roads) allRoads.push(...data.roads);
            if (data.water) allWater.push(...data.water);
            if (data.parks) allParks.push(...data.parks);
            if (data.buildings) allBuildings.push(...data.buildings);
          }
        }

        setRoads(allRoads);
        setWater(allWater);
        setParks(allParks);
        setBuildings(allBuildings);
      } catch (err: unknown) {
        if (err instanceof Error && err.name !== 'AbortError') {
          setError(err.message);
        }
      } finally {
        setLoading(false);
      }
    };

    fetchTiles();
    return () => controller.abort();
  }, [centerLat, centerLng, zoom]);

  return { roads, water, parks, buildings, loading, error };
}
```

- [ ] **Step 2: Verify TypeScript compiles**

```bash
npx tsc --noEmit
```

- [ ] **Step 3: Commit**

```bash
git add src/hooks/useCityTiles.ts
git commit -m "feat(hooks): add useCityTiles hook for tile-based city geometry fetching"
```

---

## Chunk 4: R3F Components (Water, Parks, Buildings, DynamicFog)

### Task 7: WaterBodies component

**Files:**
- Create: `src/components/world3d/ground/WaterBodies.tsx`

- [ ] **Step 1: Write component**

Create `src/components/world3d/ground/WaterBodies.tsx`:

```tsx
import { useMemo } from 'react';
import * as THREE from 'three';
import type { WaterFeature } from '../../../types/world3d';

const WATER_COLOR = '#0a1a3a';
const WATER_OPACITY = 0.4;
const RIVER_WIDTH = 8; // meters

interface WaterBodiesProps {
  water: WaterFeature[];
}

function tessellateRibbon(points: { x: number; z: number }[], halfWidth: number): Float32Array {
  const verts: number[] = [];
  for (let i = 0; i < points.length - 1; i++) {
    const p0 = points[i];
    const p1 = points[i + 1];
    const dx = p1.x - p0.x;
    const dz = p1.z - p0.z;
    const len = Math.sqrt(dx * dx + dz * dz);
    if (len < 0.001) continue;
    const nx = (-dz / len) * halfWidth;
    const nz = (dx / len) * halfWidth;
    const y = 0.02;
    verts.push(p0.x - nx, y, p0.z - nz);
    verts.push(p0.x + nx, y, p0.z + nz);
    verts.push(p1.x - nx, y, p1.z - nz);
    verts.push(p1.x - nx, y, p1.z - nz);
    verts.push(p0.x + nx, y, p0.z + nz);
    verts.push(p1.x + nx, y, p1.z + nz);
  }
  return new Float32Array(verts);
}

export function WaterBodies({ water }: WaterBodiesProps) {
  const { polygonGeo, lineGeo } = useMemo(() => {
    // Polygon water (lakes, ponds)
    const shapes: THREE.Shape[] = [];
    for (const w of water) {
      if (w.geomType !== 'polygon' || w.points.length < 3) continue;
      const shape = new THREE.Shape();
      shape.moveTo(w.points[0].x, w.points[0].z);
      for (let i = 1; i < w.points.length; i++) {
        shape.lineTo(w.points[i].x, w.points[i].z);
      }
      shapes.push(shape);
    }
    const polygonGeo = shapes.length > 0
      ? new THREE.ShapeGeometry(shapes)
      : null;

    // Line water (rivers, streams) – ribbon mesh
    const allVerts: number[] = [];
    for (const w of water) {
      if (w.geomType !== 'line' || w.points.length < 2) continue;
      const ribbon = tessellateRibbon(w.points, RIVER_WIDTH / 2);
      for (let i = 0; i < ribbon.length; i++) allVerts.push(ribbon[i]);
    }
    const lineGeo = allVerts.length > 0
      ? new THREE.BufferGeometry().setAttribute(
          'position',
          new THREE.Float32BufferAttribute(allVerts, 3)
        )
      : null;

    return { polygonGeo, lineGeo };
  }, [water]);

  return (
    <group>
      {polygonGeo && (
        <mesh geometry={polygonGeo} rotation={[-Math.PI / 2, 0, 0]} position={[0, 0.02, 0]}>
          <meshBasicMaterial
            color={WATER_COLOR}
            opacity={WATER_OPACITY}
            transparent
            depthWrite={false}
            side={THREE.DoubleSide}
          />
        </mesh>
      )}
      {lineGeo && (
        <mesh geometry={lineGeo}>
          <meshBasicMaterial
            color={WATER_COLOR}
            opacity={WATER_OPACITY}
            transparent
            depthWrite={false}
            side={THREE.DoubleSide}
          />
        </mesh>
      )}
    </group>
  );
}
```

Note: `ShapeGeometry` is created in XY plane, so we rotate -PI/2 around X to lay flat on XZ. The ribbon mesh is already in XZ plane (y=0.02 hardcoded).

- [ ] **Step 2: Commit**

```bash
git add src/components/world3d/ground/WaterBodies.tsx
git commit -m "feat(3d): add WaterBodies component for lakes and rivers"
```

### Task 8: Parks component

**Files:**
- Create: `src/components/world3d/ground/Parks.tsx`

- [ ] **Step 1: Write component**

Create `src/components/world3d/ground/Parks.tsx`:

```tsx
import { useMemo } from 'react';
import * as THREE from 'three';
import type { ParkFeature } from '../../../types/world3d';

const PARK_COLOR = '#0a2a0a';
const PARK_OPACITY = 0.3;

interface ParksProps {
  parks: ParkFeature[];
}

export function Parks({ parks }: ParksProps) {
  const geometry = useMemo(() => {
    const shapes: THREE.Shape[] = [];
    for (const p of parks) {
      if (p.points.length < 3) continue;
      const shape = new THREE.Shape();
      shape.moveTo(p.points[0].x, p.points[0].z);
      for (let i = 1; i < p.points.length; i++) {
        shape.lineTo(p.points[i].x, p.points[i].z);
      }
      shapes.push(shape);
    }
    return shapes.length > 0 ? new THREE.ShapeGeometry(shapes) : null;
  }, [parks]);

  if (!geometry) return null;

  return (
    <mesh geometry={geometry} rotation={[-Math.PI / 2, 0, 0]} position={[0, 0.01, 0]}>
      <meshBasicMaterial
        color={PARK_COLOR}
        opacity={PARK_OPACITY}
        transparent
        depthWrite={false}
        side={THREE.DoubleSide}
      />
    </mesh>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/world3d/ground/Parks.tsx
git commit -m "feat(3d): add Parks component for green areas"
```

### Task 9: BuildingsLayer rewrite

**Files:**
- Modify: `src/components/world3d/BuildingsLayer.tsx`

- [ ] **Step 1: Rewrite BuildingsLayer**

Replace entire content of `src/components/world3d/BuildingsLayer.tsx`:

```tsx
import { useMemo } from 'react';
import * as THREE from 'three';
import { mergeGeometries } from 'three/examples/jsm/utils/BufferGeometryUtils.js';
import type { BuildingFootprint } from '../../types/world3d';

const FILL_COLOR = '#0a0f18';
const FILL_OPACITY = 0.06;
const EDGE_COLOR = '#1e2840';
const EDGE_OPACITY = 0.4;

interface BuildingsLayerProps {
  buildings: BuildingFootprint[];
}

export function BuildingsLayer({ buildings }: BuildingsLayerProps) {
  const { fillGeo, edgeGeo } = useMemo(() => {
    if (buildings.length === 0) return { fillGeo: null, edgeGeo: null };

    const fillGeometries: THREE.BufferGeometry[] = [];
    const edgeGeometries: THREE.BufferGeometry[] = [];

    for (const b of buildings) {
      if (b.outline.length < 3 || b.height <= 0) continue;

      const shape = new THREE.Shape();
      shape.moveTo(b.outline[0].x, b.outline[0].z);
      for (let i = 1; i < b.outline.length; i++) {
        shape.lineTo(b.outline[i].x, b.outline[i].z);
      }

      const extruded = new THREE.ExtrudeGeometry(shape, {
        depth: b.height,
        bevelEnabled: false,
      });

      // ExtrudeGeometry extrudes along local Z. We need Y-up.
      // Rotate -90° around X to convert Z-up → Y-up
      extruded.rotateX(-Math.PI / 2);

      fillGeometries.push(extruded);
      edgeGeometries.push(new THREE.EdgesGeometry(extruded));
    }

    const fillGeo = fillGeometries.length > 0
      ? mergeGeometries(fillGeometries, false)
      : null;
    const edgeGeo = edgeGeometries.length > 0
      ? mergeGeometries(edgeGeometries, false)
      : null;

    // Dispose individual geometries
    for (const g of fillGeometries) g.dispose();
    for (const g of edgeGeometries) g.dispose();

    return { fillGeo: fillGeo ?? null, edgeGeo: edgeGeo ?? null };
  }, [buildings]);

  return (
    <group>
      {fillGeo && (
        <mesh geometry={fillGeo}>
          <meshBasicMaterial
            color={FILL_COLOR}
            opacity={FILL_OPACITY}
            transparent
            depthWrite={false}
            side={THREE.DoubleSide}
          />
        </mesh>
      )}
      {edgeGeo && (
        <lineSegments geometry={edgeGeo}>
          <lineBasicMaterial
            color={EDGE_COLOR}
            opacity={EDGE_OPACITY}
            transparent
            depthWrite={false}
          />
        </lineSegments>
      )}
    </group>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/world3d/BuildingsLayer.tsx
git commit -m "feat(3d): rewrite BuildingsLayer with real OSM footprints + wireframe"
```

### Task 10: DynamicFog component

**Files:**
- Create: `src/components/world3d/DynamicFog.tsx`

- [ ] **Step 1: Write component**

Create `src/components/world3d/DynamicFog.tsx`:

```tsx
import { useRef, useEffect } from 'react';
import { useThree, useFrame } from '@react-three/fiber';
import * as THREE from 'three';
import { useTimeOfDay } from '../../hooks/useTimeOfDay';

const FOG_COLORS: Record<string, string> = {
  night: '#050510',
  dawn: '#1a1020',
  morning: '#2a4060',
  day: '#4a6a8a',
  dusk: '#3a2020',
  twilight: '#0a0a1a',
};

interface DynamicFogProps {
  timezone: string;
  near: number;
  far: number;
}

export function DynamicFog({ timezone, near, far }: DynamicFogProps) {
  const { scene } = useThree();
  const timeOfDay = useTimeOfDay(timezone);
  const targetColor = useRef(new THREE.Color(FOG_COLORS.day));

  useEffect(() => {
    const colorHex = FOG_COLORS[timeOfDay.phase] || FOG_COLORS.day;
    targetColor.current.set(colorHex);
  }, [timeOfDay.phase]);

  useEffect(() => {
    scene.fog = new THREE.Fog(FOG_COLORS.day, near, far);
    return () => { scene.fog = null; };
  }, [scene, near, far]);

  useFrame(() => {
    if (scene.fog instanceof THREE.Fog) {
      scene.fog.color.lerp(targetColor.current, 0.02);
    }
  });

  return null;
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/world3d/DynamicFog.tsx
git commit -m "feat(3d): add DynamicFog with time-of-day color transitions"
```

---

## Chunk 5: Integration + Cleanup

### Task 11: Update GroundSystem

**Files:**
- Modify: `src/components/world3d/ground/GroundSystem.tsx`
- Modify: `src/components/world3d/ground/index.ts`

- [ ] **Step 1: Update GroundSystem props and children**

In `src/components/world3d/ground/GroundSystem.tsx`:
- Add imports: `WaterBodies` from `./WaterBodies`, `Parks` from `./Parks`
- Add types: `WaterFeature`, `ParkFeature` from `../../../types/world3d`
- Update `GroundSystemProps`: add `water: WaterFeature[]`, `parks: ParkFeature[]`
- Add `<WaterBodies water={water} />` and `<Parks parks={parks} />` as children

Updated component:
```tsx
import { useQuality } from '../../../hooks/useQuality';
import { GroundPlane } from './GroundPlane';
import { GroundGrid } from './GroundGrid';
import { RoadNetwork } from './RoadNetwork';
import { WaterBodies } from './WaterBodies';
import { Parks } from './Parks';
import type { RoadSegment, WaterFeature, ParkFeature } from '../../../types/world3d';

interface GroundSystemProps {
  roads: RoadSegment[];
  water: WaterFeature[];
  parks: ParkFeature[];
}

export function GroundSystem({ roads, water, parks }: GroundSystemProps) {
  const quality = useQuality();

  return (
    <group>
      <GroundPlane />
      {quality.ground.gridEnabled && <GroundGrid />}
      <RoadNetwork roads={roads} />
      <WaterBodies water={water} />
      <Parks parks={parks} />
    </group>
  );
}
```

- [ ] **Step 2: Update ground/index.ts exports**

Add exports for `WaterBodies` and `Parks`:
```typescript
export { GroundSystem } from './GroundSystem';
export { GroundPlane } from './GroundPlane';
export { GroundGrid } from './GroundGrid';
export { RoadNetwork } from './RoadNetwork';
export { WaterBodies } from './WaterBodies';
export { Parks } from './Parks';
```

- [ ] **Step 3: Commit**

```bash
git add src/components/world3d/ground/GroundSystem.tsx src/components/world3d/ground/index.ts
git commit -m "feat(3d): update GroundSystem with water and parks layers"
```

### Task 12: WorldScene integration

**Files:**
- Modify: `src/components/world3d/WorldScene.tsx`
- Modify: `src/components/world3d/index.ts`

- [ ] **Step 1: Update WorldScene.tsx**

Key changes:
1. Replace `import { useRoadNetwork }` with `import { useCityTiles }`
2. Replace `const { roads } = useRoadNetwork(...)` with `const { roads, water, parks, buildings } = useCityTiles(activeCamera.lat, activeCamera.lng, 16)`
3. Add `import { DynamicFog } from './DynamicFog'`
4. Replace static `<fog attach="fog" args={[...]} />` with `<DynamicFog timezone={timezone} near={quality.fog.near} far={quality.fog.far} />`
5. Update `<GroundSystem roads={roads} />` to `<GroundSystem roads={roads} water={water} parks={parks} />`
6. Update `<BuildingsLayer tilesUrl={...} ... />` to `<BuildingsLayer buildings={buildings} />`
7. Remove `tilesUrl` from `WorldSceneProps` interface

- [ ] **Step 2: Update world3d/index.ts exports**

Add `export { DynamicFog } from './DynamicFog';`

- [ ] **Step 3: Verify TypeScript compiles**

```bash
npx tsc --noEmit
```

- [ ] **Step 4: Commit**

```bash
git add src/components/world3d/WorldScene.tsx src/components/world3d/index.ts
git commit -m "feat(3d): integrate useCityTiles, DynamicFog, and real buildings into WorldScene"
```

### Task 13: Delete useRoadNetwork

**Files:**
- Delete: `src/hooks/useRoadNetwork.ts`

- [ ] **Step 1: Verify no remaining imports**

```bash
grep -r "useRoadNetwork" src/
```
Expected: zero results

- [ ] **Step 2: Delete the file**

```bash
rm src/hooks/useRoadNetwork.ts
```

- [ ] **Step 3: Final TypeScript check**

```bash
npx tsc --noEmit
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore: remove useRoadNetwork (replaced by useCityTiles)"
```

---

## Verification

### End-to-end test sequence

1. **SQL views**: `psql -d placebo -c "SELECT count(*) FROM roads_view;"` – should return rows
2. **Rust build**: `cargo build -p placebo-api` – compiles clean
3. **Rust tests**: `cargo test -p placebo-api` – all pass
4. **API server**: `cargo run -p placebo-api` (needs PostGIS + Redis running)
5. **Tile endpoint**: `curl "http://localhost:3000/api/v1/world/tile?z=16&x=57483&y=25953&center_lat=35.6595&center_lng=139.7004" | jq '.roads | length'`
6. **Frontend build**: `npx tsc --noEmit` – compiles clean
7. **Visual test**: `npm run dev` → open 3D world → see roads, water, parks, wireframe buildings

### What to look for visually
- Roads: dark translucent ribbons on the ground (same as before but from real API data)
- Water: blue translucent shapes (lakes) and ribbons (rivers) at y=0.02
- Parks: green translucent polygons at y=0.01
- Buildings: glass wireframe extrusions with real footprints and heights
- Fog: color transitions smoothly between time-of-day phases

### Known limitations
- `mergeGeometries` for 2600+ buildings may use significant memory – monitor FPS
- `ST_Area(way::geography) > 10` in buildings_tile_view is computed per query (not materialized) – if slow, consider materializing
- No client-side tile caching – every camera switch re-fetches tiles
