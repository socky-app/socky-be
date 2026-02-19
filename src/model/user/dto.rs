use serde::Deserialize;

use crate::model::user::{UserRole, UserStatus};

/// Create user request parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserDto {
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub status: UserStatus,
}

/// Request payload for user authentication.
#[derive(Deserialize)]
pub struct LoginRequestDto {
    /// Email for authentication
    pub email: String,
    /// User's password in plain text
    pub password: String,
}

/// Update user request parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserStatusDto {
    pub status: UserStatus,
}

/// Update user password request parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserPasswordDto {
    pub password: String,
}
