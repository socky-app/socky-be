use crate::{
    model::user::UserRole,
    service::ServiceError,
    utils::token::{AccessClaims, Claims},
    web::WebError,
};
use axum::{extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Current authenticated user info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    pub id: i64,
    pub role: UserRole,
}

impl CurrentUser {
    pub fn new(id: i64, role: UserRole) -> Self {
        Self {
            id,
            role,
        }
    }
}

impl From<AccessClaims> for CurrentUser {
    fn from(value: AccessClaims) -> Self {
        Self {
            id: value.user_id(),
            role: value.user_role(),
        }
    }
}

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
            .ok_or_else(|| {
                tracing::error!(
                    "CurrentUser not found - auth middleware missing or user not authenticated"
                );
                WebError::from(ServiceError::CurrentUserExtractionError)
            })
    }
}
