use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::services::{boost_service, user_service};
use crate::repositories::user_repo::UpdateUser;

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me).put(update_me))
        .route("/me/boosts", get(my_boosts))
        .route("/:id", get(get_public))
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub locale: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn get_me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let profile = user_service::get_my_profile(&state.db, auth.id).await?;
    Ok(Json(json!({ "data": profile })))
}

async fn update_me(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<Value>, AppError> {
    let update = UpdateUser {
        display_name: req.display_name,
        avatar_url: req.avatar_url,
        locale: req.locale,
    };
    let profile = user_service::update_my_profile(&state.db, auth.id, &update).await?;
    Ok(Json(json!({ "data": profile })))
}

async fn my_boosts(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let boosts = boost_service::get_user_boosts(&state.db, auth.id).await?;
    Ok(Json(json!({ "data": boosts })))
}

async fn get_public(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let user = user_service::get_public_profile(&state.db, id).await?;
    Ok(Json(json!({ "data": user })))
}
