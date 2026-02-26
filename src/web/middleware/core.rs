use std::{any::Any, sync::Arc};

use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
    Json, Router,
};
use serde_json::json;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::{debug, error, field, info, info_span, warn, Span};
use uuid::Uuid;

use crate::{
    context::{CurrentUser, ErrorDetails},
    web::WebError,
};

// TODO: Implement logic to filter logs for dummy requests

/// Apply the global trace middleware to the router.
pub fn apply_core_middleware(router: Router) -> Router {
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
                        status_code = field::Empty,
                        latency_ms = field::Empty,
                        user_id = field::Empty,
                        user_role = field::Empty,
                    )
                })
                .on_response(|response: &Response<_>, latency: Duration, span: &Span| {
                    let status = response.status();

                    span.record("status_code", response.status().as_u16());
                    span.record("latency_ms", latency.as_millis());

                    // Server Errors (5xx)
                    if status.is_server_error() {
                        if let Some(err) = response.extensions().get::<Arc<ErrorDetails>>() {
                            let chain_json = serde_json::to_string(&err.error_chain)
                                .unwrap_or_else(|_| "[]".to_string());
                            error!(
                                client_msg = %err.client_message,
                                error_message = %err.error_message,
                                error_type = %err.error_type,
                                error_debug = %err.error_debug,
                                err_chain = %chain_json,
                                "Server error processing request"
                            );
                        } else {
                            // Fallback: Catches framework-generated 500s or panics
                            error!("Server error processing request");
                        }
                    }
                    // Client Errors (4xx)
                    else if status.is_client_error() {
                        if let Some(err) = response.extensions().get::<Arc<ErrorDetails>>() {
                            let chain_json = serde_json::to_string(&err.error_chain)
                                .unwrap_or_else(|_| "[]".to_string());
                            warn!(
                                client_msg = %err.client_message,
                                error_message = %err.error_message,
                                error_type = %err.error_type,
                                error_debug = %err.error_debug,
                                err_chain = %chain_json,
                                "Client error processing request"
                            );
                        } else {
                            // Fallback: Catches 404 Not Found, 400 Bad Request from Axum Extractors, etc.
                            warn!("Client error processing request");
                        }
                    }
                    // Success (2xx)
                    else if status.is_success() {
                        info!("Request completed successfully");
                    }
                    // Redirects & Others (3xx, etc.)
                    else {
                        debug!("Request finished with non-standard status");
                    }
                })
                .on_request(())
                .on_failure(()),
        )
        // Automatically grabs the `x-request-id` from the incoming request
        // and attaches it to the HTTP headers of the outgoing Response.
        .layer(PropagateRequestIdLayer::x_request_id())
        // Catches panics from your handlers, and sends a WebError::Panic
        // back up to the TraceLayer.
        .layer(CatchPanicLayer::custom(handle_panic));

    router.layer(middleware)
}

/// Custom handler to gracefully format panics into JSON responses
fn handle_panic(err: Box<dyn Any + Send + 'static>) -> Response {
    // Try to extract the panic message
    let details = if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "Unknown panic".to_string()
    };

    WebError::Panic(details).into_response()
}
