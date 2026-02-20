use crate::{
    model::user::UserRole, service::ServiceError, utils::token::{AccessClaims, Claims}, web::WebError
};
use axum::{extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Current authenticated user info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    pub user_id: i64,
    pub user_role: UserRole,
    pub family_id: Uuid
}

impl CurrentUser {
    pub fn new(user_id: i64, user_role: UserRole, family_id: Uuid) -> Self {
        Self { user_id, user_role, family_id}
    }
}

impl From<AccessClaims> for CurrentUser {
    fn from(value: AccessClaims) -> Self {
        Self {
            user_id: value.user_id(),
            user_role: value.user_role(),
            family_id: *value.family_id(),
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
