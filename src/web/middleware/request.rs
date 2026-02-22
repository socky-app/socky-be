use axum::{
    extract::Request,
    http::{Method, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::debug;
use uuid::Uuid;

use crate::{context::{CurrentUser, ErrorDetails}, controller::log_controller::LogController};

pub async fn request_middleware(mut request: Request, next: Next) -> Response {
    debug!("{:<15} - request_middleware", "MIDDLEWARE>>"); // TODO: use spans for logging

    // Setup context before request
    let request_id = Uuid::new_v4();
    let method = request.method().to_string();
    let uri = request.uri().to_string();
    let start = std::time::Instant::now();

    // Insert request ID into extensions
    request.extensions_mut().insert(request_id);

    // Execute the request
    let mut response = next.run(request).await;
    debug!("{:<15} - request_middleware", "MIDDLEWARE<<");

    // Get request results
    let latency = start.elapsed();
    let status = response.status();
    let error_detail = response.extensions().get::<ErrorDetails>();
    let curr_user = response.extensions().get::<CurrentUser>();

    // Call log controller
    LogController::log_request(request_id.to_string(), method, uri, curr_user, error_detail).await;

    // Inject request ID into Headers
    response
        .headers_mut()
        .insert("x-request-id", request_id.to_string().parse().unwrap());

    response
}
