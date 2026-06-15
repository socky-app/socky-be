use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::model::user::error::UserStatusError;

/// Database row representation of a user.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: i64,
    pub email: Option<String>,
    pub username: Option<String>,
    pub full_name: String,
    pub password_hash: Option<String>,
    pub is_ghost: bool,
    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl TryFrom<UserRow> for UserEntity {
    type Error = String;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        if row.is_ghost {
            Ok(UserEntity::Ghost(GhostUser {
                id: row.id,
                full_name: row.full_name,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: row.deleted_at,
            }))
        } else {
            Ok(UserEntity::Registered(RegisteredUser {
                id: row.id,
                email: row
                    .email
                    .ok_or_else(|| "Registered user missing email".to_string())?,
                username: row
                    .username
                    .ok_or_else(|| "Registered user missing username".to_string())?,
                full_name: row.full_name,
                password_hash: row
                    .password_hash
                    .ok_or_else(|| "Registered user missing password_hash".to_string())?,
                role: row
                    .role
                    .ok_or_else(|| "Registered user missing role".to_string())?,
                status: row
                    .status
                    .ok_or_else(|| "Registered user missing status".to_string())?,
                last_login_at: row.last_login_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: row.deleted_at,
            }))
        }
    }
}

/// Represents any type of user in the system (Registered or Ghost).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserEntity {
    Registered(RegisteredUser),
    Ghost(GhostUser),
}

/// Database entity representing a registered user account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredUser {
    /// Unique identifier for the user (automatically incremented).
    pub id: i64,
    /// User's email address (normalized, unique).
    pub email: String,
    /// User's unique handle.
    pub username: String,
    /// User's display name.
    pub full_name: String,
    /// Secure Argon2 peppered password hash.
    pub password_hash: String,
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
    /// Soft delete timestamp.
    pub deleted_at: Option<NaiveDateTime>,
}

/// Database entity representing an anonymized or stub user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostUser {
    pub id: i64,
    pub full_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
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
    /// Disabled user (self-deactivated, scheduled for deletion).
    Disabled = 2,
    /// Pending user (newly created, waiting verification).
    Pending = 3,
    /// Locked user (security block, support intervention needed).
    Locked = 4,
}

impl UserStatus {
    /// Checks if the user status allows auth operations.
    /// Returns Ok(()) if active, or returns a specialized UserStatusError on restriction.
    pub fn can_authenticate(&self) -> Result<(), UserStatusError> {
        match self {
            UserStatus::Active => Ok(()),
            UserStatus::Disabled => Err(UserStatusError::Disabled),
            UserStatus::Pending => Err(UserStatusError::Pending),
            UserStatus::Locked => Err(UserStatusError::Locked),
        }
    }
}
