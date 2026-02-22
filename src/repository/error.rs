use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Database query failed")]
    DatabaseQueryFailed(#[from] sqlx::Error),
    #[error("Object not found: {entity} (id: {id})")]
    NotFound { entity: &'static str, id: i64 },
}
