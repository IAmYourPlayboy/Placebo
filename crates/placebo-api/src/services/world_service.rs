use deadpool_redis::Pool as RedisPool;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AppError;
use crate::repositories::world_repo::{
    self, BuildingRow, ParkRow, RoadRow, WaterRow,
};

// ---------------------------------------------------------------------------
// Feature types for raw cache (lat/lng coords)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadFeatureRaw {
    pub osm_id: i64,
    pub highway: String,
    pub name: Option<String>,
    pub width_meters: f64,
    /// Coords as [[lng, lat], ...] – LineString points
    pub coords: Vec<[f64; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterFeatureRaw {
    pub osm_id: i64,
    #[serde(rename = "geomType")]
    pub geom_type: String,
    #[serde(rename = "type")]
    pub water_type: String,
    pub name: Option<String>,
    /// Coords as [[lng, lat], ...] – polygon ring or linestring points
    pub coords: Vec<[f64; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkFeatureRaw {
    pub osm_id: i64,
    pub park_type: String,
    pub name: Option<String>,
    pub coords: Vec<[f64; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingFeatureRaw {
    pub osm_id: i64,
    pub height_meters: f64,
    pub name: Option<String>,
    pub coords: Vec<[f64; 2]>,
}

/// Cached tile data – stores raw lat/lng so the cache is reusable for any center point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileRawData {
    pub roads: Vec<RoadFeatureRaw>,
    pub water: Vec<WaterFeatureRaw>,
    pub parks: Vec<ParkFeatureRaw>,
    pub buildings: Vec<BuildingFeatureRaw>,
}

// ---------------------------------------------------------------------------
// Response types (local-meter coordinates)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct RoadFeature {
    pub osm_id: i64,
    pub highway: String,
    pub name: Option<String>,
    pub width_meters: f64,
    /// Local-meter coords: [[x, z], ...]
    pub coords: Vec<[f64; 2]>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WaterFeature {
    pub osm_id: i64,
    #[serde(rename = "geomType")]
    pub geom_type: String,
    #[serde(rename = "type")]
    pub water_type: String,
    pub name: Option<String>,
    pub coords: Vec<[f64; 2]>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParkFeature {
    pub osm_id: i64,
    pub park_type: String,
    pub name: Option<String>,
    pub coords: Vec<[f64; 2]>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildingFeature {
    pub osm_id: i64,
    pub height_meters: f64,
    pub name: Option<String>,
    pub coords: Vec<[f64; 2]>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TileResponse {
    pub roads: Vec<RoadFeature>,
    pub water: Vec<WaterFeature>,
    pub parks: Vec<ParkFeature>,
    pub buildings: Vec<BuildingFeature>,
}

// ---------------------------------------------------------------------------
// Coordinate conversion
// ---------------------------------------------------------------------------

/// Convert geographic lat/lng to local-meter coordinates relative to a center.
/// Returns (x, z) in meters where +x = east, +z = north.
pub fn geo_to_local(lat: f64, lng: f64, center_lat: f64, center_lng: f64) -> (f64, f64) {
    let center_lat_rad = center_lat.to_radians();
    let x = (lng - center_lng) * center_lat_rad.cos() * 111_320.0;
    let z = (lat - center_lat) * 111_320.0;
    (x, z)
}

// ---------------------------------------------------------------------------
// GeoJSON parsing
// ---------------------------------------------------------------------------

/// Extract a flat list of [lng, lat] coordinate pairs from a GeoJSON geometry string.
/// Handles: LineString, Polygon (outer ring only), MultiLineString (all lines flattened),
/// MultiPolygon (first polygon outer ring).
pub fn parse_geojson_coords(geojson: &str) -> Vec<[f64; 2]> {
    let v: serde_json::Value = match serde_json::from_str(geojson) {
        Ok(val) => val,
        Err(_) => return vec![],
    };

    let geom_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let coordinates = v.get("coordinates");

    match (geom_type, coordinates) {
        ("LineString", Some(coords)) => parse_coord_array(coords),
        ("MultiLineString", Some(coords)) => {
            // flatten all line segments
            coords
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .flat_map(|line| parse_coord_array(line))
                .collect()
        }
        ("Polygon", Some(coords)) => {
            // outer ring is index 0
            coords
                .as_array()
                .and_then(|rings| rings.first())
                .map(parse_coord_array)
                .unwrap_or_default()
        }
        ("MultiPolygon", Some(coords)) => {
            // first polygon, outer ring
            coords
                .as_array()
                .and_then(|polys| polys.first())
                .and_then(|poly| poly.as_array())
                .and_then(|rings| rings.first())
                .map(parse_coord_array)
                .unwrap_or_default()
        }
        _ => vec![],
    }
}

fn parse_coord_array(coords: &serde_json::Value) -> Vec<[f64; 2]> {
    coords
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|pt| {
            let arr = pt.as_array()?;
            let lng = arr.first()?.as_f64()?;
            let lat = arr.get(1)?.as_f64()?;
            Some([lng, lat])
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Row → raw conversion
// ---------------------------------------------------------------------------

fn roads_to_raw(rows: Vec<RoadRow>) -> Vec<RoadFeatureRaw> {
    rows.into_iter()
        .map(|r| RoadFeatureRaw {
            osm_id: r.osm_id,
            highway: r.highway,
            name: r.name,
            width_meters: r.width_meters,
            coords: parse_geojson_coords(&r.geojson),
        })
        .filter(|f| !f.coords.is_empty())
        .collect()
}

fn water_to_raw(rows: Vec<WaterRow>) -> Vec<WaterFeatureRaw> {
    rows.into_iter()
        .map(|r| WaterFeatureRaw {
            osm_id: r.osm_id,
            geom_type: r.geom_type,
            water_type: r.water_type,
            name: r.name,
            coords: parse_geojson_coords(&r.geojson),
        })
        .filter(|f| !f.coords.is_empty())
        .collect()
}

fn parks_to_raw(rows: Vec<ParkRow>) -> Vec<ParkFeatureRaw> {
    rows.into_iter()
        .map(|r| ParkFeatureRaw {
            osm_id: r.osm_id,
            park_type: r.park_type,
            name: r.name,
            coords: parse_geojson_coords(&r.geojson),
        })
        .filter(|f| !f.coords.is_empty())
        .collect()
}

fn buildings_to_raw(rows: Vec<BuildingRow>) -> Vec<BuildingFeatureRaw> {
    rows.into_iter()
        .map(|r| BuildingFeatureRaw {
            osm_id: r.osm_id,
            height_meters: r.height_meters,
            name: r.name,
            coords: parse_geojson_coords(&r.geojson),
        })
        .filter(|f| !f.coords.is_empty())
        .collect()
}

// ---------------------------------------------------------------------------
// Raw → response conversion (lat/lng → local meters)
// ---------------------------------------------------------------------------

fn raw_coords_to_local(
    raw_coords: &[[f64; 2]],
    center_lat: f64,
    center_lng: f64,
) -> Vec<[f64; 2]> {
    raw_coords
        .iter()
        .map(|[lng, lat]| {
            let (x, z) = geo_to_local(*lat, *lng, center_lat, center_lng);
            [x, z]
        })
        .collect()
}

fn raw_to_response(raw: TileRawData, center_lat: f64, center_lng: f64) -> TileResponse {
    let roads = raw
        .roads
        .into_iter()
        .map(|f| RoadFeature {
            osm_id: f.osm_id,
            highway: f.highway,
            name: f.name,
            width_meters: f.width_meters,
            coords: raw_coords_to_local(&f.coords, center_lat, center_lng),
        })
        .collect();

    let water = raw
        .water
        .into_iter()
        .map(|f| WaterFeature {
            osm_id: f.osm_id,
            geom_type: f.geom_type,
            water_type: f.water_type,
            name: f.name,
            coords: raw_coords_to_local(&f.coords, center_lat, center_lng),
        })
        .collect();

    let parks = raw
        .parks
        .into_iter()
        .map(|f| ParkFeature {
            osm_id: f.osm_id,
            park_type: f.park_type,
            name: f.name,
            coords: raw_coords_to_local(&f.coords, center_lat, center_lng),
        })
        .collect();

    let buildings = raw
        .buildings
        .into_iter()
        .map(|f| BuildingFeature {
            osm_id: f.osm_id,
            height_meters: f.height_meters,
            name: f.name,
            coords: raw_coords_to_local(&f.coords, center_lat, center_lng),
        })
        .collect();

    TileResponse {
        roads,
        water,
        parks,
        buildings,
    }
}

// ---------------------------------------------------------------------------
// Main service function
// ---------------------------------------------------------------------------

pub async fn get_tile(
    pool: &PgPool,
    redis: &RedisPool,
    z: u8,
    x: u32,
    y: u32,
    center_lat: f64,
    center_lng: f64,
) -> Result<TileResponse, AppError> {
    let cache_key = format!("tile:{z}:{x}:{y}");

    // Try Redis cache first
    if let Ok(mut conn) = redis.get().await {
        if let Ok(Some(cached)) = conn.get::<_, Option<String>>(&cache_key).await {
            if let Ok(raw) = serde_json::from_str::<TileRawData>(&cached) {
                return Ok(raw_to_response(raw, center_lat, center_lng));
            }
        }
    }

    // Cache miss – run 4 queries in parallel
    let (west, south, east, north) = world_repo::tile_to_bbox(z, x, y);
    let tolerance = world_repo::zoom_tolerance(z);

    let (roads_res, water_res, parks_res, buildings_res) = tokio::join!(
        world_repo::get_roads(pool, west, south, east, north, tolerance),
        world_repo::get_water(pool, west, south, east, north, tolerance),
        world_repo::get_parks(pool, west, south, east, north, tolerance),
        world_repo::get_buildings(pool, west, south, east, north, tolerance),
    );

    let raw = TileRawData {
        roads: roads_to_raw(roads_res.map_err(AppError::from)?),
        water: water_to_raw(water_res.map_err(AppError::from)?),
        parks: parks_to_raw(parks_res.map_err(AppError::from)?),
        buildings: buildings_to_raw(buildings_res.map_err(AppError::from)?),
    };

    // Fire-and-forget cache write (TTL 3600s); silently ignore errors
    if let Ok(serialized) = serde_json::to_string(&raw) {
        if let Ok(mut conn) = redis.get().await {
            let _ = conn.set_ex::<_, _, ()>(&cache_key, serialized, 3600).await;
        }
    }

    Ok(raw_to_response(raw, center_lat, center_lng))
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_to_local_zero_offset() {
        let (x, z) = geo_to_local(48.8566, 2.3522, 48.8566, 2.3522);
        assert!(x.abs() < 1e-6, "x should be 0.0, got {x}");
        assert!(z.abs() < 1e-6, "z should be 0.0, got {z}");
    }

    #[test]
    fn test_geo_to_local_north_is_positive_z() {
        // Moving north (higher lat) should give positive z
        let (_, z) = geo_to_local(48.9, 2.3522, 48.8566, 2.3522);
        assert!(z > 0.0, "north offset should produce positive z, got {z}");
    }

    #[test]
    fn test_geo_to_local_east_is_positive_x() {
        // Moving east (higher lng) should give positive x
        let (x, _) = geo_to_local(48.8566, 2.4, 48.8566, 2.3522);
        assert!(x > 0.0, "east offset should produce positive x, got {x}");
    }

    #[test]
    fn test_geo_to_local_distance_approx() {
        // 1 degree of latitude ≈ 111320 meters
        let (_, z) = geo_to_local(49.8566, 2.3522, 48.8566, 2.3522);
        let expected = 111_320.0_f64;
        assert!(
            (z - expected).abs() < 10.0,
            "1 degree lat should be ~111320m, got {z}"
        );
    }

    #[test]
    fn test_parse_geojson_coords_linestring() {
        let gj = r#"{"type":"LineString","coordinates":[[2.0,48.0],[3.0,49.0]]}"#;
        let coords = parse_geojson_coords(gj);
        assert_eq!(coords.len(), 2);
        assert_eq!(coords[0], [2.0, 48.0]);
        assert_eq!(coords[1], [3.0, 49.0]);
    }

    #[test]
    fn test_parse_geojson_coords_polygon() {
        let gj = r#"{"type":"Polygon","coordinates":[[[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,1.0],[0.0,0.0]]]}"#;
        let coords = parse_geojson_coords(gj);
        assert_eq!(coords.len(), 5);
        assert_eq!(coords[0], [0.0, 0.0]);
    }

    #[test]
    fn test_parse_geojson_coords_multipolygon() {
        let gj = r#"{"type":"MultiPolygon","coordinates":[[[[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,0.0]]],[[[5.0,5.0],[6.0,5.0],[6.0,6.0],[5.0,5.0]]]]}"#;
        let coords = parse_geojson_coords(gj);
        // Should return outer ring of first polygon
        assert_eq!(coords.len(), 4);
        assert_eq!(coords[0], [0.0, 0.0]);
    }

    #[test]
    fn test_parse_geojson_coords_multilinestring() {
        let gj = r#"{"type":"MultiLineString","coordinates":[[[0.0,0.0],[1.0,1.0]],[[2.0,2.0],[3.0,3.0]]]}"#;
        let coords = parse_geojson_coords(gj);
        assert_eq!(coords.len(), 4);
    }

    #[test]
    fn test_parse_geojson_coords_invalid() {
        assert!(parse_geojson_coords("not json").is_empty());
        assert!(parse_geojson_coords("{}").is_empty());
    }
}
