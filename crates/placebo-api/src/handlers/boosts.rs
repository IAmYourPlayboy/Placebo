use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::services::boost_service;

// ---------------------------------------------------------------------------
// Router (nested under /cameras/:id)
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new().route("/:camera_id/boost", post(apply_boost))
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct BoostRequest {
    pub days: i16,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn apply_boost(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(camera_id): Path<Uuid>,
    Json(req): Json<BoostRequest>,
) -> Result<Json<Value>, AppError> {
    boost_service::apply_boost(&state.db, auth.id, camera_id, req.days).await?;
    Ok(Json(json!({ "data": { "ok": true } })))
}
