use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};

use crate::{
    config::AppConfig, context::CurrentUser, service::auth_controller::AuthController, web::Result,
};

pub async fn auth_middleware(
    State(app_config): State<Arc<AppConfig>>,
    mut request: Request,
    next: Next,
) -> Result<Response> {
    tracing::debug!("{:<15} - auth_middleware", "MIDDLEWARE>>");

    let (mut parts, body) = request.into_parts();

    let token = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    // Validate token
    let claims = AuthController::verify_access_token(token, &app_config.auth)?;

    // Create the user
    let current_user = CurrentUser::from(claims);

    // Insert into request extensions for dowstream handlers
    parts.extensions.insert(current_user.clone());

    let request = Request::from_parts(parts, body);
    let mut response = next.run(request).await;

    tracing::debug!("{:<15} - auth_middleware", "MIDDLEWARE<<");

    // Insert into response extensions for upstream middleware
    response.extensions_mut().insert(current_user);

    Ok(response)
}
