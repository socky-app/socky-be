use std::fmt;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::{json, Value};
use thiserror::Error;

use crate::service::ServiceError;

/// Web error.
#[derive(Debug, Error)]
#[error("{status_code}: {details}")]
pub struct WebError {
    status_code: StatusCode,
    details: ErrorDetails,
}

impl From<ServiceError> for WebError {
    fn from(service_error: ServiceError) -> Self {
        let (status_code, user_message) = match &service_error {
            // Auth
            ServiceError::Auth(_e) => (StatusCode::FORBIDDEN, "Authentication failed".into()),

            // Default
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".into(),
            ),
        };

        WebError {
            status_code,
            details: (service_error, user_message).into(),
        }
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        tracing::debug!("{:<12} - {self}", "INTO_RES");

        let WebError {
            status_code,
            details,
        } = self;

        // Build the Public Response Body
        let body = Json(json!({
            "error": details.user_message
        }));

        // Create response and add details to it
        let mut response = (status_code, body).into_response();
        response.extensions_mut().insert(details);

        response
    }
}

/// Error details used during request logging.
#[derive(Debug, Clone)]
pub struct ErrorDetails {
    pub error_type: String,
    pub error_data: Value,
    pub user_message: String,
}

impl From<(ServiceError, String)> for ErrorDetails {
    fn from((service_error, user_message): (ServiceError, String)) -> Self {
        // Serialize the error to JSON
        let mut json_value = serde_json::to_value(service_error).unwrap_or(Value::Null);

        // Extract "type" (The variant name)
        let error_type = json_value
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        // Extract "data" (The inner details)
        let error_data = json_value
            .get_mut("data")
            .map(|v| v.take())
            .unwrap_or(Value::Null);

        ErrorDetails {
            error_type,
            error_data,
            user_message,
        }
    }
}

impl std::fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message)
    }
}
