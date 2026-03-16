pub mod app_state;
pub mod config;
pub mod error;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod redis;
pub mod repositories;
pub mod services;
pub mod websocket;

use axum::{routing::get, Router};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::app_state::AppState;

pub async fn build_app(state: AppState) -> Router {
    let cors = if state.config.is_production() {
        CorsLayer::new()
            .allow_origin(Any) // Will be restricted to specific origins later
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::permissive()
    };

    Router::new()
        .route("/health", get(handlers::health::health))
        .route("/readiness", get(handlers::health::readiness))
        .nest("/api/v1", handlers::api_router())
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(cors)
        .with_state(state)
}
