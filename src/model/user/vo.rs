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
            id: user.id,
            email: user.email,
            role: user.role,
            status: user.status,
            last_login_at: user.last_login_at,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
