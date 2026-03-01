use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::user::{UserRole, UserStatus};

pub mod dto;
pub mod vo;

/// Minimal user info for auth.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserCredentialsEntity {
    pub id: i64,
    pub password_hash: String,
    pub role: UserRole,
    pub status: UserStatus,
}

/// Refresh token definition.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefreshTokenEntity {
    pub id: i64,
    pub token_hash: Vec<u8>,
    pub user_id: i64,
    pub family_id: Uuid,
    pub access_id: Uuid,
    pub is_revoked: bool,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RevokedTokenEntity {
    pub id: i64,
    pub user_id: i64,
    pub family_id: Uuid,
    pub was_already_revoked: bool,
}