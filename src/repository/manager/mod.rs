use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use sqlx::PgPool;
use thiserror::Error;

use crate::config::DbConfig;

mod db;

#[serde_as]
#[derive(Error, Debug, Serialize)]
pub enum RepositoryManagerError {
    #[error("Failed to create DB pool")]
    CreatePoolFailed(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
    #[error("Failed to connect to DB")]
    ConnectionFailed(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
}

#[derive(Clone)]
pub struct RepositoryManager {
    pool: PgPool,
}

impl RepositoryManager {
    pub async fn new(config: &DbConfig) -> Result<Self, RepositoryManagerError> {
        // If just re-exporting, would be better to change the return
        // here as well, to avoid using the internal repo error for this.
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