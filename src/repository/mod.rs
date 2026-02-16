//! Repository layer

use sqlx::PgPool;

use crate::{
    config::DbConfig,
    repository::db::{create_pool, test_connection},
};

mod db;
mod error;
mod helper;
mod ops;

pub mod token_repo;
pub mod transaction_repo;
pub mod user_repo;

pub use error::RepositoryError;
pub use ops::{create::Create, delete::Delete, get::Get, soft_delete::SoftDelete, update::Update};

pub(in crate::repository) type Result<T> = core::result::Result<T, RepositoryError>;

#[derive(Clone)]
pub struct RepositoryManager {
    pool: PgPool,
}

impl RepositoryManager {
    pub async fn new(config: &DbConfig) -> Result<Self> {
        let rm = RepositoryManager {
            pool: create_pool(config).await?,
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
