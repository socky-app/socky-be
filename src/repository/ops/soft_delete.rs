use crate::repository::ops::DatabaseTable;
use crate::repository::{RepositoryError, RepositoryManager, Result};
use chrono::Utc;
use sqlx::postgres::Postgres;
use sqlx::{Executor, QueryBuilder};

use std::fmt::Debug;

pub trait SoftDelete: DatabaseTable + Sized {
    async fn soft_delete(rm: &RepositoryManager, id: i64) -> Result<()> {
        soft_delete::<Self, _>(id, rm.pool()).await
    }
}

pub async fn soft_delete<'c, R, E>(id: i64, executor: E) -> Result<()>
where
    R: DatabaseTable,
    E: Executor<'c, Database = sqlx::Postgres>,
{
    let query = sqlx::query_scalar::<_, i64>(
        "UPDATE users SET deleted_at = $1 WHERE id = $2 AND deleted_at IS NULL RETURNING id",
    )
    .bind(Utc::now().naive_utc())
    .bind(id);

    let deleted = query.fetch_all(executor).await.inspect_err(|e| {
        tracing::error!("Database error soft deleting {}: {:?}", R::TABLE, e);
    })?;

    if deleted.is_empty() {
        Err(RepositoryError::NotFound {
            entity: R::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}
