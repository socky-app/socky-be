use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::model::user::error::UserError;

pub mod dto;
pub mod error;
pub mod vo;

/// User definition.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserEntity {
    pub user_id: i64,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Minimal user info for login.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LoginCredentialsEntity {
    pub user_id: i64,
    pub password_hash: String,
    pub role: UserRole,
    pub status: UserStatus,
}

/// User role enum for access control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[repr(i16)]
pub enum UserRole {
    Standard = 1, // Standard operations allowed
    Support = 2,  // Diagnostic and support operations
    Admin = 3,    // All operations
}

/// User status enum for account control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[repr(i16)]
pub enum UserStatus {
    Active = 1,   // Can perform all actions
    Disabled = 2, // Can re-activate by itself
    Pending = 3,  // Needs external action for re-activation
    Locked = 4,   // Only support can re-activate
}

impl UserStatus {
    /// Checks if the user status allows login.
    /// Returns Ok(()) if allowed, or an appropriate UserError otherwise.
    pub fn check_status(&self) -> Result<(), UserError> {
        match self {
            UserStatus::Active => Ok(()),
            UserStatus::Disabled => Err(UserError::UserIsDisabled),
            UserStatus::Pending => Err(UserError::UserIsPending),
            UserStatus::Locked => Err(UserError::UserIsLocked),
        }
    }
}
