use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use tracing::{trace, Span};

use crate::{
    config::AppConfig,
    context::CurrentUser,
    controller::auth_controller::AuthController,
    web::{Result, WebError},
};

pub async fn auth_middleware(
    State(app_config): State<Arc<AppConfig>>,
    mut request: Request,
    next: Next,
) -> Result<Response> {
    trace!("Middleware auth ->");

    // Extract token cleanly
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(WebError::AuthorizationBearer)?;

    // Validate token
    let claims = AuthController::verify_access_token(token, &app_config.auth)?;

    // Create the user
    // TODO: Check if CurrentUser should be wrapped in and Arc
    let current_user = CurrentUser::from(claims);

    // Insert user info into current span
    Span::current().record("user_id", current_user.id);
    Span::current().record("user_role", current_user.role as i16);

    // Insert into request extensions for dowstream handlers
    request.extensions_mut().insert(current_user.clone());

    // Execute downstream handlers
    let mut response = next.run(request).await;

    trace!("Middleware auth <-");

    // Insert into response extensions for upstream middleware
    response.extensions_mut().insert(current_user);

    Ok(response)
}
