use serde::Deserialize;

/// Create user request parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserDto {
    pub username: String,
    pub email: String,
    pub password: String,
    /// User status: Defaults to 1.
    pub status: Option<i16>,
}

/// Request payload for user authentication.
#[derive(Deserialize)]
pub struct LoginRequestDto {
    /// Username or email for authentication
    pub username: String,
    /// User's password in plain text
    pub password: String,
}

/// Update user request parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserDto {
    pub email: String,
}

/// Update user password request parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserPasswordDto {
    pub password: String,
}
