use axum::http::StatusCode;
use thiserror::Error;

use crate::{
    model::user::error::UserError, repository::RepositoryError, service::auth_controller::AuthError,
};

#[derive(Debug, Error, strum_macros::AsRefStr)]
pub enum ServiceError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Auth error")]
    Auth(#[from] AuthError),

    #[error("Invalid user error")]
    User(#[from] UserError),

    #[error("Current user not in request parts")]
    CurrentUserExtractionError,

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl ServiceError {
    /// Determine status code and user-facing message
    pub fn get_status_code_and_message(&self) -> (StatusCode, String) {
        match self {
            ServiceError::Repository(e) => match e {
                RepositoryError::NotFound { entity, id } => {
                    (StatusCode::NOT_FOUND, "Resource not found".to_string())
                }
                RepositoryError::DatabaseQueryFailed(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Service is temporarily unavailable. Please try again later.".to_string(),
                ),
            },

            ServiceError::Auth(e) => match e {
                AuthError::HashingFailed => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Authentication processing failed.".to_string(),
                ),
                AuthError::InvalidLoginCredentials => (
                    StatusCode::UNAUTHORIZED,
                    "Invalid username or password.".to_string(),
                ),
                AuthError::InvalidToken | AuthError::ExpiredToken | AuthError::RevokedToken => (
                    StatusCode::UNAUTHORIZED,
                    "Invalid or expired token.".to_string(),
                ),
                AuthError::TokenCreationFailed => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to generate token.".to_string(),
                ),
            },

            ServiceError::User(e) => match e {
                UserError::InvalidUserStatus => (
                    StatusCode::BAD_REQUEST,
                    "User status is invalid.".to_string(),
                ),
                UserError::UserIsDisabled => (
                    StatusCode::FORBIDDEN,
                    "User account is disabled.".to_string(),
                ),
                UserError::UserIsPending => (
                    StatusCode::BAD_REQUEST,
                    "User account is pending activation.".to_string(),
                ),
                UserError::UserIsLocked => (
                    StatusCode::BAD_REQUEST,
                    "User account is locked.".to_string(),
                ),
            },

            ServiceError::CurrentUserExtractionError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),

            ServiceError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        }
    }
}
