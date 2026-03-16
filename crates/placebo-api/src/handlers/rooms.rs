use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::repositories::room_repo::UpdateRoom;
use crate::services::room_service;

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_room).get(my_rooms))
        .route("/:id", get(get_room).put(update_room).delete(delete_room))
        .route("/:id/members", get(get_members).post(join_room))
        .route("/:room_id/members/:user_id", delete(remove_member))
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomRequest {
    pub name: String,
    pub camera_id: Option<Uuid>,
    #[serde(default)]
    pub is_private: bool,
    pub max_members: Option<i16>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoomRequest {
    pub name: Option<String>,
    pub camera_id: Option<Option<Uuid>>,
    pub is_private: Option<bool>,
    pub max_members: Option<i16>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn create_room(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateRoomRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let room = room_service::create_room(
        &state.db,
        auth.id,
        req.name,
        req.camera_id,
        req.is_private,
        req.max_members,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(json!({ "data": room }))))
}

async fn my_rooms(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let rooms = room_service::get_user_rooms(&state.db, auth.id).await?;
    Ok(Json(json!({ "data": rooms })))
}

async fn get_room(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let room = room_service::get_room(&state.db, id).await?;
    Ok(Json(json!({ "data": room })))
}

async fn update_room(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRoomRequest>,
) -> Result<Json<Value>, AppError> {
    let update = UpdateRoom {
        name: req.name,
        camera_id: req.camera_id,
        is_private: req.is_private,
        max_members: req.max_members,
    };
    let room = room_service::update_room(&state.db, id, auth.id, update).await?;
    Ok(Json(json!({ "data": room })))
}

async fn delete_room(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    room_service::delete_room(&state.db, id, auth.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn join_room(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    room_service::join_room(&state.db, id, auth.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((room_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    room_service::leave_room(&state.db, room_id, user_id, auth.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_members(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let members = room_service::get_members(&state.db, id).await?;
    Ok(Json(json!({ "data": members })))
}
