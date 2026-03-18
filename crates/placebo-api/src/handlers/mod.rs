pub mod auth;
pub mod boosts;
pub mod cameras;
pub mod clips;
pub mod health;
pub mod ratings;
pub mod rooms;
pub mod users;
pub mod world;

use axum::Router;
use crate::app_state::AppState;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/cameras", cameras::router()
            .merge(ratings::router())
            .merge(boosts::router())
            .merge(clips::camera_router())
        )
        .nest("/rooms", rooms::router())
        .nest("/users", users::router())
        .nest("/clips", clips::user_router())
        .nest("/world", world::router())
}
