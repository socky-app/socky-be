use std::{error::Error, fmt::format};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::{json, Value};
use strum_macros::AsRefStr;
use thiserror::Error;

use crate::{
    common::ErrorType,
    context::ErrorDetails,
    controller::{auth_controller::AuthError, ControllerError},
    impl_error_type,
    model::user::error::{UserRoleError, UserStatusError},
    repository::RepositoryError,
    web::health_router::HealthError,
};

/// Web error.
#[derive(Debug, Error, AsRefStr)]
pub enum WebError {
    #[error("Controller: {0}")]
    Controller(#[from] ControllerError),

    #[error("Current user missing in request parts")]
    UserExtraction,

    #[error("Health: {0}")]
    Health(#[from] HealthError),

    #[error("Insuficient permission: {0}")]
    UserRole(#[from] UserRoleError),
}

impl_error_type!(WebError {
    delegate: [Controller, Health, UserRole],
    terminal: [UserExtraction]
});

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        tracing::debug!("{:<15} - {self}", "INTO_RES");

        let (status_code, client_message) = get_status_code_and_message(&self);

        // Build the Public Response Body
        let client_error = ClientError {
            message: client_message.clone(),
        };

        // Build the error data
        let mut error_data = format!("{:?}", self);
        let mut current_source = self.source();

        while let Some(cause) = current_source {
            error_data.push_str(&format!("\n  Caused by: {}", cause));
            current_source = cause.source();
        }

        // Build the Error Details
        let details = ErrorDetails {
            error_type: self.error_type(),
            error_data,
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
        WebError::Controller(controller_error) => match controller_error {
            ControllerError::Repository(e) => match e {
                RepositoryError::NotFound { entity, id } => {
                    (StatusCode::NOT_FOUND, "Resource not found.".to_string())
                }
                RepositoryError::DatabaseQueryFailed(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An unexpected internal error occurred.".to_string(),
                ),
            },

            ControllerError::Auth(e) => match e {
                AuthError::HashingFailed(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An unexpected internal error occurred.".to_string(),
                ),
                AuthError::InvalidEmail {..} | AuthError::InvalidPassword {..} => (
                    StatusCode::UNAUTHORIZED,
                    "Invalid email or password.".to_string(),
                ),
                AuthError::MissingToken
                | AuthError::NotFoundToken
                | AuthError::InvalidToken
                | AuthError::ExpiredToken
                | AuthError::RevokedToken {..} => (
                    StatusCode::UNAUTHORIZED,
                    "Invalid or expired token.".to_string(),
                ),
                AuthError::TokenCreationFailed => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An unexpected internal error occurred.".to_string(),
                ),
            },

            ControllerError::UserStatus(e) => match e {
                UserStatusError::Disabled => (
                    StatusCode::FORBIDDEN,
                    "User account is disabled.".to_string(),
                ),
                UserStatusError::Pending => (
                    StatusCode::FORBIDDEN,
                    "User account is pending activation.".to_string(),
                ),
                UserStatusError::Locked => {
                    (StatusCode::FORBIDDEN, "User account is locked.".to_string())
                }
            },

            ControllerError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected internal error occurred.".to_string(),
            ),
        },

        // We return internal server error here because this indicates a misconfigured route,
        // that should be under the auth_middleware.
        WebError::UserExtraction => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "An unexpected internal error occurred.".to_string(),
        ),

        WebError::UserRole(_) => (StatusCode::FORBIDDEN, "Permission denied.".to_string()),

        WebError::Health(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Service is temporarily unavailable.".to_string(),
        ),
    }
}
