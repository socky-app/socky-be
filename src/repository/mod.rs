//! Repository layer

use sqlx::PgPool;

use crate::repository::db::{create_pool, create_pool_from_config, test_connection};

mod db;
mod ops;
mod helper;
pub mod error;
pub mod repo_transaction;

pub use error::{Error, Result};
pub use db::DatabaseConfig;

#[derive(Clone)]
pub struct RepositoryManager {
    pool: PgPool,
}

impl RepositoryManager {
    pub async fn new() -> Result<Self> {
        let rm = RepositoryManager {
            pool: create_pool().await?,
        };

        test_connection(&rm.pool).await?;

        Ok(rm)
    }

    pub async fn new_from_config(config: &DatabaseConfig) -> Result<Self> {
        let rm = RepositoryManager {
            pool: create_pool_from_config(config).await?,
        };

        test_connection(&rm.pool).await?;

        Ok(rm)
    }

    pub async fn test_connection(&self) -> Result<()> {
        test_connection(&self.pool).await?;

        Ok(())
    }

    pub(in crate::repository) fn pool(&self) -> &PgPool {
		&self.pool
	}
}

