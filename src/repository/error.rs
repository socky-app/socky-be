use strum_macros::AsRefStr;
use thiserror::Error;

use crate::common::ErrorType;

#[derive(Debug, Error, AsRefStr)]
pub enum RepositoryError {
    #[error("Database query failed")]
    DatabaseQueryFailed(#[from] sqlx::Error),
    #[error("Object not found: {entity} (id: {id})")]
    NotFound { entity: &'static str, id: i64 },
}

impl ErrorType for RepositoryError {}
