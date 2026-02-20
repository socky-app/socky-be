use chrono::NaiveDateTime;
use serde::Serialize;

use crate::model::user::{UserEntity, UserRole, UserStatus};

/// User item for list display.
#[derive(Debug, Serialize)]
pub struct UserVo {
    pub id: i64,
    pub email: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// User option.
pub type UserOptionVo = Option<i64>;

impl From<UserEntity> for UserVo {
    fn from(user: UserEntity) -> Self {
        Self {
            id: user.user_id,
            email: user.email,
            role: user.role,
            status: user.status,
            last_login_at: user.last_login_at,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

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
