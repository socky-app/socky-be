use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::ErrorKind};
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

use crate::utils::token::TokenError;

// Using a consistent algorithm throughout
const SELECTED_ALGO: Algorithm = Algorithm::HS256;

pub trait Claims: Serialize + for<'de> Deserialize<'de> {
    const TOKEN_TYPE: &str;
    fn jti(&self) -> Uuid;
    fn sub(&self) -> i64;
    fn iat(&self) -> i64;
    fn exp(&self) -> i64;
}

// Generates a token using any type that implements the Claims trait.
#[tracing::instrument(name = "generate_jwt", skip_all)]
pub fn generate_jwt<T: Claims>(claims: &T, secret: &str) -> Result<String, TokenError> {
    let key = EncodingKey::from_secret(secret.as_bytes());
    let header = Header::new(SELECTED_ALGO);

    let token = encode(&header, claims, &key).map_err(|_| TokenError::TokenCreationFailed)?;
    
    debug!(
        sub = %claims.sub(),
        token_type = %T::TOKEN_TYPE,
        "Token generated",
    );

    Ok(token)
}

/// Validates a token using any type that implements the Claims trait.
#[tracing::instrument(name = "validate_jwt", skip_all)]
pub fn validate_jwt<T: Claims>(token: &str, secret: &str) -> Result<T, TokenError> {
    let mut validation = Validation::new(SELECTED_ALGO);
    validation.set_audience(&[T::TOKEN_TYPE]);

    let token_data = decode::<T>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| match e.into_kind() {
        ErrorKind::ExpiredSignature => TokenError::ExpiredToken,
        _ => TokenError::InvalidToken,
    })?;

    debug!(
        sub = %token_data.claims.sub(),
        token_type = %T::TOKEN_TYPE,
        "Token validated",
    );

    Ok(token_data.claims)
}
