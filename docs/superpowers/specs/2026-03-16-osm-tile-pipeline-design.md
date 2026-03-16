# OSM Tile Pipeline – Design Spec

**Date**: 2026-03-16
**Status**: Approved
**Scope**: Real OSM data (roads, water, parks, buildings) from PostGIS to 3D world via tile-based API

---

## 1. Overview

Deliver real OpenStreetMap geometry (roads, water bodies, parks, building footprints) from PostGIS to the Placebo 3D world. The server does all heavy lifting – coordinate conversion, geometry simplification, layer filtering. The client receives ready-to-render JSON and builds Three.js geometry directly.

### Data flow

```
Geofabrik PBF → osm2pgsql → PostGIS (planet_osm_line / polygon)
                                ↓
                          SQL Views (roads, water, parks, buildings_3d)
                                ↓
                  Axum handler: GET /api/v1/world/tile
                  - Computes tile bbox from z/x/y
                  - 4 parallel SQL queries (tokio::join!)
                  - ST_SimplifyPreserveTopology (zoom-dependent)
                  - Converts lat/lng → local meters (center_lat/lng)
                  - Returns JSON
                                ↓
                  Frontend: useCityTiles hook
                  - Computes visible tiles for viewport
                  - Fetches tiles in parallel
                  - Merges into unified arrays
                  - Passes to R3F components
```

### Key decisions

- **Tile-based JSON API** (not MVT protobuf) – server converts to local meters, client just renders
- **PostGIS native** on macOS (not Docker) – single `placebo` database for API + OSM data
- **osm2pgsql** with `full.style` imports roads, water, parks, buildings, landuse
- **No client-side caching** – all processing on server, Redis caches tile responses (1h TTL)
- **Zoom 15-17** – z16 default (~600m per tile), 3×3 grid = ~1.8km viewport

---

## 2. PostGIS – SQL Views

OSM data lives in `planet_osm_line` (1.6M rows) and `planet_osm_polygon` (1.9M rows) for the Kanto region. Spatial GIST indexes make bbox queries ~5ms.

### 2.1 roads_view

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

### 2.2 water_view

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

### 2.3 parks_view

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

### 2.4 buildings_tile_view (new, for tile endpoint)

The existing `buildings_3d` view uses `ST_Force3D` and aliases geometry as `geom` (designed for pg2b3dm). For the tile endpoint we need a simpler 2D view that queries `planet_osm_polygon` directly:

```sql
CREATE OR REPLACE VIEW buildings_tile_view AS
SELECT
  osm_id,
  way,
  CASE
    WHEN height ~ '^\d+(\.\d+)?$' THEN height::float
    WHEN "building:levels" ~ '^\d+$' THEN "building:levels"::int * 3.0
    ELSE 9.0  -- default 3 floors
  END AS height_meters,
  name
FROM planet_osm_polygon
WHERE building IS NOT NULL
  AND building NOT IN ('no', 'entrance')
  AND ST_Area(way::geography) > 10;  -- 10 sq meters, geography-based
```

Note: `ST_Area(way::geography)` computes area in square meters even though `way` is in SRID 4326 (degrees). The existing `buildings_3d` view uses `ST_Area(way) > 10` which is wrong for latlong projection – this new view fixes that.

### 2.5 Indexes

osm2pgsql creates GIST indexes on `way` automatically. Add partial indexes for filtered queries:

```sql
CREATE INDEX IF NOT EXISTS idx_line_highway ON planet_osm_line (highway) WHERE highway IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_polygon_building ON planet_osm_polygon (building) WHERE building IS NOT NULL AND building != 'no';
CREATE INDEX IF NOT EXISTS idx_polygon_water ON planet_osm_polygon (water) WHERE water IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_polygon_leisure ON planet_osm_polygon (leisure) WHERE leisure IS NOT NULL;
```

**File locations:**
- `pipeline/sql/03_roads_view.sql`
- `pipeline/sql/04_water_view.sql`
- `pipeline/sql/05_parks_view.sql`
- `pipeline/sql/06_indexes.sql`

---

## 3. Axum API – Tile Endpoint

### 3.1 Endpoint

```
GET /api/v1/world/tile?z={zoom}&x={x}&y={y}&center_lat={lat}&center_lng={lng}
```

**Parameters:**
| Param | Type | Required | Description |
|-------|------|----------|-------------|
| z | u8 | yes | Zoom level (15-17) |
| x | u32 | yes | Tile X (Slippy Map) |
| y | u32 | yes | Tile Y (Slippy Map) |
| center_lat | f64 | yes | Camera center latitude (for local meter conversion) |
| center_lng | f64 | yes | Camera center longitude |

### 3.2 Handler logic

1. **Validate** z in 15..=17, compute tile bbox from z/x/y using Slippy Map math
2. **4 parallel queries** via `tokio::join!`:
   - `SELECT FROM roads_view WHERE ST_Intersects(way, bbox)`
   - `SELECT FROM water_view WHERE ST_Intersects(way, bbox)`
   - `SELECT FROM parks_view WHERE ST_Intersects(way, bbox)`
   - `SELECT FROM buildings_tile_view WHERE ST_Intersects(way, bbox)`
3. **Simplify** geometry: `ST_SimplifyPreserveTopology(way, tolerance)`
   - z17: tolerance = 0.00001 (~1m)
   - z16: tolerance = 0.00005 (~5m)
   - z15: tolerance = 0.0001 (~10m)
4. **Convert** each coordinate from lat/lng to local meters:
   ```rust
   fn geo_to_local(lat: f64, lng: f64, center_lat: f64, center_lng: f64) -> (f64, f64) {
       let x = (lng - center_lng) * center_lat.to_radians().cos() * 111320.0;
       let z = (lat - center_lat) * 111320.0;
       (x, z)
   }
   ```
5. **Return** JSON response

### 3.3 Response format

```json
{
  "tile": { "z": 16, "x": 57483, "y": 25953 },
  "roads": [
    {
      "points": [{"x": -45.2, "z": 12.8}, {"x": -40.1, "z": 15.3}],
      "highway": "primary",
      "name": "Meiji-dori",
      "width": 12
    }
  ],
  "water": [
    {
      "points": [{"x": 120.0, "z": -80.0}, {"x": 125.0, "z": -75.0}],
      "type": "river",
      "name": "Shibuya River"
    }
  ],
  "parks": [
    {
      "points": [{"x": 200.0, "z": 150.0}, {"x": 210.0, "z": 155.0}],
      "type": "park",
      "name": "Miyashita Park"
    }
  ],
  "buildings": [
    {
      "outline": [{"x": 10.0, "z": 5.0}, {"x": 15.0, "z": 5.0}, {"x": 15.0, "z": 10.0}],
      "height": 45.0
    }
  ]
}
```

### 3.4 Caching

- **Redis**: key `tile:{z}:{x}:{y}`, TTL 3600s (1 hour)
- center_lat/lng NOT part of cache key – tile geometry is absolute, conversion happens after cache hit
- Cache the raw PostGIS results (in lat/lng), convert to local meters per-request

**Cached structure** (Rust struct `TileRawData`, serialized as JSON in Redis):
```rust
#[derive(Serialize, Deserialize)]
struct TileRawData {
    roads: Vec<RawRoad>,      // points as lat/lng
    water: Vec<RawWater>,
    parks: Vec<RawPark>,
    buildings: Vec<RawBuilding>,
}

#[derive(Serialize, Deserialize)]
struct RawRoad {
    coords: Vec<[f64; 2]>,  // [lng, lat] pairs (GeoJSON order)
    highway: String,
    name: Option<String>,
    width: f64,
}
// Similar for RawWater, RawPark, RawBuilding
```

Flow: PostGIS query → `TileRawData` → serialize to Redis. On cache hit: deserialize → convert coords to local meters using request's center_lat/lng → return API response.

### 3.5 Rate limiting

- Max 9 tiles per request batch (3×3 viewport)
- Existing Redis rate limiter applies per-user

### 3.6 Compression

- `tower-http::CompressionLayer` with gzip
- Typical tile z16: ~80KB raw JSON → ~15KB gzipped

### 3.7 File locations

- `crates/placebo-api/src/handlers/world.rs` – handler + query params
- `crates/placebo-api/src/repos/world.rs` – SQL queries, PostGIS interaction
- `crates/placebo-api/src/services/world.rs` – coordinate conversion, response assembly
- `crates/placebo-api/src/routes.rs` – add `/world` routes

---

## 4. Frontend – useCityTiles Hook

Replaces `useRoadNetwork`. Single hook for all city geometry layers.

### 4.1 Interface

```typescript
// src/hooks/useCityTiles.ts

interface CityTilesResult {
  roads: RoadSegment[];
  water: WaterFeature[];
  parks: ParkFeature[];
  buildings: BuildingFootprint[];
  loading: boolean;
  error: string | null;
}

function useCityTiles(
  centerLat: number,
  centerLng: number,
  zoom?: number  // default 16
): CityTilesResult
```

### 4.2 Logic

1. Compute visible tile coordinates for 3×3 grid around center
2. Fetch each tile from `/api/v1/world/tile?z=..&x=..&y=..&center_lat=..&center_lng=..`
3. Merge all tile responses into unified arrays (dedup by checking tile boundaries)
4. When active camera changes → recompute visible tiles → fetch new ones, drop old
5. Use `AbortController` for cleanup on unmount

### 4.3 Types (add to types/world3d.ts)

Move `RoadSegment` and `DEFAULT_WIDTHS` from `useRoadNetwork.ts` to `types/world3d.ts` (since `RoadNetwork.tsx` and other components import it). Then add new types:

```typescript
// Moved from useRoadNetwork.ts
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
  type: 'river' | 'lake' | 'canal' | 'stream' | 'water' | 'wetland' | 'drain';
  name: string | null;
}

export interface ParkFeature {
  points: { x: number; z: number }[];
  type: 'park' | 'forest' | 'garden' | 'grass' | 'wood' | 'scrub' | 'grassland' | 'playground' | 'pitch' | 'meadow' | 'cemetery' | 'recreation_ground';
  name: string | null;
}

export interface BuildingFootprint {
  outline: { x: number; z: number }[];
  height: number;
}
```

### 4.4 Delete

- `src/hooks/useRoadNetwork.ts` – fully replaced by useCityTiles

---

## 5. R3F Components

### 5.1 WaterBodies – `src/components/world3d/ground/WaterBodies.tsx`

- Input: `WaterFeature[]`
- Polygon water (lakes): `THREE.Shape` → `ShapeGeometry` → flat mesh at y=0.02
- Line water (rivers/streams): ribbon mesh (same technique as RoadNetwork)
- Material: `meshBasicMaterial`, color `#0a1a3a`, opacity 0.4, transparent, depthWrite false
- River width: 8m default

### 5.2 Parks – `src/components/world3d/ground/Parks.tsx`

- Input: `ParkFeature[]`
- All polygons: `THREE.Shape` → `ShapeGeometry` → flat mesh at y=0.01
- Material: `meshBasicMaterial`, color `#0a2a0a`, opacity 0.3, transparent, depthWrite false

### 5.3 BuildingsLayer (rewrite) – `src/components/world3d/BuildingsLayer.tsx`

- Input: `BuildingFootprint[]` (from useCityTiles, not from 3D Tiles)
- For each building:
  1. `outline` points → `THREE.Shape`
  2. `ExtrudeGeometry(shape, { depth: height, bevelEnabled: false })`
  3. Two render layers:
     - **Fill**: `meshBasicMaterial`, color `#0a0f18`, opacity 0.06, transparent, side DoubleSide
     - **Edges**: `EdgesGeometry(extruded)` → `LineSegments`, color `#1e2840`, opacity 0.4
- Result: glass-like wireframe buildings with real footprints and heights
- Performance: batch all buildings into single BufferGeometry where possible (instancing for similar heights)

### 5.4 GroundSystem (update)

```tsx
export function GroundSystem({ roads, water, parks }) {
  const { ground } = useQuality();
  return (
    <group>
      <GroundPlane />
      {ground.gridEnabled && <GroundGrid />}
      <RoadNetwork roads={roads} />
      <WaterBodies water={water} />
      <Parks parks={parks} />
    </group>
  );
}
```

### 5.5 DynamicFog – `src/components/world3d/DynamicFog.tsx`

- Uses `useTimeOfDay(timezone)` to get current phase
- Updates `scene.fog.color` via `useFrame` with smooth lerp
- Fog colors per phase:
  - night: `#050510`
  - dawn: `#1a1020`
  - morning: `#2a4060`
  - day: `#4a6a8a`
  - dusk: `#3a2020`
  - twilight: `#0a0a1a`
- Near/far from QualityConfig

---

## 6. WorldScene Integration

### Updated WorldScene.tsx

```tsx
const { roads, water, parks, buildings, loading } = useCityTiles(
  activeCamera.lat, activeCamera.lng, 16
);

<Canvas>
  <QualityContext.Provider value={quality}>
    {showStats && <Stats />}

    <SkySystem timezone={timezone} />
    <DynamicFog timezone={timezone} near={quality.fog.near} far={quality.fog.far} />
    <LightingSystem timezone={timezone} roads={roads} />

    <Suspense fallback={null}>
      <GroundSystem roads={roads} water={water} parks={parks} />
      <BuildingsLayer buildings={buildings} />

      <CameraFrustum camera={activeCamera} showVideo={true} />
      {nearbyCameras...}
    </Suspense>

    <PostStack />
    <NavigationControls ... />
  </QualityContext.Provider>
</Canvas>
```

### Changes to BuildingsLayer props

Old: `{ tilesUrl, centerLat, centerLng, activeCamera }`
New: `{ buildings: BuildingFootprint[] }`

---

## 7. File Change Summary

### New files
| File | Purpose |
|------|---------|
| `pipeline/sql/03_roads_view.sql` | Roads SQL view |
| `pipeline/sql/04_water_view.sql` | Water SQL view |
| `pipeline/sql/05_parks_view.sql` | Parks SQL view |
| `pipeline/sql/06_indexes.sql` | Partial indexes for query performance |
| `crates/placebo-api/src/handlers/world.rs` | Tile endpoint handler |
| `crates/placebo-api/src/repos/world.rs` | PostGIS queries |
| `crates/placebo-api/src/services/world.rs` | Coordinate conversion, response assembly |
| `src/hooks/useCityTiles.ts` | Frontend hook for tile fetching |
| `src/components/world3d/ground/WaterBodies.tsx` | Water rendering |
| `src/components/world3d/ground/Parks.tsx` | Parks rendering |
| `src/components/world3d/DynamicFog.tsx` | Time-based fog color |

### Modified files
| File | Change |
|------|--------|
| `src/types/world3d.ts` | Add WaterFeature, ParkFeature, BuildingFootprint types |
| `src/components/world3d/BuildingsLayer.tsx` | Rewrite: wireframe from real footprints |
| `src/components/world3d/ground/GroundSystem.tsx` | Add water + parks props/children |
| `src/components/world3d/ground/index.ts` | Export WaterBodies, Parks |
| `src/components/world3d/WorldScene.tsx` | useCityTiles, DynamicFog, new BuildingsLayer |
| `crates/placebo-api/src/handlers/mod.rs` | Add world module to api_router() |
| `crates/placebo-api/src/handlers/mod.rs` | Add world module |

### Deleted files
| File | Reason |
|------|--------|
| `src/hooks/useRoadNetwork.ts` | Replaced by useCityTiles |

---

## 8. Infrastructure Notes

### Local dev setup (already done)
- PostgreSQL 17 native (Homebrew), running on default port 5432
- PostGIS 3.6.2 extension enabled
- Database: `placebo` (shared with API tables)
- OSM data: Kanto region (438MB PBF), imported via osm2pgsql with `full.style`
- Tables: `planet_osm_line` (1.6M rows), `planet_osm_polygon` (1.9M rows)
- Shibuya area: 852 roads, 2606 buildings, 4 water features, 23 parks

### Production path
- Same PostgreSQL + PostGIS on VPS
- Redis for tile caching (already in axum stack)
- gzip compression via tower-http
- Future: PMTiles on Cloudflare R2 for scale beyond single-server PostGIS
