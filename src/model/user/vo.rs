use chrono::NaiveDateTime;
use serde::Serialize;

use crate::model::user::UserEntity;

/// User item for list display.
#[derive(Debug, Serialize)]
pub struct UserVo {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub status: i16,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// User option.
pub type UserOptionVo = Option<i64>;

impl From<UserEntity> for UserVo {
    fn from(user: UserEntity) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            status: user.status,
            last_login_at: user.last_login_at,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// Response payload for autheticated user.
#[derive(Debug, Default, Serialize)]
pub struct AuthResponseVo {
    /// Token for authenticating subsequent requests
    pub access_token: String,
    /// Token for refreshing the user credentials
    pub refresh_token: String,
    /// User information
    pub user_info: LoggedUserInfoVo,
}

/// Comprehensive user information for authenticated sessions.
#[derive(Debug, Default, Serialize, Clone)]
pub struct LoggedUserInfoVo {
    /// Unique identifier of the user
    pub id: i64,
    /// Username of the user
    pub username: String,
    /// Email of the user
    pub email: String,
    // /// List of permission codes the user has access to
    // pub permissions: Vec<String>,
}
