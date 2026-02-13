use crate::repository::ops::delete_strategy::Deletability;
use crate::repository::{RepositoryManager, Result};
use sqlx::{Executor, QueryBuilder};


pub mod create;
pub mod delete;
pub mod delete_strategy;
pub mod get;
pub mod soft_delete;
pub mod update;

/// Trait representing a database table
pub trait DatabaseTable {
    const TABLE: &'static str;
    type DeleteStrategy: Deletability;

    /// Helper to get the fragment of SQL needed for filtering soft-deletable tables
    fn push_where<'r>(id: i64, query_builder: &mut QueryBuilder<'r, sqlx::Postgres>) {
        query_builder.push("WHERE id = ").push_bind(id);

        if Self::DeleteStrategy::SUPPORTS_SOFT_DELETE {
            query_builder.push(" AND deleted_at IS NULL");
        }
    }

    /// Generic check for existence based on a column and value
    async fn exists_by_column(rm: &RepositoryManager, column: &str, value: &str) -> Result<bool> {
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
                tracing::error!(
                    "Database error checking existance on {}: {:?}",
                    Self::TABLE,
                    e
                );
            })?;

        Ok(exists)
    }
}
