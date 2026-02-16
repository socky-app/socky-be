use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::config::AuthConfig;

// Using a consistent algorithm throughout
const SELECTED_ALGO: Algorithm = Algorithm::HS256;

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Failed to generate new token")]
    TokenCreationFailed,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    ExpiredToken,
}

/// Represents the claims in the access token payload.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessClaims {
    user_id: i64,
    aud: String,
    exp: usize,
    iat: usize,
}

impl AccessClaims {
    pub fn new(user_id: i64, iat: DateTime<Utc>, exp: DateTime<Utc>) -> Self {
        AccessClaims {
            user_id,
            aud: Self::AUDIENCE.to_string(),
            // Safe cast: Postgres/Chrono timestamps fit in usize on 64-bit systems
            // or just use u64/i64 for claims to be safe
            iat: iat.timestamp() as usize,
            exp: exp.timestamp() as usize,
        }
    }
}

/// Represents the claims in the refresh token payload.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshClaims {
    user_id: i64,
    family_id: Uuid,
    aud: String,
    exp: usize,
    iat: usize,
}

impl RefreshClaims {
    pub fn new(user_id: i64, family_id: Uuid, iat: DateTime<Utc>, exp: DateTime<Utc>) -> Self {
        RefreshClaims {
            user_id,
            family_id,
            aud: Self::AUDIENCE.to_string(),
            iat: iat.timestamp() as usize,
            exp: exp.timestamp() as usize,
        }
    }

    pub fn family_id(&self) -> &Uuid {
        &self.family_id
    }
}

pub trait Claims: Serialize + for<'de> Deserialize<'de> {
    const AUDIENCE: &str;
    fn user_id(&self) -> i64;
}

impl Claims for AccessClaims {
    const AUDIENCE: &str = "socky_access";
    fn user_id(&self) -> i64 {
        self.user_id
    }
}

impl Claims for RefreshClaims {
    const AUDIENCE: &str = "socky_refresh";
    fn user_id(&self) -> i64 {
        self.user_id
    }
}

#[derive(Debug)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub access_expires_at: NaiveDateTime,
    pub refresh_expires_at: NaiveDateTime,
    pub family_id: Uuid, // <--- Add this back!
}

/// Generate a pair of access and refresh tokens.
pub fn generate_tokens(user_id: i64, config: &AuthConfig) -> Result<TokenPair, TokenError> {
    generate_tokens_with_family_id(user_id, Uuid::new_v4(), config)
}

/// Generate a pair of access and refresh tokens for a given family ID.
pub fn generate_tokens_with_family_id(
    user_id: i64,
    family_id: Uuid,
    config: &AuthConfig,
) -> Result<TokenPair, TokenError> {
    // 1. Centralize Time
    let now = Utc::now();
    let access_duration = Duration::seconds(config.access_token_expiration_seconds);
    let refresh_duration = Duration::seconds(config.refresh_token_expiration_seconds);

    // 2. Calculate Expirations
    let access_expires_at = now + access_duration;
    let refresh_expires_at = now + refresh_duration;

    // 3. Create Claims
    let access_claims = AccessClaims::new(user_id, now, access_expires_at);
    let refresh_claims = RefreshClaims::new(user_id, family_id, now, refresh_expires_at);

    // 4. Encode
    let access_token = generate_token(&access_claims, config.access_token_secret.expose_secret())?;
    let refresh_token =
        generate_token(&refresh_claims, config.refresh_token_secret.expose_secret())?;

    // 5. Return complete package
    Ok(TokenPair {
        access_token,
        refresh_token,
        access_expires_at: access_expires_at.naive_utc(),
        refresh_expires_at: refresh_expires_at.naive_utc(),
        family_id,
    })
}

/// Generates a token using any type that implements the Claims trait.
fn generate_token<T: Claims>(claims: &T, secret: &str) -> Result<String, TokenError> {
    let key = EncodingKey::from_secret(secret.as_bytes());
    let header = Header::new(SELECTED_ALGO);

    tracing::trace!(
        "Generating {} token for user_id: {}",
        T::AUDIENCE,
        claims.user_id()
    );

    encode(&header, claims, &key).map_err(|_| TokenError::TokenCreationFailed)
}

/// Validates a token using any type that implements the Claims trait.
pub fn validate_token<T: Claims>(token: &str, secret: &str) -> Result<T, TokenError> {
    let mut validation = Validation::new(SELECTED_ALGO);
    validation.set_audience(&[T::AUDIENCE]);

    let token_data = decode::<T>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| match e.into_kind() {
        ErrorKind::ExpiredSignature => TokenError::ExpiredToken,
        _ => TokenError::InvalidToken,
    })?;

    tracing::trace!(
        "Successfully verified {} token for user '{}'",
        T::AUDIENCE,
        token_data.claims.user_id()
    );

    Ok(token_data.claims)
}
