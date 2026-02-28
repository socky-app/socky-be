use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::debug;
use uuid::Uuid;

use crate::{config::AuthConfig, model::user::UserRole};

mod jwt;

pub use jwt::{validate_jwt, Claims};

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("failed to generate new token")]
    TokenCreationFailed,
    #[error("invalid token")]
    InvalidToken,
    #[error("token expired")]
    ExpiredToken,
}

/// Represents the claims in the access token payload.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessClaims {
    jti: Uuid,
    sub: i64,
    role: UserRole,
    exp: i64,
    iat: i64,
}

impl AccessClaims {
    pub fn new(user_id: i64, user_role: UserRole, iat: DateTime<Utc>, exp: DateTime<Utc>) -> Self {
        AccessClaims {
            jti: Uuid::new_v4(),
            sub: user_id,
            role: user_role,
            iat: iat.timestamp(),
            exp: exp.timestamp(),
        }
    }

    pub fn user_role(&self) -> UserRole {
        self.role
    }
}

impl Claims for AccessClaims {
    const TOKEN_TYPE: &str = "access";
    fn jti(&self) -> Uuid { self.jti }
    fn sub(&self) -> i64 { self.sub }
    fn iat(&self) -> i64 { self.iat }
    fn exp(&self) -> i64 { self.exp }
}

#[derive(Debug)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_id: Uuid,
    pub user_id: i64,
    pub family_id: Uuid,
    pub refresh_expires_at: NaiveDateTime,
}

/// Generate a pair of access and refresh tokens.
pub fn generate_tokens(
    user_id: i64,
    user_role: UserRole,
    config: &AuthConfig,
) -> Result<TokenPair, TokenError> {
    generate_tokens_with_family_id(user_id, user_role, Uuid::new_v4(), config)
}

/// Generate a pair of access and refresh tokens for a given family ID.
pub fn generate_tokens_with_family_id(
    user_id: i64,
    user_role: UserRole,
    family_id: Uuid,
    config: &AuthConfig,
) -> Result<TokenPair, TokenError> {
    // 1. Centralize time
    let now = Utc::now();
    tracing::info!("Generate token now: {}", now);
    let access_duration = Duration::seconds(config.access_token_expiration_seconds);
    let refresh_duration = Duration::seconds(config.refresh_token_expiration_seconds);

    // 2. Calculate expirations
    let access_expires_at = now + access_duration;
    let refresh_expires_at = now + refresh_duration;

    // 3. Create access token (JWT)
    let access_claims = AccessClaims::new(user_id, user_role, now, access_expires_at);
    let access_token =
        jwt::generate_jwt(&access_claims, config.access_token_secret.expose_secret())?;

    // 4. Create refresh token (Opaque String)
    let mut random_bytes = [0u8; 32]; // 256 bits of entropy
    rand::fill(&mut random_bytes);
    let refresh_token = URL_SAFE_NO_PAD.encode(random_bytes);

    // 5. Return complete package
    Ok(TokenPair {
        access_token,
        refresh_token,
        access_token_id: access_claims.jti,
        user_id,
        family_id,
        refresh_expires_at: refresh_expires_at.naive_utc(),
    })
}
