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
use crate::extractors::auth::{AuthUser, OptionalAuthUser};
use crate::services::rating_service;

// ---------------------------------------------------------------------------
// Router (nested under /cameras/:id)
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:camera_id/ratings", post(rate).get(get_ratings).delete(delete_rating))
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RateRequest {
    pub score: i16,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn rate(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(camera_id): Path<Uuid>,
    Json(req): Json<RateRequest>,
) -> Result<Json<Value>, AppError> {
    rating_service::rate_camera(&state.db, camera_id, auth.id, req.score).await?;
    Ok(Json(json!({ "data": { "ok": true } })))
}

async fn get_ratings(
    State(state): State<AppState>,
    opt_auth: OptionalAuthUser,
    Path(camera_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let user_id = opt_auth.0.map(|u| u.id);
    let stats = rating_service::get_camera_ratings(&state.db, camera_id, user_id).await?;
    Ok(Json(json!({ "data": stats })))
}

async fn delete_rating(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(camera_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    rating_service::delete_rating(&state.db, camera_id, auth.id).await?;
    Ok(Json(json!({ "data": { "ok": true } })))
}
