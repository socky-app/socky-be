use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Refresh token definition.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefreshTokenEntity {
    id: i64,
    user_id: i64,
    family_id: Uuid,
    token_hash: String,
    is_revoked: bool,
    expires_at: NaiveDateTime,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

/// Token creation parameters.
#[derive(Debug, Clone)]
pub struct CreateRefreshTokenDto {
    user_id: i64,
    family_id: Uuid,
    token_hash: String,
    expires_at: NaiveDateTime,
}
