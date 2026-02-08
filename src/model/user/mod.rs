use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::model::user::error::UserError;

pub mod dto;
pub mod error;
pub mod vo;

/// User definition.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserEntity {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub status: i16,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Minimal user info for login.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LoginCredentialsEntity {
    pub id: i64,
    pub password_hash: String,
    pub status: i16,
}

/// User status enum for authentication and account control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Normal = 1,   // Active
    Disabled = 2, // Disabled
    Pending = 3,  // Pending approval
    Locked = 4,   // Locked
}

impl TryFrom<i16> for UserStatus {
    type Error = UserError;

    /// Convert i16 to UserStatus, returns error if value is invalid.
    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(UserStatus::Normal),
            2 => Ok(UserStatus::Disabled),
            3 => Ok(UserStatus::Pending),
            4 => Ok(UserStatus::Locked),
            _ => Err(UserError::InvalidUserStatus),
        }
    }
}

impl UserStatus {
    /// Checks if the user status allows login.
    /// Returns Ok(()) if allowed, or an appropriate UserError otherwise.
    pub fn check_status(&self) -> Result<(), UserError> {
        match self {
            UserStatus::Normal => Ok(()),
            UserStatus::Disabled => Err(UserError::UserIsDisabled),
            UserStatus::Pending => Err(UserError::UserIsPending),
            UserStatus::Locked => Err(UserError::UserIsLocked),
        }
    }
}
