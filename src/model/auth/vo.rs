use serde::Serialize;

use crate::model::user::UserRole;

/// Response payload for autheticated user.
#[derive(Debug, Serialize)]
pub struct AuthResponseVo {
    /// Token for authenticating subsequent requests
    pub access_token: String,
    /// Token for refreshing the user credentials
    pub refresh_token: String,
    /// User information
    pub user_info: LoggedUserInfoVo,
}

/// Comprehensive user information for authenticated sessions.
#[derive(Debug, Serialize, Clone)]
pub struct LoggedUserInfoVo {
    /// Unique identifier of the user
    pub id: i64,
    /// Email of the user
    pub email: String,
    /// Access role of the user
    pub role: UserRole,
}
