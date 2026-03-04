use sqlx::PgPool;
use thiserror::Error;

use crate::config::DbConfig;

mod db;

#[derive(Error, Debug)]
pub enum RepositoryManagerError {
    #[error("failed to create DB pool")]
    CreatePoolFailed(#[source] sqlx::Error),
    #[error("failed to connect to DB")]
    ConnectionFailed(#[source] sqlx::Error),
}

impl RepositoryManagerError {
    pub fn inner(&self) -> &sqlx::Error {
        match self {
            RepositoryManagerError::CreatePoolFailed(error) => error,
            RepositoryManagerError::ConnectionFailed(error) => error,
        }
    }
}

#[derive(Clone)]
pub struct RepositoryManager {
    pool: PgPool,
}

impl RepositoryManager {
    pub async fn new(config: &DbConfig) -> Result<Self, RepositoryManagerError> {
        let rm = RepositoryManager {
            pool: db::create_pool(config).await?,
        };

        db::test_connection(&rm.pool).await?;

        Ok(rm)
    }

    pub async fn test_connection(&self) -> Result<(), RepositoryManagerError> {
        db::test_connection(&self.pool).await?;

        Ok(())
    }

    pub(in crate::repository) fn pool(&self) -> &PgPool {
        &self.pool
    }
}
