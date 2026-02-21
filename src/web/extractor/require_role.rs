use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{
    context::CurrentUser,
    model::user::{error::UserRoleError, UserRole},
    service::ServiceError,
    web::WebError,
};

/// A generic extractor that checks if the user's role meets the minimum required level.
pub struct RequireRole<const MIN_ROLE: i16>(pub CurrentUser);

impl<S, const MIN_ROLE: i16> FromRequestParts<S> for RequireRole<MIN_ROLE>
where
    S: Send + Sync,
{
    type Rejection = WebError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Get the base user
        let user = CurrentUser::from_request_parts(parts, state).await?;

        // 2. Cast the user's role enum to i16 and compare it to the generic requirement
        if (user.role as i16) >= MIN_ROLE {
            Ok(RequireRole(user))
        } else {
            tracing::warn!(
                "User {} (role: {:?}) blocked from route requiring role level: {}",
                user.id,
                user.role as i16,
                MIN_ROLE
            );
            Err(WebError::from(ServiceError::UserRole(UserRoleError {
                required: MIN_ROLE,
                current: user.role as i16,
            })))
        }
    }
}
