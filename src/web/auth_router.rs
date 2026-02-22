use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::header,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

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
    Router::new().route("/me", get(me_handler))
}

/// Login with email and password.
// TODO: #[tracing::instrument(name = "login", skip(pool, addr, headers, request))]
async fn login_handler(
    State(rm): State<RepositoryManager>,
    State(app_config): State<Arc<AppConfig>>,
    Json(request): Json<LoginRequestDto>,
) -> Result<Json<AuthResponseVo>> {
    tracing::debug!("{:<15} - login_handler", "HANDLER");

    Ok(Json(
        AuthController::login(&rm, request, &app_config.auth).await?,
    ))
}

/// Refresh credentials and rotate tokens.
// TODO: #[tracing::instrument(name = "refresh", skip(pool, addr, headers, request))]
async fn refresh_handler(
    State(rm): State<RepositoryManager>,
    State(app_config): State<Arc<AppConfig>>,
    Json(request): Json<RefreshRequestDto>,
) -> Result<Json<AuthResponseVo>> {
    tracing::debug!("{:<15} - refresh_handler", "HANDLER");

    Ok(Json(
        AuthController::refresh(&rm, &request.refresh_token, &app_config.auth).await?,
    ))
}

/// Logout user.
// TODO: #[tracing::instrument(name = "logout", skip(pool, addr, headers, request))]
async fn logout_handler(
    State(rm): State<RepositoryManager>,
    State(app_config): State<Arc<AppConfig>>,
    Json(request): Json<LogoutRequestDto>,
) -> Result<()> {
    tracing::debug!("{:<15} - logout_handler", "HANDLER");

    Ok(AuthController::logout(&rm, &request.refresh_token, &app_config.auth).await?)
}

/// Get user information.
// TODO: #[tracing::instrument(name = "me", skip(pool, addr, headers, request))]
async fn me_handler(
    State(rm): State<RepositoryManager>,
    current_user: CurrentUser,
) -> Result<Json<LoggedUserInfoVo>> {
    tracing::debug!("{:<15} - me_handler", "HANDLER");

    Ok(Json(
        AuthController::get_login_info(&rm, current_user.id).await?,
    ))
}
