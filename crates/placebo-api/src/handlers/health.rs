use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use crate::app_state::AppState;
use crate::error::AppError;

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn readiness(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    // Check PostgreSQL
    sqlx::query("SELECT 1").execute(&state.db).await?;

    // Check Redis
    let mut conn = state.redis.get().await?;
    deadpool_redis::redis::cmd("PING")
        .query_async::<String>(&mut conn)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(json!({
        "status": "ok",
        "services": {
            "postgres": "connected",
            "redis": "connected"
        }
    })))
}
