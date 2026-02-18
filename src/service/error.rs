use serde::Serialize;
use thiserror::Error;

use crate::{
    model::user::error::UserError,
    repository::RepositoryError,
    service::auth_controller::AuthError,
};

#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ServiceError {
    #[error("Object not found: {entity} (id: {id})")]
    NotFound { entity: String, id: String },

    #[error("Database query failed: {0}")]
    DatabaseQueryFailed(String),

    #[error("Auth error")]
    Auth(#[from] AuthError),

    #[error("Invalid user error")]
    User(#[from] UserError),

    #[error("Current user not in request parts")]
    CurrentUserExtractionError,

    #[error("Internal server error: {0}")]
    Internal(String),
}

// From implementation for repository error

impl From<RepositoryError> for ServiceError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::NotFound { entity, id } => ServiceError::NotFound {
                entity: entity.to_string(),
                id: id.to_string(),
            },
            RepositoryError::DatabaseQueryFailed(e) => {
                ServiceError::DatabaseQueryFailed(e.to_string())
            }
        }
    }
}
