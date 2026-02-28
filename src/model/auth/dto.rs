use chrono::NaiveDateTime;
use serde::Deserialize;
use uuid::Uuid;

/// Request payload for user authentication.
#[derive(Deserialize)]
pub struct LoginRequestDto {
    /// Email for authentication
    pub email: String,
    /// User's password in plain text
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequestDto {
    /// Refresh token
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct LogoutRequestDto {
    /// Refresh token
    pub refresh_token: String,
}

/// Update user password request parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserPasswordDto {
    pub old_password: String,
    pub new_password: String,
}

/// Token creation parameters.
#[derive(Debug, Clone)]
pub struct CreateRefreshTokenDto<'a> {
    pub user_id: i64,
    pub family_id: Uuid,
    pub token_hash: &'a [u8],
    pub expires_at: NaiveDateTime,
}
