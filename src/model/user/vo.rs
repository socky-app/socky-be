use chrono::NaiveDateTime;
use serde::Serialize;

use crate::model::user::{GhostUser, RegisteredUser, UserEntity, UserRole, UserStatus};

/// Polymorphic user representation for list display.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserVo {
    Registered(RegisteredUserVo),
    Ghost(GhostUserVo),
}

/// Representation of a registered user.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RegisteredUserVo {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub full_name: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

/// Representation of a ghost user (no email/username/role/status).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GhostUserVo {
    pub id: i64,
    pub full_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

/// User option.
pub type UserOptionVo = Option<i64>;

impl From<UserEntity> for UserVo {
    fn from(user: UserEntity) -> Self {
        match user {
            UserEntity::Registered(u) => UserVo::Registered(RegisteredUserVo {
                id: u.id,
                email: u.email,
                username: u.username,
                full_name: u.full_name,
                role: u.role,
                status: u.status,
                last_login_at: u.last_login_at,
                created_at: u.created_at,
                updated_at: u.updated_at,
                deleted_at: u.deleted_at,
            }),
            UserEntity::Ghost(u) => UserVo::Ghost(GhostUserVo {
                id: u.id,
                full_name: u.full_name,
                created_at: u.created_at,
                updated_at: u.updated_at,
                deleted_at: u.deleted_at,
            }),
        }
    }
}

/// Public profile item for search and discovery.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PublicProfileVo {
    pub id: i64,
    pub username: String,
    pub full_name: String,
    pub created_at: NaiveDateTime,
}

impl From<RegisteredUser> for PublicProfileVo {
    fn from(user: RegisteredUser) -> Self {
        Self {
            id: user.id,
            username: user.username,
            full_name: user.full_name,
            created_at: user.created_at,
        }
    }
}

/// Response payload for checking username availability.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CheckUsernameResponseVo {
    /// True if the username is available for registration
    pub available: bool,
}
