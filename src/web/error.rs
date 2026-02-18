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
        let (status_code, user_message) = service_error.get_status_code_and_message();

        // Build the Public Response Body
        let client_error = ClientError {
            error: user_message.clone(),
        };

        // Build the Error Details
        let details = ErrorDetails {
            error_type: service_error.as_ref().into(),
            error_data: format!("{:?}", service_error),
            user_message: user_message,
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
    error: String,
}

/// Error details used during request logging.
#[derive(Debug, Clone)]
pub struct ErrorDetails {
    pub error_type: String,
    pub error_data: String,
    pub user_message: String,
}

impl std::fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.user_message)
    }
}
