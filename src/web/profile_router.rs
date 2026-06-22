use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::{
    app::AppState,
    controller::profile_controller::ProfileController,
    model::user::{
        dto::{CheckUsernameQueryDto, ProfileSearchQueryDto},
        vo::{CheckUsernameResponseVo, PublicProfileVo},
    },
    repository::RepositoryManager,
    web::{ClientError, Result},
};

/// Public profile routes
pub fn public() -> Router<AppState> {
    Router::new().route("/check-username", get(check_username_handler))
}

/// Protected profile routes
pub fn requires_active() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_profiles_handler))
        .route("/{id}", get(get_profile_handler))
}

/// Check username availability
#[utoipa::path(
    get,
    path = "/api/profile/check-username",
    tag = "Profile",
    summary = "Check if username is available",
    params(
        CheckUsernameQueryDto
    ),
    responses(
        (status = 200, description = "Availability checked", body = CheckUsernameResponseVo),
        (status = 500, description = "Internal server error", body = ClientError)
    )
)]
async fn check_username_handler(
    State(rm): State<RepositoryManager>,
    Query(query): Query<CheckUsernameQueryDto>,
) -> Result<Json<CheckUsernameResponseVo>> {
    trace!("Handler profile check_username");
    let available = ProfileController::check_username(&rm, &query.username).await?;
    Ok(Json(CheckUsernameResponseVo { available }))
}

/// Search public profiles
#[utoipa::path(
    get,
    path = "/api/profile/search",
    tag = "Profile",
    summary = "Search public profiles",
    params(
        ProfileSearchQueryDto
    ),
    responses(
        (status = 200, description = "Profiles retrieved", body = Vec<PublicProfileVo>),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn search_profiles_handler(
    State(rm): State<RepositoryManager>,
    Query(query): Query<ProfileSearchQueryDto>,
) -> Result<Json<Vec<PublicProfileVo>>> {
    trace!("Handler profile search");
    Ok(Json(ProfileController::search(&rm, &query.query).await?))
}

/// Get public profile by ID
#[utoipa::path(
    get,
    path = "/api/profile/{id}",
    tag = "Profile",
    summary = "Get public profile by ID",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Profile retrieved", body = PublicProfileVo),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 404, description = "Profile not found", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn get_profile_handler(
    State(rm): State<RepositoryManager>,
    Path(id): Path<i64>,
) -> Result<Json<PublicProfileVo>> {
    trace!("Handler profile get");
    Ok(Json(ProfileController::get_profile(&rm, id).await?))
}
