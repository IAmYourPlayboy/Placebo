use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NearbyParams {
    pub lat: f64,
    pub lng: f64,
    #[serde(default = "default_radius")]
    pub radius_km: f64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_radius() -> f64 {
    5.0
}

fn default_limit() -> i64 {
    50
}

impl NearbyParams {
    pub fn radius_meters(&self) -> f64 {
        self.radius_km.min(100.0) * 1000.0
    }
}

#[derive(Debug, Deserialize)]
pub struct BboxParams {
    pub sw_lat: f64,
    pub sw_lng: f64,
    pub ne_lat: f64,
    pub ne_lng: f64,
    #[serde(default = "default_bbox_limit")]
    pub limit: i64,
}

fn default_bbox_limit() -> i64 {
    200
}
