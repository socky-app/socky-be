use serde::Deserialize;

use crate::model::{
    pagination,
    user::{UserRole, UserStatus},
};

/// Data transfer object representing the payload to create a new user.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateRegisteredUserDto {
    /// The normalized email address of the user.
    pub email: String,
    /// Unique handle.
    pub username: String,
    /// Display name.
    pub full_name: String,
    /// The hashed password.
    pub password_hash: String,
    /// Access role assigned to the user.
    pub role: UserRole,
    /// Initial lifecycle status assigned to the user.
    pub status: UserStatus,
}

/// Data transfer object representing the payload to create a new ghost user.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateGhostUserDto {
    /// Display name for the ghost user.
    pub full_name: String,
}

/// Data transfer object representing the payload to update a user's status.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserStatusDto {
    /// New lifecycle status to assign to the target user.
    pub status: UserStatus,
}

/// Data transfer object representing the payload to update a user's profile.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateProfileDto {
    /// The user's new display name.
    pub full_name: String,
}

/// Data transfer object representing the payload to update a user's username handle.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateUsernameDto {
    /// The user's new username handle.
    pub username: String,
}

/// Query parameters for listing users
#[derive(Deserialize, utoipa::IntoParams)]
pub struct UserQueryDto {
    pub is_ghost: Option<bool>,
    pub status: Option<UserStatus>,
    pub role: Option<UserRole>,
    #[serde(default = "pagination::default_page")]
    pub page: i64,
    #[serde(default = "pagination::default_limit")]
    pub limit: i64,
}

/// Query parameters for searching public profiles
#[derive(Deserialize, utoipa::IntoParams)]
pub struct ProfileSearchQueryDto {
    /// The search query
    pub query: String,
}

/// Query parameters for checking username availability
#[derive(Deserialize, utoipa::IntoParams)]
pub struct CheckUsernameQueryDto {
    /// The username to check
    pub username: String,
}
