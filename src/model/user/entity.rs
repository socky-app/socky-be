use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::model::user::error::UserStatusError;

/// Database entity representing a User registered in the system.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserEntity {
    /// Unique identifier for the user (automatically incremented).
    pub id: i64,
    /// User's email address (normalized, unique).
    pub email: String,
    /// Secure Argon2 peppered password hash.
    pub password_hash: String,
    // pub username: String,
    // pub full_name: String,
    /// Role for access control / RBAC.
    pub role: UserRole,
    /// Account lifecycle status.
    pub status: UserStatus,
    /// Timestamp of the last successful login.
    pub last_login_at: Option<NaiveDateTime>,
    /// Creation timestamp.
    pub created_at: NaiveDateTime,
    /// Auto-updating timestamp for modifications.
    pub updated_at: NaiveDateTime,
}

/// User role enum for access control (RBAC).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[repr(i16)]
pub enum UserRole {
    /// Standard operations allowed (default).
    Standard = 1,
    /// Support and diagnostic operations.
    Support = 2,
    /// Administrative operations (full system access).
    Admin = 3,
}

/// User status enum for account lifecycle control.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[repr(i16)]
pub enum UserStatus {
    /// Fully active user (normal execution permitted).
    Active = 1,
    /// Disabled user (self-deactivated, has limited login/re-enable access).
    Disabled = 2,
    /// Pending user (newly created, waiting verification).
    Pending = 3,
    /// Locked user (security block, support intervention needed).
    Locked = 4,
    // Deleted = 5, // Soft deleted, cannot be re-activated
}

impl UserStatus {
    /// Checks if the user status allows logging in.
    /// Returns Ok(()) if active, or returns a specialized UserStatusError on restriction.
    pub fn check_status(&self) -> Result<(), UserStatusError> {
        match self {
            UserStatus::Active => Ok(()),
            UserStatus::Disabled => Err(UserStatusError::Disabled),
            UserStatus::Pending => Err(UserStatusError::Pending),
            UserStatus::Locked => Err(UserStatusError::Locked),
        }
    }
}
