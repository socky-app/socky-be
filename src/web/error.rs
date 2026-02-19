use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::{json, Value};
use thiserror::Error;

use crate::{
    model::user::error::UserError,
    repository::RepositoryError,
    service::{auth_controller::AuthError, ServiceError},
};

/// Web error.
#[derive(Debug, Error)]
#[error("{0}")]
pub struct WebError(pub ServiceError);

impl From<ServiceError> for WebError {
    fn from(inner: ServiceError) -> Self {
        WebError(inner)
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        tracing::debug!("{:<12} - {self}", "INTO_RES");

        let service_error = self.0;

        // Determine status code and user-facing message
        let (status_code, client_message) = get_status_code_and_message(&service_error);

        // Build the Public Response Body
        let client_error = ClientError {
            message: client_message.clone(),
        };

        // Build the Error Details
        let details = ErrorDetails {
            error_type: service_error.as_ref().into(),
            error_data: format!("{:?}", service_error),
            client_message,
        };

        // Create response and add details to it
        let mut response = (status_code, Json(client_error)).into_response();
        response.extensions_mut().insert(details);

        response
    }
}

/// Public JSON error structure
#[derive(Serialize)]
struct ClientError {
    message: String,
}

/// Error details used during request logging.
#[derive(Debug, Clone)]
pub struct ErrorDetails {
    pub error_type: String,
    pub error_data: String,
    pub client_message: String,
}

impl std::fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.client_message)
    }
}

/// Determine status code and user-facing message
fn get_status_code_and_message(service_error: &ServiceError) -> (StatusCode, String) {
    match service_error {
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
            AuthError::MissingToken
            | AuthError::InvalidToken
            | AuthError::ExpiredToken
            | AuthError::RevokedToken => (
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
