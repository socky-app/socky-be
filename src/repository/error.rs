use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("repository query failed")]
    DatabaseQueryFailed(#[from] sqlx::Error),
    #[error("could not find {entity} with id {id}")]
    NotFound { entity: &'static str, id: i64 },
    #[error("database consistency violation: {0}")]
    ConsistencyViolation(String),
}
