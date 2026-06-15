use serde::Serialize;

use crate::model::user::{UserRole, UserStatus};

/// Response payload for autheticated user.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AuthResponseVo {
    /// Token for authenticating subsequent requests
    pub access_token: String,
    /// Token for refreshing the user credentials
    pub refresh_token: String,
    /// User information
    pub user_info: LoggedUserInfoVo,
}

/// Comprehensive user information for authenticated sessions.
#[derive(Debug, Serialize, Clone, utoipa::ToSchema)]
pub struct LoggedUserInfoVo {
    /// Unique identifier of the user
    pub id: i64,
    /// Email of the user
    pub email: String,
    /// Username handle of the user
    pub username: String,
    /// Display name of the user
    pub full_name: String,
    /// Access role of the user
    pub role: UserRole,
    /// Lifecycle status of the user
    pub status: UserStatus,
}
