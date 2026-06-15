use chrono::NaiveDateTime;
use serde::Deserialize;
use uuid::Uuid;

/// Request payload for user authentication.
#[derive(Deserialize, utoipa::ToSchema)]
pub struct LoginRequestDto {
    /// Email for authentication.
    pub email: String,
    /// User's password in plain text.
    pub password: String,
}

/// Request payload for user registration.
#[derive(Deserialize, utoipa::ToSchema)]
pub struct SignupRequestDto {
    /// Email for authentication.
    pub email: String,
    /// User's password in plain text.
    pub password: String,
    /// Unique username handle.
    pub username: String,
    /// Display name.
    pub full_name: String,
}

/// Request payload to refresh expired credentials.
#[derive(Deserialize, utoipa::ToSchema)]
pub struct RefreshRequestDto {
    /// Opaque refresh token string.
    pub refresh_token: String,
}

/// Request payload to log out a session.
#[derive(Deserialize, utoipa::ToSchema)]
pub struct LogoutRequestDto {
    /// Opaque refresh token string.
    pub refresh_token: String,
}

/// Request payload to update the user's password.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserPasswordDto {
    /// User's current password.
    pub old_password: String,
    /// User's new password.
    pub new_password: String,
}

/// Internal DTO representing token parameters to insert in the database.
#[derive(Debug, Clone)]
pub struct CreateRefreshTokenDto<'a> {
    /// HMAC hash of the opaque refresh token.
    pub token_hash: &'a [u8],
    /// Owner user identifier.
    pub user_id: i64,
    /// Unique family identifier for token rotation sequence.
    pub family_id: Uuid,
    /// Unique identifier of the associated access token.
    pub access_id: Uuid,
    /// Expiration timestamp.
    pub expires_at: NaiveDateTime,
}
