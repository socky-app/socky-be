use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::{json, Value};
use thiserror::Error;

use crate::{
    context::ErrorDetails, model::user::error::{UserRoleError, UserStatusError}, repository::RepositoryError, service::{ServiceError, auth_controller::AuthError}
};

/// Web error.
#[derive(Debug, Error, strum_macros::AsRefStr)]
pub enum WebError {
    #[error(transparent)]
    Service(#[from] ServiceError),

    #[error("Current user missing in request parts")]
    UserExtraction,

    #[error("Insuficient permission: ")]
    UserRole(#[from] UserRoleError),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        tracing::debug!("{:<15} - {self}", "INTO_RES");

        let (status_code, client_message) = get_status_code_and_message(&self);

        // Build the Public Response Body
        let client_error = ClientError {
            message: client_message.clone(),
        };

        // Build the Error Details
        let details = ErrorDetails {
            error_type: self.as_ref().into(),
            error_data: format!("{:?}", self),
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

/// Determine status code and user-facing message
fn get_status_code_and_message(error: &WebError) -> (StatusCode, String) {
    match error {
        WebError::Service(service_error) => match service_error {
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
                    "Invalid email or password.".to_string(),
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

            ServiceError::UserStatus(e) => match e {
                UserStatusError::Disabled => (
                    StatusCode::FORBIDDEN,
                    "User account is disabled.".to_string(),
                ),
                UserStatusError::Pending => (
                    StatusCode::BAD_REQUEST,
                    "User account is pending activation.".to_string(),
                ),
                UserStatusError::Locked => (
                    StatusCode::BAD_REQUEST,
                    "User account is locked.".to_string(),
                ),
            },

            ServiceError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        }

        WebError::UserExtraction => (
            StatusCode::UNAUTHORIZED,
            "You must be logged in to access this resource.".to_string(),
        ),

        WebError::UserRole(_) => (
            StatusCode::FORBIDDEN,
            "Permission denied".to_string(),
        ),
    }
}
