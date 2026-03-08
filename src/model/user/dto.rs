use serde::Deserialize;

use crate::model::user::{UserRole, UserStatus};

/// Create user request parameters.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateUserDto {
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub status: UserStatus,
}

/// Update user request parameters.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserStatusDto {
    pub status: UserStatus,
}
