use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

use crate::{
    context::ErrorDetails,
    controller::{auth_controller::AuthError, ControllerError},
    model::user::error::{UserRoleError, UserStatusError},
    repository::RepositoryError,
    web::health_router::HealthError,
};

/// Web error.
#[derive(Debug, Error)]
pub enum WebError {
    #[error(transparent)]
    Controller(#[from] ControllerError),

    #[error("missing authorization bearer")]
    AuthorizationBearer,

    #[error("current user missing in request parts")]
    UserExtraction,

    #[error("service health check failed")]
    Health(#[from] HealthError),

    #[error("insuficient permission")]
    UserRole(#[from] UserRoleError),

    #[error("{0}")]
    Panic(String),

    #[error("{0}")]
    NotFound(String),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        // Map error to status code and message
        let (status_code, client_message) = get_status_code_and_message(&self);

        // Build the Public Response Body
        let client_error = ClientError {
            message: client_message.clone(),
        };

        // Build the Error Details
        let details = ErrorDetails::new(&self, client_message);

        // Create response and add details to it
        let mut response = (status_code, Json(client_error)).into_response();
        response.extensions_mut().insert(Arc::new(details));

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
                RepositoryError::NotFound { entity: _, id: _ } => {
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
                AuthError::InvalidEmail { .. } | AuthError::InvalidPassword { .. } => (
                    StatusCode::UNAUTHORIZED,
                    "Invalid email or password.".to_string(),
                ),
                AuthError::NotFoundToken
                | AuthError::InvalidToken
                | AuthError::ExpiredToken
                | AuthError::RevokedToken { .. } => (
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

        WebError::AuthorizationBearer => (
            StatusCode::UNAUTHORIZED,
            "Invalid or expired token.".to_string(),
        ),

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

        WebError::Panic(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "An unexpected internal error occurred.".to_string(),
        ),

        WebError::NotFound(_) => (StatusCode::NOT_FOUND, "Resource not found.".to_string()),
    }
}
