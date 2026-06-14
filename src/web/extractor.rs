//! Extractor types to handle role access.
//!
//! Use `axum::middleware::from_extractor::<T>()` to create middlewares when multiple routes
//! have the same permissions, or use the extractor directly on the route if more
//! granular access is required.

use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{context::CurrentUser, model::user::UserRole, web::WebError};

mod require_role;

pub type RequireStandard = require_role::RequireRole<{ UserRole::Standard as i16 }>;
pub type RequireSupport = require_role::RequireRole<{ UserRole::Support as i16 }>;
pub type RequireAdmin = require_role::RequireRole<{ UserRole::Admin as i16 }>;

/// Axum extractor for CurrentUser
///
/// Usage: async fn handler(current_user: CurrentUser) -> Response
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = WebError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or(WebError::UserExtraction)
    }
}
