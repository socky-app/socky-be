use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::header,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tracing::trace;

use crate::{
    app::AppState,
    config::AppConfig,
    context::CurrentUser,
    model::user::{
        dto::{LoginRequestDto, LogoutRequestDto, RefreshRequestDto},
        vo::{AuthResponseVo, LoggedUserInfoVo},
    },
    repository::RepositoryManager,
    controller::auth_controller::AuthController,
    web::Result,
};

/// Public auth routes
pub fn public() -> Router<AppState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/refresh", post(refresh_handler))
        .route("/logout", post(logout_handler))
}

/// Protected auth routes
pub fn protected() -> Router<AppState> {
    Router::new()
        .route("/me", get(me_handler))
        .route("/health", get(health_handler))
}

/// Login with email and password.
async fn login_handler(
    State(rm): State<RepositoryManager>,
    State(app_config): State<Arc<AppConfig>>,
    Json(request): Json<LoginRequestDto>,
) -> Result<Json<AuthResponseVo>> {
    trace!("Handler auth login");

    Ok(Json(
        AuthController::login(&rm, request, &app_config.auth).await?,
    ))
}

/// Refresh credentials and rotate tokens.
async fn refresh_handler(
    State(rm): State<RepositoryManager>,
    State(app_config): State<Arc<AppConfig>>,
    Json(request): Json<RefreshRequestDto>,
) -> Result<Json<AuthResponseVo>> {
    trace!("Handler auth refresh");

    Ok(Json(
        AuthController::refresh(&rm, &request.refresh_token, &app_config.auth).await?,
    ))
}

/// Logout user.
async fn logout_handler(
    State(rm): State<RepositoryManager>,
    State(app_config): State<Arc<AppConfig>>,
    Json(request): Json<LogoutRequestDto>,
) -> Result<()> {
    trace!("Handler auth logout");

    Ok(AuthController::logout(&rm, &request.refresh_token, &app_config.auth).await?)
}

/// Get user information.
async fn me_handler(
    State(rm): State<RepositoryManager>,
    current_user: CurrentUser,
) -> Result<Json<LoggedUserInfoVo>> {
    trace!("Handler auth me");

    Ok(Json(
        AuthController::get_login_info(&rm, current_user.id).await?,
    ))
}

/// Check user authentication.
async fn health_handler(
    current_user: CurrentUser,
) -> Result<()> {
    trace!("Handler auth health");
    Ok(())
}
