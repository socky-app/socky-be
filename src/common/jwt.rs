use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::config::AuthConfig;

// Using a consistent algorithm throughout
const SELECTED_ALGO: Algorithm = Algorithm::HS256;

/// Represents the claims in the access JWT payload.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessClaims {
    user_id: i64,
    aud: String,
    exp: usize,
    iat: usize,
}

impl AccessClaims {
    fn new(user_id: i64, expiration_seconds: i64) -> Self {
        let (exp, iat) = get_exp_iat(expiration_seconds);
        AccessClaims {
            user_id,
            aud: Self::AUDIENCE.to_string(),
            exp,
            iat,
        }
    }
}

/// Represents the claims in the refresh JWT payload.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshClaims {
    user_id: i64,
    family_id: Uuid,
    aud: String,
    exp: usize,
    iat: usize,
}

impl RefreshClaims {
    fn new(user_id: i64, family_id: Uuid, expiration_seconds: i64) -> Self {
        let (exp, iat) = get_exp_iat(expiration_seconds);
        RefreshClaims {
            user_id,
            family_id,
            aud: Self::AUDIENCE.to_string(),
            exp,
            iat,
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

/// Generate a pair of access and refresh tokens.
pub fn generate_tokens(
    user_id: i64,
    config: &AuthConfig,
) -> Result<(String, String), jsonwebtoken::errors::Error> {
    generate_tokens_with_family_id(user_id, Uuid::new_v4(), config)
}

/// Generate a pair of access and refresh tokens for a given family ID.
pub fn generate_tokens_with_family_id(
    user_id: i64,
    family_id: Uuid,
    config: &AuthConfig,
) -> Result<(String, String), jsonwebtoken::errors::Error> {
    let access_claims = AccessClaims::new(user_id, config.access_token_expiration_seconds);
    let refresh_claims =
        RefreshClaims::new(user_id, family_id, config.refresh_token_expiration_seconds);

    let access_token = generate_token(&access_claims, &config.access_token_secret)?;
    let refresh_token = generate_token(&refresh_claims, &config.refresh_token_secret)?;

    Ok((access_token, refresh_token))
}

/// Generates a JWT using any type that implements the Claims trait.
fn generate_token<T: Claims>(
    claims: &T,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let key = EncodingKey::from_secret(secret.as_bytes());
    let header = Header::new(SELECTED_ALGO);

    tracing::trace!(
        "Generating {} token for user_id: {}",
        T::AUDIENCE,
        claims.user_id()
    );

    encode(&header, claims, &key)
}

/// Validates a JWT using any type that implements the Claims trait.
pub fn validate_token<T: Claims>(
    token: &str,
    secret: &str,
) -> Result<T, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(SELECTED_ALGO);
    validation.set_audience(&[T::AUDIENCE]);

    let token_data = decode::<T>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    tracing::trace!(
        "Successfully verified {} token for user '{}'",
        T::AUDIENCE,
        token_data.claims.user_id()
    );

    Ok(token_data.claims)
}

fn get_exp_iat(expiration_seconds: i64) -> (usize, usize) {
    let iat = Utc::now().timestamp();
    let exp = iat + expiration_seconds;
    (exp as usize, iat as usize)
}
