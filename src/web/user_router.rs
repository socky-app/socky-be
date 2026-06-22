use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use tracing::trace;

use crate::{
    app::AppState,
    context::CurrentUser,
    controller::user_controller::UserController,
    model::{
        pagination::PaginatedResponse,
        user::{
            dto::{UpdateProfileDto, UpdateUserStatusDto, UpdateUsernameDto, UserQueryDto},
            vo::{RegisteredUserVo, UserVo},
            UserRole, UserStatus,
        },
    },
    repository::RepositoryManager,
    web::{
        extractor::{RequireAdmin, RequireSupport},
        ClientError, Result,
    },
};

/// Protected user routes
/// Protected routes that require an Active user
pub fn requires_active() -> Router<AppState> {
    Router::new()
        // Administrative Routes (Support + Admin)
        .route("/", get(list_users_handler))
        .route("/{id}", get(get_user_handler))
        .route("/{id}", patch(update_user_profile_handler))
        .route("/{id}/enable", post(enable_user_handler))
        .route("/{id}/disable", post(disable_user_handler))
        .route("/{id}/lock", post(lock_user_handler))
        .route("/{id}", delete(admin_delete_user_handler))
        // Self-Service Routes
        .route("/me", patch(me_update_profile_handler))
        .route("/me/username", patch(me_update_username_handler))
}

/// Protected routes that allow Disabled/Locked users
pub fn requires_auth() -> Router<AppState> {
    Router::new()
        .route("/me", get(me_handler).delete(me_delete_handler))
        .route("/me/enable", post(me_enable_handler))
}

// ============================================================================
// SELF-SERVICE ENDPOINTS
// ============================================================================

/// Get current user information.
#[utoipa::path(
    get,
    path = "/api/user/me",
    tag = "User",
    summary = "Get current user profile",
    responses(
        (status = 200, description = "User profile retrieved", body = RegisteredUserVo),
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
) -> Result<Json<RegisteredUserVo>> {
    trace!("Handler user me");
    Ok(Json(
        UserController::get_registered_user(&rm, current_user.id).await?,
    ))
}

/// Re-activate current user account
#[utoipa::path(
    post,
    path = "/api/user/me/enable",
    tag = "User",
    summary = "Reactivate current user account",
    responses(
        (status = 200, description = "Account reactivated"),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn me_enable_handler(
    State(rm): State<RepositoryManager>,
    current_user: CurrentUser,
) -> Result<()> {
    trace!("Handler user me_enable");
    let dto = UpdateUserStatusDto {
        status: UserStatus::Active,
    };
    Ok(UserController::update_status(&rm, current_user.id, dto).await?)
}

/// Soft-delete and anonymize user account
#[utoipa::path(
    delete,
    path = "/api/user/me",
    tag = "User",
    summary = "Soft-delete current user account",
    responses(
        (status = 200, description = "Account soft-deleted"),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn me_delete_handler(
    State(rm): State<RepositoryManager>,
    current_user: CurrentUser,
) -> Result<()> {
    trace!("Handler user me_delete");
    Ok(UserController::delete_user(&rm, current_user.id).await?)
}

/// Update current user profile
#[utoipa::path(
    patch,
    path = "/api/user/me",
    tag = "User",
    summary = "Update current user profile fields",
    request_body = UpdateProfileDto,
    responses(
        (status = 200, description = "Profile updated"),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn me_update_profile_handler(
    State(rm): State<RepositoryManager>,
    current_user: CurrentUser,
    Json(request): Json<UpdateProfileDto>,
) -> Result<()> {
    trace!("Handler user me_update_profile");
    Ok(UserController::update_profile(&rm, current_user.id, request).await?)
}

/// Update current user username
#[utoipa::path(
    patch,
    path = "/api/user/me/username",
    tag = "User",
    summary = "Update current user username handle",
    request_body = UpdateUsernameDto,
    responses(
        (status = 200, description = "Username updated"),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 409, description = "Conflict (Username already exists)", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn me_update_username_handler(
    State(rm): State<RepositoryManager>,
    current_user: CurrentUser,
    Json(request): Json<UpdateUsernameDto>,
) -> Result<()> {
    trace!("Handler user me_update_username");
    Ok(UserController::update_username(&rm, current_user.id, request).await?)
}

// ============================================================================
// ADMINISTRATIVE ENDPOINTS
// ============================================================================

/// List all users
#[utoipa::path(
    get,
    path = "/api/user",
    tag = "User",
    summary = "List users (Support/Admin)",
    params(UserQueryDto),
    responses(
        (status = 200, description = "List of users", body = PaginatedResponse<UserVo>),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn list_users_handler(
    State(rm): State<RepositoryManager>,
    _require_support: RequireSupport,
    Query(query): Query<UserQueryDto>,
) -> Result<Json<PaginatedResponse<UserVo>>> {
    trace!("Handler admin list_users");
    Ok(Json(UserController::list_users(&rm, &query).await?))
}

/// Get specific user details
#[utoipa::path(
    get,
    path = "/api/user/{id}",
    tag = "User",
    summary = "Get user details (Support/Admin)",
    params(
        ("id" = i64, Path, description = "User database ID")
    ),
    responses(
        (status = 200, description = "User profile retrieved", body = UserVo),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 404, description = "Not found", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn get_user_handler(
    State(rm): State<RepositoryManager>,
    _require_support: RequireSupport,
    Path(id): Path<i64>,
) -> Result<Json<UserVo>> {
    trace!("Handler admin get_user");
    Ok(Json(UserController::get_user(&rm, id).await?))
}

/// Update user profile
#[utoipa::path(
    patch,
    path = "/api/user/{id}",
    tag = "User",
    summary = "Update user profile (Support/Admin)",
    params(
        ("id" = i64, Path, description = "User database ID")
    ),
    request_body = UpdateProfileDto,
    responses(
        (status = 200, description = "Profile updated"),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn update_user_profile_handler(
    State(rm): State<RepositoryManager>,
    _require_support: RequireSupport,
    Path(id): Path<i64>,
    Json(request): Json<UpdateProfileDto>,
) -> Result<()> {
    trace!("Handler admin update_user_profile");
    Ok(UserController::update_profile(&rm, id, request).await?)
}

/// Enable user
#[utoipa::path(
    post,
    path = "/api/user/{id}/enable",
    tag = "User",
    summary = "Enable user account (Support/Admin)",
    params(
        ("id" = i64, Path, description = "User database ID")
    ),
    responses(
        (status = 200, description = "Account enabled"),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn enable_user_handler(
    State(rm): State<RepositoryManager>,
    _require_support: RequireSupport,
    Path(id): Path<i64>,
) -> Result<()> {
    trace!("Handler admin enable_user");
    let dto = UpdateUserStatusDto {
        status: UserStatus::Active,
    };
    Ok(UserController::update_status(&rm, id, dto).await?)
}

/// Disable user
#[utoipa::path(
    post,
    path = "/api/user/{id}/disable",
    tag = "User",
    summary = "Disable user account (Support/Admin)",
    params(
        ("id" = i64, Path, description = "User database ID")
    ),
    responses(
        (status = 200, description = "Account disabled"),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn disable_user_handler(
    State(rm): State<RepositoryManager>,
    _require_support: RequireSupport,
    Path(id): Path<i64>,
) -> Result<()> {
    trace!("Handler admin disable_user");
    let dto = UpdateUserStatusDto {
        status: UserStatus::Disabled,
    };
    Ok(UserController::update_status(&rm, id, dto).await?)
}

/// Lock user
#[utoipa::path(
    post,
    path = "/api/user/{id}/lock",
    tag = "User",
    summary = "Lock user account (Support/Admin)",
    params(
        ("id" = i64, Path, description = "User database ID")
    ),
    responses(
        (status = 200, description = "Account locked"),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn lock_user_handler(
    State(rm): State<RepositoryManager>,
    _require_support: RequireSupport,
    Path(id): Path<i64>,
) -> Result<()> {
    trace!("Handler admin lock_user");
    let dto = UpdateUserStatusDto {
        status: UserStatus::Locked,
    };
    Ok(UserController::update_status(&rm, id, dto).await?)
}

/// Delete user (Admin only)
#[utoipa::path(
    delete,
    path = "/api/user/{id}",
    tag = "User",
    summary = "Delete and anonymize user account (Admin only)",
    params(
        ("id" = i64, Path, description = "User database ID")
    ),
    responses(
        (status = 200, description = "Account deleted"),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn admin_delete_user_handler(
    State(rm): State<RepositoryManager>,
    _require_admin: RequireAdmin,
    Path(id): Path<i64>,
) -> Result<()> {
    trace!("Handler admin delete_user");
    Ok(UserController::admin_delete_user(&rm, id).await?)
}
