use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::redis::session;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub is_premium: bool,
}

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("Missing authorization token".into()))?;

        // Look up session in Redis
        let session_data = session::get(&state.redis, token)
            .await
            .map_err(|e| {
                tracing::error!("Redis session lookup failed: {e}");
                AppError::Internal(anyhow::anyhow!("Session lookup failed"))
            })?
            .ok_or_else(|| AppError::Unauthorized("Invalid or expired token".into()))?;

        Ok(AuthUser {
            id: session_data.user_id,
            email: session_data.email,
            is_premium: session_data.is_premium,
        })
    }
}

/// Optional auth – returns None if no token present instead of erroring.
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[axum::async_trait]
impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let has_auth = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .is_some();

        if !has_auth {
            return Ok(OptionalAuthUser(None));
        }

        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalAuthUser(Some(user))),
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}
