use serde::Deserialize;

use crate::model::user::{UserRole, UserStatus};

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
