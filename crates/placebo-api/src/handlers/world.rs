use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::services::world_service::{self, TileResponse};

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new().route("/tile", get(get_tile))
}

// ---------------------------------------------------------------------------
// Query params
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TileParams {
    pub z: u8,
    pub x: u32,
    pub y: u32,
    pub center_lat: f64,
    pub center_lng: f64,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

async fn get_tile(
    State(state): State<AppState>,
    Query(params): Query<TileParams>,
) -> Result<Json<TileResponse>, AppError> {
    if !(15..=17).contains(&params.z) {
        return Err(AppError::Validation(
            "z must be between 15 and 17 (inclusive)".into(),
        ));
    }

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
