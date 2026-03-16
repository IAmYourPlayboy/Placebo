use sqlx::PgPool;

// ---------------------------------------------------------------------------
// Row types
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Tile math helpers
// ---------------------------------------------------------------------------

/// Convert XYZ tile coordinates to WGS-84 bounding box.
/// Returns (west, south, east, north) in degrees.
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

/// ST_SimplifyPreserveTopology tolerance in degrees per zoom level.
pub fn zoom_tolerance(z: u8) -> f64 {
    match z {
        17 => 0.00001,
        16 => 0.00005,
        15 => 0.0001,
        _ => 0.00005,
    }
}

// ---------------------------------------------------------------------------
// PostGIS query functions
// ---------------------------------------------------------------------------

pub async fn get_roads(
    pool: &PgPool,
    west: f64,
    south: f64,
    east: f64,
    north: f64,
    tolerance: f64,
) -> Result<Vec<RoadRow>, sqlx::Error> {
    sqlx::query_as::<_, RoadRow>(
        r#"SELECT osm_id, highway, name, width_meters,
                  ST_AsGeoJSON(ST_SimplifyPreserveTopology(way, $5)) AS geojson
           FROM roads_view
           WHERE way && ST_MakeEnvelope($1, $2, $3, $4, 4326)"#,
    )
    .bind(west)
    .bind(south)
    .bind(east)
    .bind(north)
    .bind(tolerance)
    .fetch_all(pool)
    .await
}

pub async fn get_water(
    pool: &PgPool,
    west: f64,
    south: f64,
    east: f64,
    north: f64,
    tolerance: f64,
) -> Result<Vec<WaterRow>, sqlx::Error> {
    sqlx::query_as::<_, WaterRow>(
        r#"SELECT osm_id, geom_type, water_type, name,
                  ST_AsGeoJSON(ST_SimplifyPreserveTopology(way, $5)) AS geojson
           FROM water_view
           WHERE way && ST_MakeEnvelope($1, $2, $3, $4, 4326)"#,
    )
    .bind(west)
    .bind(south)
    .bind(east)
    .bind(north)
    .bind(tolerance)
    .fetch_all(pool)
    .await
}

pub async fn get_parks(
    pool: &PgPool,
    west: f64,
    south: f64,
    east: f64,
    north: f64,
    tolerance: f64,
) -> Result<Vec<ParkRow>, sqlx::Error> {
    sqlx::query_as::<_, ParkRow>(
        r#"SELECT osm_id, park_type, name,
                  ST_AsGeoJSON(ST_SimplifyPreserveTopology(way, $5)) AS geojson
           FROM parks_view
           WHERE way && ST_MakeEnvelope($1, $2, $3, $4, 4326)"#,
    )
    .bind(west)
    .bind(south)
    .bind(east)
    .bind(north)
    .bind(tolerance)
    .fetch_all(pool)
    .await
}

pub async fn get_buildings(
    pool: &PgPool,
    west: f64,
    south: f64,
    east: f64,
    north: f64,
    tolerance: f64,
) -> Result<Vec<BuildingRow>, sqlx::Error> {
    sqlx::query_as::<_, BuildingRow>(
        r#"SELECT osm_id, height_meters, name,
                  ST_AsGeoJSON(ST_SimplifyPreserveTopology(way, $5)) AS geojson
           FROM buildings_tile_view
           WHERE way && ST_MakeEnvelope($1, $2, $3, $4, 4326)"#,
    )
    .bind(west)
    .bind(south)
    .bind(east)
    .bind(north)
    .bind(tolerance)
    .fetch_all(pool)
    .await
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_to_bbox_zoom0_tile00() {
        let (west, south, east, north) = tile_to_bbox(0, 0, 0);
        assert!((west - (-180.0)).abs() < 1e-6, "west should be -180, got {west}");
        assert!((east - 180.0).abs() < 1e-6, "east should be 180, got {east}");
        // north/south are Mercator-limited (~85.05)
        assert!(north > 85.0, "north should be ~85.05, got {north}");
        assert!(south < -85.0, "south should be ~-85.05, got {south}");
    }

    #[test]
    fn test_tile_to_bbox_zoom1() {
        // z=1, x=0, y=0 → NW quarter of the world
        let (west, south, east, north) = tile_to_bbox(1, 0, 0);
        assert!((west - (-180.0)).abs() < 1e-6);
        assert!((east - 0.0).abs() < 1e-6);
        assert!(north > 0.0);
        assert!(south > 0.0 || south.abs() < 1e-6);
    }

    #[test]
    fn test_tile_to_bbox_east_greater_than_west() {
        for z in 0u8..=17 {
            let x = 0u32;
            let y = 0u32;
            let (west, south, east, north) = tile_to_bbox(z, x, y);
            assert!(east > west, "z={z}: east ({east}) should be > west ({west})");
            assert!(north > south, "z={z}: north ({north}) should be > south ({south})");
        }
    }

    #[test]
    fn test_zoom_tolerance() {
        assert_eq!(zoom_tolerance(17), 0.00001);
        assert_eq!(zoom_tolerance(16), 0.00005);
        assert_eq!(zoom_tolerance(15), 0.0001);
        // fallback for any other zoom
        assert_eq!(zoom_tolerance(14), 0.00005);
        assert_eq!(zoom_tolerance(10), 0.00005);
        assert_eq!(zoom_tolerance(18), 0.00005);
    }
}
