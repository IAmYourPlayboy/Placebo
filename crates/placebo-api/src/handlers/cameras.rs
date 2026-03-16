use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::extractors::geo::{BboxParams, NearbyParams};
use crate::extractors::pagination::PaginationParams;
use crate::services::camera_service;

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/nearby", get(nearby))
        .route("/search", get(search))
        .route("/bbox", get(bbox))
        .route("/categories", get(categories))
        .route("/count", get(count))
        .route("/:id", get(get_by_id))
}

// ---------------------------------------------------------------------------
// Query param structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    pub category: Option<String>,
    #[serde(rename = "type")]
    pub camera_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    #[serde(default = "default_search_limit")]
    pub limit: i64,
}

fn default_search_limit() -> i64 {
    50
}

#[derive(Debug, Deserialize)]
pub struct CountParams {
    pub category: Option<String>,
    #[serde(rename = "type")]
    pub camera_type: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Value>, AppError> {
    let result = camera_service::list_cameras(
        &state.db,
        params.pagination.page,
        params.pagination.per_page,
        params.category.as_deref(),
        params.camera_type.as_deref(),
    )
    .await?;

    Ok(Json(json!({
        "data": result.data,
        "meta": result.meta
    })))
}

async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let camera = camera_service::get_camera(&state.db, id).await?;
    Ok(Json(json!({ "data": camera })))
}

async fn nearby(
    State(state): State<AppState>,
    Query(params): Query<NearbyParams>,
) -> Result<Json<Value>, AppError> {
    let cameras = camera_service::get_nearby(
        &state.db,
        params.lat,
        params.lng,
        params.radius_meters(),
        params.limit,
    )
    .await?;

    Ok(Json(json!({ "data": cameras })))
}

async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Value>, AppError> {
    let cameras = camera_service::search_cameras(&state.db, &params.q, params.limit).await?;
    Ok(Json(json!({ "data": cameras })))
}

async fn bbox(
    State(state): State<AppState>,
    Query(params): Query<BboxParams>,
) -> Result<Json<Value>, AppError> {
    let cameras = camera_service::get_in_bbox(
        &state.db,
        params.sw_lat,
        params.sw_lng,
        params.ne_lat,
        params.ne_lng,
        params.limit,
    )
    .await?;

    Ok(Json(json!({ "data": cameras })))
}

async fn categories(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let cats = camera_service::get_categories(&state.db).await?;
    Ok(Json(json!({ "data": cats })))
}

async fn count(
    State(state): State<AppState>,
    Query(params): Query<CountParams>,
) -> Result<Json<Value>, AppError> {
    let total = camera_service::get_count(
        &state.db,
        params.category.as_deref(),
        params.camera_type.as_deref(),
    )
    .await?;

    Ok(Json(json!({ "data": { "count": total } })))
}
