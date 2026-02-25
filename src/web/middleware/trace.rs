use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    http::{Method, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
    Json, Router,
};
use serde_json::json;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::{debug, info_span, Span};
use uuid::Uuid;

use crate::context::{CurrentUser, ErrorDetails};

// TODO: Implement logic to filter logs for dummy requests
// TODO: Add CatchPanicLayer to handle panics

/// Apply the global trace middleware to the router.
pub fn apply_trace_middleware(router: Router) -> Router {
    let middleware = ServiceBuilder::new()
        // Generates a UUID and inserts it into the `x-request-id` header
        // of the incoming request so all subsequent layers can read it.
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        // Reads the request ID we just generated and starts the timer.
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    // Extract the ID inserted by the previous layer
                    let request_id = request
                        .headers()
                        .get("x-request-id")
                        .and_then(|val| val.to_str().ok())
                        .unwrap_or("unknown");

                    // Create the Root Span. This attaches the request_id to EVERY
                    // log line emitted anywhere in your app during this request.
                    info_span!(
                        "http_request",
                        request_id = %request_id,
                        method = %request.method(),
                        uri = %request.uri().path(),
                        status_code = tracing::field::Empty,
                        latency_ms = tracing::field::Empty,
                        user_id = tracing::field::Empty,
                    )
                })
                .on_response(
                    |response: &Response<Body>, latency: Duration, span: &Span| {
                        let status = response.status();

                        // 1. Server Errors (5xx) -> Always ERROR
                        if status.is_server_error() {
                            if let Some(err) = response.extensions().get::<Arc<ErrorDetails>>() {
                                let chain_json = serde_json::to_string(&err.error_chain)
                                    .unwrap_or_else(|_| "[]".to_string());
                                tracing::error!(
                                    client_msg = %err.client_message,
                                    error_message = %err.error_message,
                                    error_type = %err.error_type,
                                    error_debug = %err.error_debug,
                                    err_chain = %chain_json,
                                    "Server error processing request"
                                );
                            } else {
                                // Fallback: Catches framework-generated 500s or panics
                                tracing::error!("Server error processing request");
                            }
                        }
                        // 2. Client Errors (4xx) -> Always WARN
                        else if status.is_client_error() {
                            if let Some(err) = response.extensions().get::<Arc<ErrorDetails>>() {
                                let chain_json = serde_json::to_string(&err.error_chain)
                                    .unwrap_or_else(|_| "[]".to_string());
                                tracing::warn!(
                                    client_msg = %err.client_message,
                                    error_message = %err.error_message,
                                    error_type = %err.error_type,
                                    error_debug = %err.error_debug,
                                    err_chain = %chain_json,
                                    "Client error processing request"
                                );
                            } else {
                                // Fallback: Catches 404 Not Found, 400 Bad Request from Axum Extractors, etc.
                                tracing::warn!("Client error processing request");
                            }
                        }
                        // 3. Success (2xx) -> INFO
                        else if status.is_success() {
                            tracing::info!("Request completed successfully");
                        }
                        // 4. Redirects & Others (3xx, etc.) -> DEBUG or INFO
                        else {
                            tracing::debug!("Request finished with non-standard status");
                        }
                    },
                )
                .on_request(())
                .on_failure(()),
        )
        // Automatically grabs the `x-request-id` from the incoming request
        // and attaches it to the HTTP headers of the outgoing Response.
        .layer(PropagateRequestIdLayer::x_request_id());

    router.layer(middleware)
}
