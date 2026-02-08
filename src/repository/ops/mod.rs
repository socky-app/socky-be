use crate::repository::{RepositoryError, RepositoryManager, Result};
use sqlx::postgres::Postgres;
use sqlx::{Executor, QueryBuilder};

use std::fmt::Debug;

pub mod create;
pub mod get;
pub mod update;
pub mod delete;
pub mod soft_delete;

pub trait DatabaseTable {
    const TABLE: &'static str;

    /// Generic check for existence based on a column and value
    async fn exists_by_column(
        rm: RepositoryManager, 
        column: &str, 
        value: &str
    ) -> Result<bool> {
        let sql = format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1 AND deleted_at IS NULL)",
            Self::TABLE,
            column
        );
        
        let exists = sqlx::query_scalar::<_, bool>(&sql)
            .bind(value)
            .fetch_one(rm.pool())
            .await
            .inspect_err(|e| {
                tracing::error!("Database error checking existance on {}: {:?}", Self::TABLE, e);
            })?;

        Ok(exists)
    }
}
