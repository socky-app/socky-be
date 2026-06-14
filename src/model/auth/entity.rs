use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::user::{UserRole, UserStatus};

/// Minimal user info for authentication verification.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserCredentialsEntity {
    /// User identifier.
    pub id: i64,
    /// Securely hashed peppered password.
    pub password_hash: String,
    /// User's access role.
    pub role: UserRole,
    /// User's current account status.
    pub status: UserStatus,
}

/// Refresh token database record.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefreshTokenEntity {
    /// Token identifier.
    pub id: i64,
    /// Salted HMAC SHA-512 hash of the refresh token.
    pub token_hash: Vec<u8>,
    /// Associated user identifier.
    pub user_id: i64,
    /// Token family identifier for rotation tracking.
    pub family_id: Uuid,
    /// Unique identifier of the associated access token.
    pub access_id: Uuid,
    /// Flag indicating whether the token has been explicitly revoked.
    pub is_revoked: bool,
    /// Expiration timestamp.
    pub expires_at: NaiveDateTime,
    /// Creation timestamp.
    pub created_at: NaiveDateTime,
    /// Auto-updating timestamp for modifications.
    pub updated_at: NaiveDateTime,
}

/// Helper entity representing a revoked token.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RevokedTokenEntity {
    /// Token identifier.
    pub id: i64,
    /// Associated user identifier.
    pub user_id: i64,
    /// Token family identifier.
    pub family_id: Uuid,
    /// True if the token was already revoked prior to this call (replay flag).
    pub was_already_revoked: bool,
}

/// Detailed entity mapping the refresh token joined with user details for rotation checks.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TokenRotationEntity {
    /// Token identifier.
    pub id: i64,
    /// Salted HMAC SHA-512 hash of the refresh token.
    pub token_hash: Vec<u8>,
    /// Associated user identifier.
    pub user_id: i64,
    /// Token family identifier.
    pub family_id: Uuid,
    /// Associated access token UUID.
    pub access_id: Uuid,
    /// Status indicating if the token is revoked.
    pub is_revoked: bool,
    /// Token expiration timestamp.
    pub expires_at: NaiveDateTime,
    /// Token creation timestamp.
    pub created_at: NaiveDateTime,
    /// Token modification timestamp.
    pub updated_at: NaiveDateTime,
    /// Associated user's email address.
    pub user_email: String,
    /// Associated user's access role.
    pub user_role: UserRole,
    /// Associated user's current account status.
    pub user_status: UserStatus,
}
