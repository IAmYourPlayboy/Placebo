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
    services::ServeDir,
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

    // Static directory for loop_mp4 demo assets referenced by the HLS proxy.
    // Path is resolved relative to the process's working directory; we run
    // the API from `crates/placebo-api/`, so the local `static/` folder works.
    let static_dir = std::path::PathBuf::from(
        std::env::var("PLACEBO_STATIC_DIR").unwrap_or_else(|_| "static".to_string()),
    );

    Router::new()
        .route("/health", get(handlers::health::health))
        .route("/readiness", get(handlers::health::readiness))
        .nest("/api/v1", handlers::api_router())
        .nest_service("/static", ServeDir::new(static_dir))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(cors)
        .with_state(state)
}
