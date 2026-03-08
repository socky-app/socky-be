use std::sync::Arc;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use tracing::trace;

use crate::{
    app::AppState,
    config::AppConfig,
    context::CurrentUser,
    controller::auth_controller::AuthController,
    model::auth::{
        dto::{LoginRequestDto, LogoutRequestDto, RefreshRequestDto, UpdateUserPasswordDto},
        vo::{AuthResponseVo, LoggedUserInfoVo},
    },
    repository::RepositoryManager,
    web::{ClientError, Result},
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
        .route("/logout-all", post(logout_all_handler))
        .route("/password", post(update_password_handler))
        .route("/me", get(me_handler))
        .route("/health", get(health_handler))
}

/// Login with email and password.
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Auth",
    summary = "Login user",
    request_body = LoginRequestDto,
    responses(
        (status = 200, description = "Login successful", body = AuthResponseVo),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    )
)]
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
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "Auth",
    summary = "Refresh user credentials",
    request_body = RefreshRequestDto,
    responses(
        (status = 200, description = "Refresh successful", body = AuthResponseVo),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    )
)]
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
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "Auth",
    summary = "Logout user",
    request_body = LogoutRequestDto,
    responses(
        (status = 200, description = "Logout successful"),
        (status = 500, description = "Internal server error", body = ClientError)
    )
)]
async fn logout_handler(
    State(rm): State<RepositoryManager>,
    State(app_config): State<Arc<AppConfig>>,
    Json(request): Json<LogoutRequestDto>,
) -> Result<()> {
    trace!("Handler auth logout");

    Ok(AuthController::logout(&rm, &request.refresh_token, &app_config.auth).await?)
}

/// Logout user from all sessions.
#[utoipa::path(
    post,
    path = "/api/auth/logout-all",
    tag = "Auth",
    summary = "Logout user from all sessions",
    responses(
        (status = 200, description = "Logout all successful"),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn logout_all_handler(
    State(rm): State<RepositoryManager>,
    current_user: CurrentUser,
) -> Result<()> {
    trace!("Handler auth logout");

    Ok(AuthController::logout_all(&rm, current_user.id).await?)
}

/// Change password.
#[utoipa::path(
    post,
    path = "/api/auth/password",
    tag = "Auth",
    summary = "Change password",
    request_body = UpdateUserPasswordDto,
    responses(
        (status = 200, description = "Password updated successfully"),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn update_password_handler(
    State(rm): State<RepositoryManager>,
    State(app_config): State<Arc<AppConfig>>,
    current_user: CurrentUser,
    Json(request): Json<UpdateUserPasswordDto>,
) -> Result<()> {
    trace!("Handler update password");

    Ok(AuthController::update_password(&rm, current_user.id, request, &app_config.auth).await?)
}

/// Get user information.
#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "Auth",
    summary = "Get user information",
    responses(
        (status = 200, description = "User information retrieved", body = LoggedUserInfoVo),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
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
#[utoipa::path(
    get,
    path = "/api/auth/health",
    tag = "Auth",
    summary = "Check user authentication",
    responses(
        (status = 200, description = "Auth session is active"),
        (status = 401, description = "Unauthorized", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn health_handler(_current_user: CurrentUser) -> Result<()> {
    trace!("Handler auth health");
    Ok(())
}
