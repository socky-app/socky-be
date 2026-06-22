use axum::{extract::Request, middleware::Next, response::Response};
use tracing::trace;

use crate::{
    context::CurrentUser,
    controller::ControllerError,
    model::user::{error::UserStatusError, UserStatus},
    web::{Result, WebError},
};

pub async fn active_middleware(request: Request, next: Next) -> Result<Response> {
    trace!("Middleware active ->");

    // Extract current user from request extensions
    let user = request
        .extensions()
        .get::<CurrentUser>()
        .ok_or(WebError::UserExtraction)?;

    user.status
        .require_active()
        .map_err(WebError::StatusAuthorization)?;
    let response = next.run(request).await;

    trace!("Middleware active <-");

    Ok(response)
}
