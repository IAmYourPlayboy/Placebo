# OSM Tile Pipeline Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver real OSM geometry (roads, water, parks, buildings) from PostGIS to the Placebo 3D world via a tile-based JSON API.

**Architecture:** Axum endpoint queries 4 PostGIS views in parallel, caches raw results in Redis, converts lat/lng to local meters per-request, returns JSON. React hook `useCityTiles` fetches a 3×3 tile grid and feeds data to R3F components.

**Tech Stack:** Rust/axum + sqlx + PostGIS, Redis, React + TypeScript, React Three Fiber, Three.js

**Spec:** `docs/superpowers/specs/2026-03-16-osm-tile-pipeline-design.md`

---

## File Structure

### New files

| File | Responsibility |
|------|----------------|
| `pipeline/sql/03_roads_view.sql` | SQL: roads_view with width_meters |
| `pipeline/sql/04_water_view.sql` | SQL: water_view (polygon + line UNION) |
| `pipeline/sql/05_parks_view.sql` | SQL: parks_view |
| `pipeline/sql/06_buildings_tile_view.sql` | SQL: buildings_tile_view (2D, for tiles) |
| `pipeline/sql/07_indexes.sql` | Partial indexes for filtered queries |
| `crates/placebo-api/src/handlers/world.rs` | Tile endpoint handler + query params |
| `crates/placebo-api/src/repositories/world_repo.rs` | PostGIS queries for 4 layers |
| `crates/placebo-api/src/services/world_service.rs` | Coord conversion, response assembly, Redis cache |
| `src/hooks/useCityTiles.ts` | Tile fetching + merging hook |
| `src/components/world3d/ground/WaterBodies.tsx` | Water polygon/ribbon rendering |
| `src/components/world3d/ground/Parks.tsx` | Park polygon rendering |
| `src/components/world3d/DynamicFog.tsx` | Time-based fog color |

### Modified files

| File | Change |
|------|--------|
| `src/types/world3d.ts` | Add RoadSegment, WaterFeature, ParkFeature, BuildingFootprint, DEFAULT_ROAD_WIDTHS |
| `crates/placebo-api/src/handlers/mod.rs` | Add `pub mod world;`, nest `/world` route |
| `crates/placebo-api/src/repositories/mod.rs` | Add `pub mod world_repo;` |
| `crates/placebo-api/src/services/mod.rs` | Add `pub mod world_service;` |
| `src/components/world3d/BuildingsLayer.tsx` | Rewrite: ExtrudeGeometry from real footprints |
| `src/components/world3d/ground/GroundSystem.tsx` | Add water + parks children |
| `src/components/world3d/ground/index.ts` | Export WaterBodies, Parks |
| `src/components/world3d/WorldScene.tsx` | useCityTiles, DynamicFog, new props |
| `src/components/world3d/ground/RoadNetwork.tsx` | Import RoadSegment from types/ |
| `src/components/world3d/lighting/LightingSystem.tsx` | Import RoadSegment from types/ |
| `src/components/world3d/lighting/NightLights.tsx` | Import RoadSegment from types/ |
| `src/screens/World3DScreen.tsx` | Remove tilesUrl prop from WorldScene |

### Deleted files

| File | Reason |
|------|--------|
| `src/hooks/useRoadNetwork.ts` | Replaced by useCityTiles (deleted in Chunk 5, after all imports updated) |

---

## Chunk 1: PostGIS Views & Indexes

### Task 1: Create SQL views for roads, water, parks, buildings

**Files:**
- Create: `pipeline/sql/03_roads_view.sql`
- Create: `pipeline/sql/04_water_view.sql`
- Create: `pipeline/sql/05_parks_view.sql`
- Create: `pipeline/sql/06_buildings_tile_view.sql`
- Create: `pipeline/sql/07_indexes.sql`

- [ ] **Step 1: Write roads_view SQL**

Create `pipeline/sql/03_roads_view.sql`:

```sql
-- Placebo — Roads View for tile endpoint
-- Filters valid highways with computed width in meters

CREATE OR REPLACE VIEW roads_view AS
SELECT
  osm_id,
  highway,
  name,
  way,
  (CASE highway
    WHEN 'motorway' THEN 15.0
    WHEN 'trunk' THEN 14.0
    WHEN 'primary' THEN 12.0
    WHEN 'secondary' THEN 9.0
    WHEN 'tertiary' THEN 7.0
    WHEN 'residential' THEN 6.0
    WHEN 'unclassified' THEN 5.0
    WHEN 'service' THEN 4.0
    WHEN 'footway' THEN 2.0
    WHEN 'cycleway' THEN 2.0
    WHEN 'steps' THEN 1.5
    WHEN 'pedestrian' THEN 3.0
    WHEN 'path' THEN 1.5
    ELSE 5.0
  END)::float8 AS width_meters
FROM planet_osm_line
WHERE highway IS NOT NULL
  AND highway NOT IN ('proposed', 'construction', 'abandoned', 'platform');
```

- [ ] **Step 2: Write water_view SQL**

Create `pipeline/sql/04_water_view.sql`:

```sql
-- Placebo — Water View for tile endpoint
-- UNION of polygon water (lakes) and linear water (rivers/streams)

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

- [ ] **Step 3: Write parks_view SQL**

Create `pipeline/sql/05_parks_view.sql`:

```sql
-- Placebo — Parks View for tile endpoint

CREATE OR REPLACE VIEW parks_view AS
SELECT osm_id,
       COALESCE(leisure, "natural", landuse) AS park_type,
       name, way
FROM planet_osm_polygon
WHERE leisure IN ('park', 'garden', 'playground', 'pitch')
   OR "natural" IN ('wood', 'scrub', 'grassland')
   OR landuse IN ('grass', 'forest', 'recreation_ground', 'cemetery', 'meadow');
```

- [ ] **Step 4: Write buildings_tile_view SQL**

Create `pipeline/sql/06_buildings_tile_view.sql`:

```sql
-- Placebo — Buildings Tile View (2D footprints for tile endpoint)
-- Separate from buildings_3d which is for pg2b3dm

CREATE OR REPLACE VIEW buildings_tile_view AS
SELECT
  osm_id,
  way,
  (CASE
    WHEN height ~ '^\d+(\.\d+)?$' THEN height::float8
    WHEN "building:levels" ~ '^\d+$' THEN "building:levels"::int * 3.0
    ELSE 9.0
  END)::float8 AS height_meters,
  name
FROM planet_osm_polygon
WHERE building IS NOT NULL
  AND building NOT IN ('no', 'entrance')
  AND ST_Area(way::geography) > 10;
```

- [ ] **Step 5: Write partial indexes**

Create `pipeline/sql/07_indexes.sql`:

```sql
-- Placebo — Partial indexes for tile query performance

CREATE INDEX IF NOT EXISTS idx_line_highway
  ON planet_osm_line (highway)
  WHERE highway IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_polygon_building
  ON planet_osm_polygon (building)
  WHERE building IS NOT NULL AND building != 'no';

CREATE INDEX IF NOT EXISTS idx_polygon_water
  ON planet_osm_polygon (water)
  WHERE water IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_polygon_leisure
  ON planet_osm_polygon (leisure)
  WHERE leisure IS NOT NULL;
```

- [ ] **Step 6: Execute all SQL against the placebo database**

Run:
```bash
psql -d placebo -f pipeline/sql/03_roads_view.sql
psql -d placebo -f pipeline/sql/04_water_view.sql
psql -d placebo -f pipeline/sql/05_parks_view.sql
psql -d placebo -f pipeline/sql/06_buildings_tile_view.sql
psql -d placebo -f pipeline/sql/07_indexes.sql
```

- [ ] **Step 7: Verify views with test queries**

Run:
```bash
psql -d placebo -c "SELECT count(*) FROM roads_view;"
psql -d placebo -c "SELECT count(*) FROM water_view;"
psql -d placebo -c "SELECT count(*) FROM parks_view;"
psql -d placebo -c "SELECT count(*) FROM buildings_tile_view;"
```

Expected: All return row counts > 0 (roads ~800K+, buildings ~500K+).

- [ ] **Step 8: Verify spatial query performance for Shibuya bbox**

Run:
```bash
psql -d placebo -c "EXPLAIN ANALYZE SELECT count(*) FROM roads_view WHERE ST_Intersects(way, ST_MakeEnvelope(139.695, 35.658, 139.705, 35.665, 4326));"
```

Expected: Uses GIST index scan, completes in <50ms.

- [ ] **Step 9: Commit**

```bash
git add pipeline/sql/03_roads_view.sql pipeline/sql/04_water_view.sql pipeline/sql/05_parks_view.sql pipeline/sql/06_buildings_tile_view.sql pipeline/sql/07_indexes.sql
git commit -m "feat(pipeline): add SQL views for roads, water, parks, buildings tiles"
```

---

## Chunk 2: Axum Tile API (Backend)

### Task 2: World repository – PostGIS queries

**Files:**
- Create: `crates/placebo-api/src/repositories/world_repo.rs`
- Modify: `crates/placebo-api/src/repositories/mod.rs`

- [ ] **Step 1: Write world_repo.rs with raw row types and query functions**

Create `crates/placebo-api/src/repositories/world_repo.rs`:

```rust
use sqlx::PgPool;

// ---------------------------------------------------------------------------
// Raw row types from PostGIS
// ---------------------------------------------------------------------------

#[derive(Debug, sqlx::FromRow)]
pub struct RoadRow {
    pub osm_id: i64,
    pub highway: String,
    pub name: Option<String>,
    pub width_meters: f64,
    pub coords_json: serde_json::Value, // ST_AsGeoJSON(...)::jsonb
}

#[derive(Debug, sqlx::FromRow)]
pub struct WaterRow {
    pub osm_id: i64,
    pub geom_type: String,
    pub water_type: Option<String>,
    pub name: Option<String>,
    pub coords_json: serde_json::Value,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ParkRow {
    pub osm_id: i64,
    pub park_type: Option<String>,
    pub name: Option<String>,
    pub coords_json: serde_json::Value,
}

#[derive(Debug, sqlx::FromRow)]
pub struct BuildingRow {
    pub osm_id: i64,
    pub height_meters: f64,
    pub name: Option<String>,
    pub coords_json: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Tolerance by zoom level (degrees, ~meters at equator)
// ---------------------------------------------------------------------------

fn simplify_tolerance(zoom: u8) -> f64 {
    match zoom {
        17 => 0.00001,  // ~1m
        16 => 0.00005,  // ~5m
        _  => 0.0001,   // ~10m (z15)
    }
}

// ---------------------------------------------------------------------------
// Query functions – use ST_MakeEnvelope with bound params (no SQL injection)
// ---------------------------------------------------------------------------

pub async fn get_roads(
    pool: &PgPool,
    west: f64, south: f64, east: f64, north: f64,
    zoom: u8,
) -> Result<Vec<RoadRow>, sqlx::Error> {
    let tol = simplify_tolerance(zoom);
    sqlx::query_as::<_, RoadRow>(&format!(
        r#"SELECT osm_id, highway, name, width_meters,
           ST_AsGeoJSON(ST_SimplifyPreserveTopology(way, {tol}))::jsonb AS coords_json
           FROM roads_view
           WHERE ST_Intersects(way, ST_MakeEnvelope($1, $2, $3, $4, 4326))
           LIMIT 5000"#
    ))
    .bind(west).bind(south).bind(east).bind(north)
    .fetch_all(pool)
    .await
}

pub async fn get_water(
    pool: &PgPool,
    west: f64, south: f64, east: f64, north: f64,
    zoom: u8,
) -> Result<Vec<WaterRow>, sqlx::Error> {
    let tol = simplify_tolerance(zoom);
    sqlx::query_as::<_, WaterRow>(&format!(
        r#"SELECT osm_id, geom_type, water_type, name,
           ST_AsGeoJSON(ST_SimplifyPreserveTopology(way, {tol}))::jsonb AS coords_json
           FROM water_view
           WHERE ST_Intersects(way, ST_MakeEnvelope($1, $2, $3, $4, 4326))
           LIMIT 2000"#
    ))
    .bind(west).bind(south).bind(east).bind(north)
    .fetch_all(pool)
    .await
}

pub async fn get_parks(
    pool: &PgPool,
    west: f64, south: f64, east: f64, north: f64,
    zoom: u8,
) -> Result<Vec<ParkRow>, sqlx::Error> {
    let tol = simplify_tolerance(zoom);
    sqlx::query_as::<_, ParkRow>(&format!(
        r#"SELECT osm_id, park_type, name,
           ST_AsGeoJSON(ST_SimplifyPreserveTopology(way, {tol}))::jsonb AS coords_json
           FROM parks_view
           WHERE ST_Intersects(way, ST_MakeEnvelope($1, $2, $3, $4, 4326))
           LIMIT 2000"#
    ))
    .bind(west).bind(south).bind(east).bind(north)
    .fetch_all(pool)
    .await
}

pub async fn get_buildings(
    pool: &PgPool,
    west: f64, south: f64, east: f64, north: f64,
    zoom: u8,
) -> Result<Vec<BuildingRow>, sqlx::Error> {
    let tol = simplify_tolerance(zoom);
    sqlx::query_as::<_, BuildingRow>(&format!(
        r#"SELECT osm_id, height_meters, name,
           ST_AsGeoJSON(ST_SimplifyPreserveTopology(way, {tol}))::jsonb AS coords_json
           FROM buildings_tile_view
           WHERE ST_Intersects(way, ST_MakeEnvelope($1, $2, $3, $4, 4326))
           LIMIT 5000"#
    ))
    .bind(west).bind(south).bind(east).bind(north)
    .fetch_all(pool)
    .await
}
```

- [ ] **Step 2: Register world_repo in mod.rs**

Add to `crates/placebo-api/src/repositories/mod.rs`:

```rust
pub mod world_repo;
```

- [ ] **Step 3: Verify it compiles**

Run: `cd /Users/notebook/Placebo && cargo check -p placebo-api`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add crates/placebo-api/src/repositories/world_repo.rs crates/placebo-api/src/repositories/mod.rs
git commit -m "feat(api): add world_repo with PostGIS tile queries"
```

### Task 3: World service – coordinate conversion, caching, response assembly

**Files:**
- Create: `crates/placebo-api/src/services/world_service.rs`
- Modify: `crates/placebo-api/src/services/mod.rs`

- [ ] **Step 1: Write world_service.rs**

Create `crates/placebo-api/src/services/world_service.rs`:

```rust
use deadpool_redis::Pool as RedisPool;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AppError;
use crate::repositories::world_repo;

// ---------------------------------------------------------------------------
// Slippy Map math
// ---------------------------------------------------------------------------

/// Compute tile bbox as (west, south, east, north) from z/x/y
pub fn tile_to_bbox(z: u8, x: u32, y: u32) -> (f64, f64, f64, f64) {
    let n = 2.0_f64.powi(z as i32);
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

// ---------------------------------------------------------------------------
// Coordinate conversion (lat/lng → local meters)
// ---------------------------------------------------------------------------

const METERS_PER_DEGREE: f64 = 111320.0;

fn geo_to_local(lat: f64, lng: f64, center_lat: f64, center_lng: f64) -> (f64, f64) {
    let cos_lat = center_lat.to_radians().cos();
    let x = (lng - center_lng) * cos_lat * METERS_PER_DEGREE;
    let z = (lat - center_lat) * METERS_PER_DEGREE;
    (x, z)
}

// ---------------------------------------------------------------------------
// Cached raw data (stored in Redis as JSON)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// API response types (local meters)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct TileResponse {
    pub tile: TileInfo,
    pub roads: Vec<RoadFeature>,
    pub water: Vec<WaterFeature>,
    pub parks: Vec<ParkFeature>,
    pub buildings: Vec<BuildingFeature>,
}

#[derive(Debug, Serialize)]
pub struct TileInfo {
    pub z: u8,
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Serialize)]
pub struct Point2D {
    pub x: f64,
    pub z: f64,
}

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

// ---------------------------------------------------------------------------
// Extract coords from GeoJSON
// ---------------------------------------------------------------------------

fn extract_coords(geojson: &serde_json::Value) -> Vec<[f64; 2]> {
    match geojson.get("type").and_then(|t| t.as_str()) {
        Some("LineString") => {
            geojson["coordinates"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|c| {
                            let a = c.as_array()?;
                            Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?])
                        })
                        .collect()
                })
                .unwrap_or_default()
        }
        Some("Polygon") => {
            // Take outer ring (first element of coordinates)
            geojson["coordinates"]
                .as_array()
                .and_then(|rings| rings.first())
                .and_then(|ring| ring.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|c| {
                            let a = c.as_array()?;
                            Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?])
                        })
                        .collect()
                })
                .unwrap_or_default()
        }
        Some("MultiPolygon") => {
            // Take first polygon, outer ring
            geojson["coordinates"]
                .as_array()
                .and_then(|polys| polys.first())
                .and_then(|poly| poly.as_array())
                .and_then(|rings| rings.first())
                .and_then(|ring| ring.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|c| {
                            let a = c.as_array()?;
                            Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?])
                        })
                        .collect()
                })
                .unwrap_or_default()
        }
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// Redis cache helpers
// ---------------------------------------------------------------------------

const TILE_CACHE_TTL: u64 = 3600; // 1 hour

async fn get_cached(
    redis: &RedisPool,
    z: u8, x: u32, y: u32,
) -> Option<TileRawData> {
    let key = format!("tile:{z}:{x}:{y}");
    let mut conn = redis.get().await.ok()?;
    let json: Option<String> = conn.get(&key).await.ok()?;
    json.and_then(|j| serde_json::from_str(&j).ok())
}

async fn set_cached(
    redis: &RedisPool,
    z: u8, x: u32, y: u32,
    data: &TileRawData,
) {
    let key = format!("tile:{z}:{x}:{y}");
    match serde_json::to_string(data) {
        Ok(json) => {
            if let Ok(mut conn) = redis.get().await {
                if let Err(e) = conn.set_ex::<_, _, ()>(&key, &json, TILE_CACHE_TTL).await {
                    tracing::warn!("Failed to cache tile {key}: {e}");
                }
            }
        }
        Err(e) => tracing::warn!("Failed to serialize tile {key}: {e}"),
    }
}

// ---------------------------------------------------------------------------
// Main service function
// ---------------------------------------------------------------------------

pub async fn get_tile(
    db: &PgPool,
    redis: &RedisPool,
    z: u8,
    x: u32,
    y: u32,
    center_lat: f64,
    center_lng: f64,
) -> Result<TileResponse, AppError> {
    // Validate zoom
    if !(15..=17).contains(&z) {
        return Err(AppError::Validation(
            "zoom must be 15, 16, or 17".into(),
        ));
    }

    // Check Redis cache
    let raw = if let Some(cached) = get_cached(redis, z, x, y).await {
        cached
    } else {
        // Compute bbox (west, south, east, north)
        let (w, s, e, n) = tile_to_bbox(z, x, y);

        // 4 parallel queries
        let (roads_res, water_res, parks_res, buildings_res) = tokio::join!(
            world_repo::get_roads(db, w, s, e, n, z),
            world_repo::get_water(db, w, s, e, n, z),
            world_repo::get_parks(db, w, s, e, n, z),
            world_repo::get_buildings(db, w, s, e, n, z),
        );

        let roads_rows = roads_res?;
        let water_rows = water_res?;
        let parks_rows = parks_res?;
        let buildings_rows = buildings_res?;

        // Convert to raw data (lat/lng coords)
        let raw = TileRawData {
            roads: roads_rows.iter().map(|r| RawRoad {
                coords: extract_coords(&r.coords_json),
                highway: r.highway.clone(),
                name: r.name.clone(),
                width: r.width_meters,
            }).collect(),
            water: water_rows.iter().map(|w| RawWater {
                coords: extract_coords(&w.coords_json),
                water_type: w.water_type.clone().unwrap_or_else(|| "water".into()),
                name: w.name.clone(),
            }).collect(),
            parks: parks_rows.iter().map(|p| RawPark {
                coords: extract_coords(&p.coords_json),
                park_type: p.park_type.clone().unwrap_or_else(|| "park".into()),
                name: p.name.clone(),
            }).collect(),
            buildings: buildings_rows.iter().map(|b| RawBuilding {
                coords: extract_coords(&b.coords_json),
                height: b.height_meters,
            }).collect(),
        };

        // Cache in Redis (fire and forget)
        set_cached(redis, z, x, y, &raw).await;
        raw
    };

    // Convert lat/lng to local meters
    let convert = |coords: &[[f64; 2]]| -> Vec<Point2D> {
        coords.iter().map(|[lng, lat]| {
            let (x, z) = geo_to_local(*lat, *lng, center_lat, center_lng);
            Point2D { x, z }
        }).collect()
    };

    Ok(TileResponse {
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
    })
}
```

- [ ] **Step 2: Register world_service in mod.rs**

Add to `crates/placebo-api/src/services/mod.rs`:

```rust
pub mod world_service;
```

- [ ] **Step 3: Verify compilation**

Run: `cd /Users/notebook/Placebo && cargo check -p placebo-api`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add crates/placebo-api/src/services/world_service.rs crates/placebo-api/src/services/mod.rs
git commit -m "feat(api): add world_service with coord conversion and Redis caching"
```

### Task 4: World handler – tile endpoint

**Files:**
- Create: `crates/placebo-api/src/handlers/world.rs`
- Modify: `crates/placebo-api/src/handlers/mod.rs`

- [ ] **Step 1: Write world.rs handler**

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
use crate::services::world_service;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tile", get(get_tile))
}

#[derive(Debug, Deserialize)]
pub struct TileParams {
    pub z: u8,
    pub x: u32,
    pub y: u32,
    pub center_lat: f64,
    pub center_lng: f64,
}

async fn get_tile(
    State(state): State<AppState>,
    Query(params): Query<TileParams>,
) -> Result<Json<world_service::TileResponse>, AppError> {
    let response = world_service::get_tile(
        &state.db,
        &state.redis,
        params.z,
        params.x,
        params.y,
        params.center_lat,
        params.center_lng,
    )
    .await?;

    Ok(Json(response))
}
```

- [ ] **Step 2: Register world handler in mod.rs**

Modify `crates/placebo-api/src/handlers/mod.rs` – add `pub mod world;` and nest `/world` route:

```rust
pub mod boosts;
pub mod cameras;
pub mod clips;
pub mod health;
pub mod ratings;
pub mod rooms;
pub mod users;
pub mod world;

use axum::Router;
use crate::app_state::AppState;

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

- [ ] **Step 3: Verify full API compiles**

Run: `cd /Users/notebook/Placebo && cargo check -p placebo-api`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add crates/placebo-api/src/handlers/world.rs crates/placebo-api/src/handlers/mod.rs
git commit -m "feat(api): add GET /api/v1/world/tile endpoint"
```

### Task 5: Test the tile endpoint manually

- [ ] **Step 1: Start the API server**

Run:
```bash
cd /Users/notebook/Placebo
# Ensure .env has DATABASE_URL=postgres://localhost/placebo and REDIS_URL
cargo run -p placebo-api &
```

- [ ] **Step 2: Test tile endpoint with curl**

Shibuya Crossing is at approximately lat=35.6595, lng=139.7004. At z=16, tile x≈57483, y≈25953.

Run:
```bash
curl -s "http://localhost:3000/api/v1/world/tile?z=16&x=57483&y=25953&center_lat=35.6595&center_lng=139.7004" | python3 -m json.tool | head -50
```

Expected: JSON with roads, water, parks, buildings arrays containing local-meter coordinates.

- [ ] **Step 3: Verify Redis caching**

Run the same curl again, check Redis:
```bash
redis-cli GET "tile:16:57483:25953" | head -c 200
```

Expected: Cached JSON string present.

---

## Chunk 3: Frontend Types & Hook

### Task 6: Move RoadSegment to types/world3d.ts and add new types

**Files:**
- Modify: `src/types/world3d.ts`
- Modify: `src/components/world3d/ground/RoadNetwork.tsx`
- Modify: `src/components/world3d/ground/GroundSystem.tsx`

- [ ] **Step 1: Add new types to world3d.ts**

Append to `src/types/world3d.ts` before the closing utilities section:

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

Change `import type { RoadSegment } from '../../../hooks/useRoadNetwork';`
to `import type { RoadSegment } from '../../../types/world3d';`

- [ ] **Step 3: Update GroundSystem.tsx import**

Change `import type { RoadSegment } from '../../../hooks/useRoadNetwork';`
to `import type { RoadSegment } from '../../../types/world3d';`

- [ ] **Step 3b: Update LightingSystem.tsx import**

In `src/components/world3d/lighting/LightingSystem.tsx`, change:
`import type { RoadSegment } from '../../../hooks/useRoadNetwork';`
to `import type { RoadSegment } from '../../../types/world3d';`

- [ ] **Step 3c: Update NightLights.tsx import**

In `src/components/world3d/lighting/NightLights.tsx`, change:
`import type { RoadSegment } from '../../../hooks/useRoadNetwork';`
to `import type { RoadSegment } from '../../../types/world3d';`

- [ ] **Step 4: Verify frontend compiles**

Run: `cd /Users/notebook/Placebo && npx tsc --noEmit`
Expected: No type errors.

- [ ] **Step 5: Commit**

```bash
git add src/types/world3d.ts src/components/world3d/ground/RoadNetwork.tsx src/components/world3d/ground/GroundSystem.tsx src/components/world3d/lighting/LightingSystem.tsx src/components/world3d/lighting/NightLights.tsx
git commit -m "feat(types): add RoadSegment, WaterFeature, ParkFeature, BuildingFootprint"
```

### Task 7: Create useCityTiles hook

**Files:**
- Create: `src/hooks/useCityTiles.ts`

- [ ] **Step 1: Write useCityTiles.ts**

Create `src/hooks/useCityTiles.ts`:

```typescript
import { useState, useEffect, useRef } from 'react';
import type {
  RoadSegment,
  WaterFeature,
  ParkFeature,
  BuildingFootprint,
} from '../types/world3d';

export interface CityTilesResult {
  roads: RoadSegment[];
  water: WaterFeature[];
  parks: ParkFeature[];
  buildings: BuildingFootprint[];
  loading: boolean;
  error: string | null;
}

// ─── Slippy Map tile math ───────────────────────────────────

function latLngToTile(lat: number, lng: number, zoom: number): { x: number; y: number } {
  const n = 2 ** zoom;
  const x = Math.floor(((lng + 180) / 360) * n);
  const latRad = (lat * Math.PI) / 180;
  const y = Math.floor(((1 - Math.log(Math.tan(latRad) + 1 / Math.cos(latRad)) / Math.PI) / 2) * n);
  return { x, y };
}

function getVisibleTiles(
  centerLat: number,
  centerLng: number,
  zoom: number
): { z: number; x: number; y: number }[] {
  const center = latLngToTile(centerLat, centerLng, zoom);
  const tiles: { z: number; x: number; y: number }[] = [];
  for (let dx = -1; dx <= 1; dx++) {
    for (let dy = -1; dy <= 1; dy++) {
      tiles.push({ z: zoom, x: center.x + dx, y: center.y + dy });
    }
  }
  return tiles;
}

// ─── API base URL ───────────────────────────────────────────

const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000';

async function fetchTile(
  z: number, x: number, y: number,
  centerLat: number, centerLng: number,
  signal: AbortSignal,
): Promise<{
  roads: RoadSegment[];
  water: WaterFeature[];
  parks: ParkFeature[];
  buildings: BuildingFootprint[];
}> {
  const url = `${API_BASE}/api/v1/world/tile?z=${z}&x=${x}&y=${y}&center_lat=${centerLat}&center_lng=${centerLng}`;
  const res = await fetch(url, { signal });
  if (!res.ok) throw new Error(`Tile fetch failed: ${res.status}`);
  return res.json();
}

// ─── Hook ───────────────────────────────────────────────────

export function useCityTiles(
  centerLat: number,
  centerLng: number,
  zoom: number = 16
): CityTilesResult {
  const [result, setResult] = useState<CityTilesResult>({
    roads: [], water: [], parks: [], buildings: [],
    loading: true, error: null,
  });

  const abortRef = useRef<AbortController | null>(null);

  useEffect(() => {
    // Abort previous fetch
    abortRef.current?.abort();
    const controller = new AbortController();
    abortRef.current = controller;

    const tiles = getVisibleTiles(centerLat, centerLng, zoom);

    setResult(prev => ({ ...prev, loading: true, error: null }));

    Promise.all(
      tiles.map(t => fetchTile(t.z, t.x, t.y, centerLat, centerLng, controller.signal))
    )
      .then(responses => {
        if (controller.signal.aborted) return;

        const roads: RoadSegment[] = [];
        const water: WaterFeature[] = [];
        const parks: ParkFeature[] = [];
        const buildings: BuildingFootprint[] = [];

        for (const r of responses) {
          roads.push(...r.roads);
          water.push(...r.water);
          parks.push(...r.parks);
          buildings.push(...r.buildings);
        }

        setResult({ roads, water, parks, buildings, loading: false, error: null });
      })
      .catch(err => {
        if (controller.signal.aborted) return;
        console.error('[useCityTiles] fetch error:', err);
        setResult(prev => ({ ...prev, loading: false, error: err.message }));
      });

    return () => controller.abort();
  }, [centerLat, centerLng, zoom]);

  return result;
}
```

- [ ] **Step 2: Verify useCityTiles compiles**

Run: `cd /Users/notebook/Placebo && npx tsc --noEmit`
Expected: Compiles (useRoadNetwork.ts still exists, WorldScene still imports it – updated in Chunk 5).

- [ ] **Step 3: Commit**

```bash
git add src/hooks/useCityTiles.ts
git commit -m "feat(hooks): add useCityTiles tile fetching hook"
```

---

## Chunk 4: R3F Components (Water, Parks, Buildings, DynamicFog)

### Task 8: WaterBodies component

**Files:**
- Create: `src/components/world3d/ground/WaterBodies.tsx`
- Modify: `src/components/world3d/ground/index.ts`

- [ ] **Step 1: Write WaterBodies.tsx**

Create `src/components/world3d/ground/WaterBodies.tsx`:

```tsx
import { useMemo } from 'react';
import * as THREE from 'three';
import type { WaterFeature } from '../../../types/world3d';

interface WaterBodiesProps {
  water: WaterFeature[];
}

const RIVER_WIDTH = 8; // meters

function tessellateRibbon(points: { x: number; z: number }[], width: number): Float32Array | null {
  if (points.length < 2) return null;
  const halfW = width / 2;
  const verts: number[] = [];

  for (let i = 0; i < points.length - 1; i++) {
    const curr = points[i];
    const next = points[i + 1];
    const dx = next.x - curr.x;
    const dz = next.z - curr.z;
    const len = Math.sqrt(dx * dx + dz * dz);
    if (len < 0.01) continue;

    const px = -dz / len;
    const pz = dx / len;

    verts.push(curr.x + px * halfW, 0.02, curr.z + pz * halfW);
    verts.push(curr.x - px * halfW, 0.02, curr.z - pz * halfW);
    verts.push(next.x + px * halfW, 0.02, next.z + pz * halfW);
    verts.push(curr.x - px * halfW, 0.02, curr.z - pz * halfW);
    verts.push(next.x - px * halfW, 0.02, next.z - pz * halfW);
    verts.push(next.x + px * halfW, 0.02, next.z + pz * halfW);
  }

  return verts.length > 0 ? new Float32Array(verts) : null;
}

function createPolygonGeometry(points: { x: number; z: number }[]): THREE.ShapeGeometry | null {
  if (points.length < 3) return null;

  const shape = new THREE.Shape();
  shape.moveTo(points[0].x, points[0].z);
  for (let i = 1; i < points.length; i++) {
    shape.lineTo(points[i].x, points[i].z);
  }
  shape.closePath();

  return new THREE.ShapeGeometry(shape);
}

export function WaterBodies({ water }: WaterBodiesProps) {
  const { polygonGeom, ribbonVertices } = useMemo(() => {
    const polygonGeometries: THREE.ShapeGeometry[] = [];
    const ribbonArrays: Float32Array[] = [];

    for (const feature of water) {
      // Heuristic: if first point == last point → polygon (lake)
      const pts = feature.points;
      const isClosed =
        pts.length >= 3 &&
        Math.abs(pts[0].x - pts[pts.length - 1].x) < 0.1 &&
        Math.abs(pts[0].z - pts[pts.length - 1].z) < 0.1;

      if (isClosed && pts.length >= 4) {
        const geom = createPolygonGeometry(pts);
        if (geom) polygonGeometries.push(geom);
      } else {
        const ribbon = tessellateRibbon(pts, RIVER_WIDTH);
        if (ribbon) ribbonArrays.push(ribbon);
      }
    }

    // Merge ribbon arrays
    let mergedRibbon: Float32Array | null = null;
    if (ribbonArrays.length > 0) {
      const totalLen = ribbonArrays.reduce((s, a) => s + a.length, 0);
      mergedRibbon = new Float32Array(totalLen);
      let offset = 0;
      for (const arr of ribbonArrays) {
        mergedRibbon.set(arr, offset);
        offset += arr.length;
      }
    }

    return {
      polygonGeom: polygonGeometries,
      ribbonVertices: mergedRibbon,
    };
  }, [water]);

  if (water.length === 0) return null;

  return (
    <group>
      {/* Polygon water (lakes) – rendered as flat ShapeGeometry */}
      {polygonGeom.map((geom, i) => (
        <mesh key={`wp-${i}`} rotation={[-Math.PI / 2, 0, 0]} position={[0, 0.02, 0]}>
          <primitive object={geom} attach="geometry" />
          <meshBasicMaterial
            color="#0a1a3a"
            opacity={0.4}
            transparent
            depthWrite={false}
            side={THREE.DoubleSide}
          />
        </mesh>
      ))}

      {/* Line water (rivers) – ribbon mesh */}
      {ribbonVertices && (
        <mesh>
          <bufferGeometry>
            <bufferAttribute
              attach="attributes-position"
              array={ribbonVertices}
              count={ribbonVertices.length / 3}
              itemSize={3}
            />
          </bufferGeometry>
          <meshBasicMaterial
            color="#0a1a3a"
            opacity={0.4}
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

- [ ] **Step 2: Export from index.ts**

Add to `src/components/world3d/ground/index.ts`:

```typescript
export { WaterBodies } from './WaterBodies';
```

- [ ] **Step 3: Commit**

```bash
git add src/components/world3d/ground/WaterBodies.tsx src/components/world3d/ground/index.ts
git commit -m "feat(world3d): add WaterBodies component for lakes and rivers"
```

### Task 9: Parks component

**Files:**
- Create: `src/components/world3d/ground/Parks.tsx`
- Modify: `src/components/world3d/ground/index.ts`

- [ ] **Step 1: Write Parks.tsx**

Create `src/components/world3d/ground/Parks.tsx`:

```tsx
import { useMemo } from 'react';
import * as THREE from 'three';
import type { ParkFeature } from '../../../types/world3d';

interface ParksProps {
  parks: ParkFeature[];
}

export function Parks({ parks }: ParksProps) {
  const geometries = useMemo(() => {
    const result: THREE.ShapeGeometry[] = [];

    for (const park of parks) {
      const pts = park.points;
      if (pts.length < 3) continue;

      const shape = new THREE.Shape();
      shape.moveTo(pts[0].x, pts[0].z);
      for (let i = 1; i < pts.length; i++) {
        shape.lineTo(pts[i].x, pts[i].z);
      }
      shape.closePath();

      result.push(new THREE.ShapeGeometry(shape));
    }

    return result;
  }, [parks]);

  if (geometries.length === 0) return null;

  return (
    <group>
      {geometries.map((geom, i) => (
        <mesh key={`park-${i}`} rotation={[-Math.PI / 2, 0, 0]} position={[0, 0.01, 0]}>
          <primitive object={geom} attach="geometry" />
          <meshBasicMaterial
            color="#0a2a0a"
            opacity={0.3}
            transparent
            depthWrite={false}
            side={THREE.DoubleSide}
          />
        </mesh>
      ))}
    </group>
  );
}
```

- [ ] **Step 2: Export from index.ts**

Add to `src/components/world3d/ground/index.ts`:

```typescript
export { Parks } from './Parks';
```

- [ ] **Step 3: Commit**

```bash
git add src/components/world3d/ground/Parks.tsx src/components/world3d/ground/index.ts
git commit -m "feat(world3d): add Parks component for green areas"
```

### Task 10: Rewrite BuildingsLayer with real footprints

**Files:**
- Modify: `src/components/world3d/BuildingsLayer.tsx`

- [ ] **Step 1: Rewrite BuildingsLayer.tsx**

Replace entire contents of `src/components/world3d/BuildingsLayer.tsx`:

```tsx
import { useMemo, useEffect, useRef } from 'react';
import * as THREE from 'three';
import type { BuildingFootprint } from '../../types/world3d';

interface BuildingsLayerProps {
  buildings: BuildingFootprint[];
}

/**
 * BuildingsLayer – extruded wireframe buildings from real OSM footprints.
 *
 * Each building:
 *   outline → THREE.Shape → ExtrudeGeometry → glass fill + edge wireframe
 *
 * Visual style: glass-like wireframe (dark fill + visible edges).
 * NOTE: Performance TODO – batch into merged BufferGeometry for large counts.
 */
export function BuildingsLayer({ buildings }: BuildingsLayerProps) {
  const prevFills = useRef<THREE.ExtrudeGeometry[]>([]);
  const prevEdges = useRef<THREE.EdgesGeometry[]>([]);

  const { fills, edges } = useMemo(() => {
    const fillGeometries: THREE.ExtrudeGeometry[] = [];
    const edgeGeometries: THREE.EdgesGeometry[] = [];

    for (const building of buildings) {
      const pts = building.outline;
      if (pts.length < 3) continue;

      const shape = new THREE.Shape();
      shape.moveTo(pts[0].x, pts[0].z);
      for (let i = 1; i < pts.length; i++) {
        shape.lineTo(pts[i].x, pts[i].z);
      }
      shape.closePath();

      const extruded = new THREE.ExtrudeGeometry(shape, {
        depth: building.height,
        bevelEnabled: false,
      });

      // Rotate so extrusion goes up (Y axis) instead of Z
      extruded.rotateX(-Math.PI / 2);

      fillGeometries.push(extruded);
      edgeGeometries.push(new THREE.EdgesGeometry(extruded));
    }

    return { fills: fillGeometries, edges: edgeGeometries };
  }, [buildings]);

  // Dispose old geometries on change
  useEffect(() => {
    // Dispose previous
    prevFills.current.forEach(g => g.dispose());
    prevEdges.current.forEach(g => g.dispose());
    // Store current for next cleanup
    prevFills.current = fills;
    prevEdges.current = edges;

    return () => {
      fills.forEach(g => g.dispose());
      edges.forEach(g => g.dispose());
    };
  }, [fills, edges]);

  if (fills.length === 0) return null;

  return (
    <group>
      {fills.map((geom, i) => (
        <group key={`bld-${i}`}>
          {/* Glass fill */}
          <mesh geometry={geom}>
            <meshBasicMaterial
              color="#0a0f18"
              opacity={0.06}
              transparent
              side={THREE.DoubleSide}
              depthWrite={false}
            />
          </mesh>
          {/* Edge wireframe */}
          <lineSegments geometry={edges[i]}>
            <lineBasicMaterial
              color="#1e2840"
              opacity={0.4}
              transparent
            />
          </lineSegments>
        </group>
      ))}
    </group>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/world3d/BuildingsLayer.tsx
git commit -m "feat(world3d): rewrite BuildingsLayer with real OSM footprints"
```

### Task 11: DynamicFog component

**Files:**
- Create: `src/components/world3d/DynamicFog.tsx`

- [ ] **Step 1: Write DynamicFog.tsx**

Create `src/components/world3d/DynamicFog.tsx`:

```tsx
import { useRef } from 'react';
import { useFrame, useThree } from '@react-three/fiber';
import * as THREE from 'three';
import { useTimeOfDay } from '../../hooks/useTimeOfDay';

interface DynamicFogProps {
  timezone: string;
  near: number;
  far: number;
}

const FOG_COLORS: Record<string, string> = {
  night:    '#050510',
  dawn:     '#1a1020',
  morning:  '#2a4060',
  day:      '#4a6a8a',
  dusk:     '#3a2020',
  twilight: '#0a0a1a',
};

/**
 * DynamicFog — updates scene.fog color based on time of day.
 * Smooth lerp transition between phases.
 */
export function DynamicFog({ timezone, near, far }: DynamicFogProps) {
  const { scene } = useThree();
  const { phase } = useTimeOfDay(timezone);
  const targetColor = useRef(new THREE.Color(FOG_COLORS[phase] || '#0a0a1a'));
  const currentColor = useRef(new THREE.Color(FOG_COLORS[phase] || '#0a0a1a'));

  // Update target when phase changes
  const prevPhase = useRef(phase);
  if (prevPhase.current !== phase) {
    targetColor.current.set(FOG_COLORS[phase] || '#0a0a1a');
    prevPhase.current = phase;
  }

  useFrame(() => {
    // Smooth lerp
    currentColor.current.lerp(targetColor.current, 0.02);

    if (!scene.fog) {
      scene.fog = new THREE.Fog(currentColor.current, near, far);
    } else {
      (scene.fog as THREE.Fog).color.copy(currentColor.current);
      (scene.fog as THREE.Fog).near = near;
      (scene.fog as THREE.Fog).far = far;
    }
  });

  return null;
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/world3d/DynamicFog.tsx
git commit -m "feat(world3d): add DynamicFog with time-of-day color transitions"
```

---

## Chunk 5: Integration – WorldScene + GroundSystem Wiring

### Task 12: Update GroundSystem to accept water and parks

**Files:**
- Modify: `src/components/world3d/ground/GroundSystem.tsx`

- [ ] **Step 1: Update GroundSystem.tsx**

Replace contents of `src/components/world3d/ground/GroundSystem.tsx`:

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

- [ ] **Step 2: Commit**

```bash
git add src/components/world3d/ground/GroundSystem.tsx
git commit -m "feat(world3d): wire water and parks into GroundSystem"
```

### Task 13: Update WorldScene to use useCityTiles and DynamicFog

**Files:**
- Modify: `src/components/world3d/WorldScene.tsx`

- [ ] **Step 1: Rewrite WorldScene.tsx**

Replace contents of `src/components/world3d/WorldScene.tsx`:

```tsx
import { Suspense, useState, useCallback, useRef } from 'react';
import { Canvas } from '@react-three/fiber';
import { Stats } from '@react-three/drei';
import type { Camera3D, QualityConfig } from '../../types/world3d';
import { DEFAULT_QUALITY, geoToLocal } from '../../types/world3d';
import { BuildingsLayer } from './BuildingsLayer';
import { CameraMarker3D } from './CameraMarker3D';
import { CameraFrustum } from './CameraFrustum';
import { SkySystem } from './sky';
import { GroundSystem } from './ground';
import { LightingSystem } from './lighting';
import { PostStack } from './post';
import { DynamicFog } from './DynamicFog';
import { QualityContext } from '../../hooks/useQuality';
import { useCityTiles } from '../../hooks/useCityTiles';
import { NavigationControls } from './NavigationControls';

interface WorldSceneProps {
  /** Активная камера (та что пользователь смотрит) */
  activeCamera: Camera3D;
  /** Список соседних камер (для маркеров) */
  nearbyCameras: Camera3D[];
  /** Колбэк при клике на маркер соседней камеры */
  onCameraSelect: (camera: Camera3D) => void;
  /** Часовой пояс для освещения (день/ночь) */
  timezone?: string;
  /** Настройки качества */
  quality?: QualityConfig;
  /** Показывать ли FPS-счётчик (dev mode) */
  showStats?: boolean;
}

/**
 * WorldScene — корневой компонент 3D-мира Placebo.
 *
 * Рендерит:
 * - Реальные дороги, воду, парки, здания из OSM (через tile API)
 * - Видеопоток активной камеры на плоскости
 * - Маркеры соседних камер
 * - Wireframe-эффект для зданий
 * - Освещение (день/ночь по timezone)
 */
export function WorldScene({
  activeCamera,
  nearbyCameras,
  onCameraSelect,
  timezone = 'UTC',
  quality = DEFAULT_QUALITY,
  showStats = false,
}: WorldSceneProps) {
  const [isExploring, setIsExploring] = useState(false);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const handleExplorationStart = useCallback(() => {
    setIsExploring(true);
  }, []);

  const { roads, water, parks, buildings } = useCityTiles(
    activeCamera.lat, activeCamera.lng, 16
  );

  return (
    <Canvas
      ref={canvasRef}
      camera={{
        fov: 75,
        near: 0.1,
        far: 5000,
        position: [0, activeCamera.heightAboveGround, 0],
      }}
      gl={{
        antialias: true,
        alpha: false,
        powerPreference: 'high-performance',
        stencil: quality.post.mode !== 'none',
      }}
      dpr={[1, 2]}
      style={{
        position: 'absolute',
        top: 0,
        left: 0,
        width: '100%',
        height: '100%',
        background: '#0a0a0f',
      }}
    >
      <QualityContext.Provider value={quality}>
        {showStats && <Stats />}

        <SkySystem timezone={timezone} />
        <LightingSystem timezone={timezone} roads={roads} />
        <DynamicFog timezone={timezone} near={quality.fog.near} far={quality.fog.far} />

        <Suspense fallback={null}>
          <GroundSystem roads={roads} water={water} parks={parks} />

          <BuildingsLayer buildings={buildings} />

          <CameraFrustum camera={activeCamera} showVideo={true} />

          {nearbyCameras
            .filter((cam) => cam.id !== activeCamera.id)
            .map((cam) => (
              <CameraMarker3D
                key={cam.id}
                camera={cam}
                centerCamera={activeCamera}
                onClick={() => onCameraSelect(cam)}
              />
            ))}

          {nearbyCameras
            .filter((cam) => cam.id !== activeCamera.id && cam.hlsUrl)
            .slice(0, quality.maxVideoTextures - 1)
            .map((cam) => {
              const { x, z } = geoToLocal(cam.lat, cam.lng, activeCamera.lat, activeCamera.lng);
              return (
                <group key={`frustum-${cam.id}`} position={[x, cam.heightAboveGround, z]}>
                  <CameraFrustum camera={cam} showVideo={true} frustumDepth={60} />
                </group>
              );
            })}
        </Suspense>

        <PostStack />

        <NavigationControls
          onExplorationStart={handleExplorationStart}
          isExploring={isExploring}
        />
      </QualityContext.Provider>
    </Canvas>
  );
}
```

Note: removed `tilesUrl` prop (no longer needed – BuildingsLayer uses footprints from tiles).

- [ ] **Step 2: Update World3DScreen.tsx – remove tilesUrl prop**

In `src/screens/World3DScreen.tsx`, remove the `tilesUrl=""` prop from `<WorldScene />`:

Change:
```tsx
<WorldScene
  activeCamera={activeCamera}
  nearbyCameras={cameras}
  onCameraSelect={handleCameraSelect}
  timezone="Asia/Tokyo"
  tilesUrl="" // Пока нет 3D Tiles, BuildingsLayer рисует mock здания
  showStats={true}
/>
```

To:
```tsx
<WorldScene
  activeCamera={activeCamera}
  nearbyCameras={cameras}
  onCameraSelect={handleCameraSelect}
  timezone="Asia/Tokyo"
  showStats={true}
/>
```

- [ ] **Step 3: Delete useRoadNetwork.ts (now safe – all imports updated)**

Run: `rm src/hooks/useRoadNetwork.ts`

- [ ] **Step 4: Verify frontend compiles**

Run: `cd /Users/notebook/Placebo && npx tsc --noEmit`
Expected: No type errors.

- [ ] **Step 5: Commit**

```bash
git add src/components/world3d/WorldScene.tsx src/components/world3d/DynamicFog.tsx src/screens/World3DScreen.tsx
git rm src/hooks/useRoadNetwork.ts
git commit -m "feat(world3d): integrate useCityTiles, DynamicFog, remove tilesUrl prop"
```

### Task 14: End-to-end verification

- [ ] **Step 1: Start the API server**

Run:
```bash
cd /Users/notebook/Placebo && cargo run -p placebo-api &
```

- [ ] **Step 2: Start the frontend dev server**

Run:
```bash
cd /Users/notebook/Placebo && VITE_API_URL=http://localhost:3000 npm run dev
```

- [ ] **Step 3: Verify in browser**

Open `http://localhost:1420`. Check:
1. Roads appear as white ribbons on the ground
2. Water bodies appear as dark blue areas
3. Parks appear as dark green areas
4. Buildings appear as wireframe glass extrusions with real heights
5. Camera frustums still show video
6. Fog color transitions smoothly

- [ ] **Step 4: Check browser console for errors**

Open DevTools → Console. Expected: no fetch errors, no Three.js warnings.

- [ ] **Step 5: Verify tile API performance**

Run:
```bash
curl -w "\nTime: %{time_total}s\n" -s "http://localhost:3000/api/v1/world/tile?z=16&x=57483&y=25953&center_lat=35.6595&center_lng=139.7004" -o /dev/null
```

Expected: < 200ms first request, < 10ms cached.

- [ ] **Step 6: Final commit with any remaining fixes**

```bash
git add -A
git commit -m "feat: complete OSM tile pipeline integration"
```
