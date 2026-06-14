use serde::Deserialize;

use crate::model::user::{UserRole, UserStatus};

/// Data transfer object representing the payload to create a new user.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateUserDto {
    /// The normalized email address of the user.
    pub email: String,
    /// The plain text password of the user (peppered and hashed before storing).
    pub password: String,
    /// Access role assigned to the user.
    pub role: UserRole,
    /// Initial lifecycle status assigned to the user.
    pub status: UserStatus,
}

/// Data transfer object representing the payload to update a user's status.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserStatusDto {
    /// New lifecycle status to assign to the target user.
    pub status: UserStatus,
}
