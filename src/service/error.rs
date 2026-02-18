use serde::Serialize;
use thiserror::Error;

use crate::{
    model::user::error::UserError,
    repository::RepositoryError,
    service::auth_controller::AuthError,
    utils::{hmac, password, token},
};

#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ServiceError {
    #[error("Object not found: {entity} (id: {id})")]
    NotFound { entity: String, id: String },

    #[error("Database query failed: {0}")]
    DatabaseQueryFailed(String),

    #[error("Auth error")]
    Auth(#[from] AuthError),

    #[error("Invalid user error")]
    User(#[from] UserError),

    #[error("Current user not in request parts")]
    CurrentUserExtractionError,

    #[error("Internal server error: {0}")]
    Internal(String),
}

// From implementation for repository error

impl From<RepositoryError> for ServiceError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::NotFound { entity, id } => ServiceError::NotFound {
                entity: entity.to_string(),
                id: id.to_string(),
            },
            RepositoryError::DatabaseQueryFailed(e) => {
                ServiceError::DatabaseQueryFailed(e.to_string())
            }
        }
    }
}

// From implementation for utils errors

impl From<token::TokenError> for ServiceError {
    fn from(error: token::TokenError) -> Self {
        match error {
            token::TokenError::InvalidToken => ServiceError::Auth(AuthError::InvalidToken),
            token::TokenError::ExpiredToken => ServiceError::Auth(AuthError::ExpiredToken),
            token::TokenError::TokenCreationFailed => {
                ServiceError::Internal("Token creation failed".to_string())
            }
        }
    }
}

impl From<hmac::InvalidLength> for ServiceError {
    fn from(_: hmac::InvalidLength) -> Self {
        ServiceError::Internal("HMAC failed due to invalid key length".to_string())
    }
}

impl From<password::PasswordError> for ServiceError {
    fn from(error: password::PasswordError) -> Self {
        match error {
            password::PasswordError::PasswordHashingFailed => {
                ServiceError::Internal("Password hashing failed".to_string())
            }
        }
    }
}
