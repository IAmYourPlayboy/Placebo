use axum::{
    extract::State,
    http::HeaderMap,
    Json,
    Router,
    routing::post,
};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::services::auth_service;
use placebo_shared::auth::{
    AuthResponse, LoginRequest, MessageResponse, PasswordResetConfirm,
    PasswordResetRequest, RefreshRequest, RegisterRequest,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/logout-all", post(logout_all))
        .route("/refresh", post(refresh))
        .route("/password-reset", post(request_password_reset))
        .route("/password-reset/confirm", post(confirm_password_reset))
}

/// POST /api/v1/auth/register
async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // In dev, auto-verify email. In prod, require verification.
    let auto_verify = state.config.environment == crate::config::Environment::Dev;
    let resp = auth_service::register(&state.db, &state.redis, &req, auto_verify).await?;
    Ok(Json(resp))
}

/// POST /api/v1/auth/login
async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let resp = auth_service::login(&state.db, &state.redis, &req).await?;
    Ok(Json(resp))
}

/// POST /api/v1/auth/logout – requires auth token in Authorization header
async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MessageResponse>, AppError> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("Missing token".into()))?;

    auth_service::logout(&state.redis, token).await?;
    Ok(Json(MessageResponse {
        message: "Logged out successfully".into(),
    }))
}

/// POST /api/v1/auth/logout-all – logout from all devices
async fn logout_all(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<MessageResponse>, AppError> {
    let count = auth_service::logout_all(&state.redis, auth.id).await?;
    Ok(Json(MessageResponse {
        message: format!("Logged out from {count} session(s)"),
    }))
}

/// POST /api/v1/auth/refresh – extend session TTL
async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let ttl = auth_service::refresh_session(&state.redis, &req.token).await?;
    Ok(Json(MessageResponse {
        message: format!("Session extended for {ttl} seconds"),
    }))
}

/// POST /api/v1/auth/password-reset – request reset email
async fn request_password_reset(
    State(state): State<AppState>,
    Json(req): Json<PasswordResetRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    auth_service::request_password_reset(&state.db, &req.email).await?;
    // Always return success – don't reveal if email exists
    Ok(Json(MessageResponse {
        message: "If an account with that email exists, a reset link has been sent".into(),
    }))
}

/// POST /api/v1/auth/password-reset/confirm – set new password
async fn confirm_password_reset(
    State(state): State<AppState>,
    Json(req): Json<PasswordResetConfirm>,
) -> Result<Json<MessageResponse>, AppError> {
    let errors = req.validate();
    if !errors.is_empty() {
        return Err(AppError::Validation(errors.join("; ")));
    }
    auth_service::confirm_password_reset(&state.db, &state.redis, &req.token, &req.new_password)
        .await?;
    Ok(Json(MessageResponse {
        message: "Password updated successfully. Please login with your new password.".into(),
    }))
}
