//! `GET /api/v1/me` — return the authenticated user's full profile.

use axum::{extract::State, routing::get, Json, Router};
use placebo_shared::user::MeResponse;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::repositories::user_repo;

pub fn router() -> Router<AppState> {
    Router::new().route("/me", get(me))
}

async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<MeResponse>, AppError> {
    let user = user_repo::get_by_id(&state.db, auth.id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

    Ok(Json(MeResponse {
        id: user.id,
        email: user.email,
        // Migration 008 backfills usernames for every existing row, so this is effectively
        // never None. The empty-string fallback keeps us safe against any race during a
        // partial migration.
        username: user.username.unwrap_or_default(),
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        locale: user.locale,
        is_premium: user.is_premium,
        premium_until: user.premium_until,
        date_of_birth: user.date_of_birth,
        date_of_birth_hidden: user.date_of_birth_hidden,
        email_verified: user.email_verified,
        created_at: user.created_at,
    }))
}
