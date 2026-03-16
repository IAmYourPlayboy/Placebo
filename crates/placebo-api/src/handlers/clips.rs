use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::services::clip_service;

// ---------------------------------------------------------------------------
// Router for camera-scoped clip creation: /cameras/:id/clips
// ---------------------------------------------------------------------------

pub fn camera_router() -> Router<AppState> {
    Router::new().route("/:camera_id/clips", post(request_clip))
}

// ---------------------------------------------------------------------------
// Router for user-scoped clip queries: /clips, /clips/:id
// ---------------------------------------------------------------------------

pub fn user_router() -> Router<AppState> {
    Router::new()
        .route("/", get(my_clips))
        .route("/:id", get(get_clip))
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipRequest {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn request_clip(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(camera_id): Path<Uuid>,
    Json(req): Json<ClipRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let clip = clip_service::request_clip(
        &state.db,
        camera_id,
        auth.id,
        req.start_time,
        req.end_time,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(json!({ "data": clip }))))
}

async fn my_clips(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let clips = clip_service::get_user_clips(&state.db, auth.id).await?;
    Ok(Json(json!({ "data": clips })))
}

async fn get_clip(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let clip = clip_service::get_clip(&state.db, id).await?;
    Ok(Json(json!({ "data": clip })))
}
