use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Refresh token definition.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefreshTokenEntity {
    pub id: i64,
    pub user_id: i64,
    pub family_id: Uuid,
    pub token_hash: Vec<u8>,
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

/// Token creation parameters.
#[derive(Debug, Clone)]
pub struct CreateRefreshTokenDto<'a> {
    pub user_id: i64,
    pub family_id: Uuid,
    pub token_hash: &'a [u8],
    pub expires_at: NaiveDateTime,
}
