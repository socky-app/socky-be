use std::future::Future;

use crate::repository::ops::delete_strategy::SoftDeleteStrategy;
use crate::repository::ops::DatabaseTable;
use crate::repository::{RepositoryError, RepositoryManager, Result};
use chrono::Utc;
use sqlx::postgres::Postgres;
use sqlx::{Executor, QueryBuilder};


pub trait SoftDelete: DatabaseTable<DeleteStrategy = SoftDeleteStrategy> + Sized {
    fn soft_delete(rm: &RepositoryManager, id: i64) -> impl Future<Output = Result<()>> + Send {
        soft_delete::<Self, _>(id, rm.pool())
    }
}

pub async fn soft_delete<'c, R, E>(id: i64, executor: E) -> Result<()>
where
    R: DatabaseTable<DeleteStrategy = SoftDeleteStrategy>,
    E: Executor<'c, Database = sqlx::Postgres>,
{
    let mut query_builder = QueryBuilder::<Postgres>::new(&format!("UPDATE {} SET ", R::TABLE));
    query_builder
        .push("deleted_at = ")
        .push_bind(Utc::now().naive_utc())
        .push(" ");
    R::push_where(id, &mut query_builder);
    query_builder.push(" RETURNING id");

    let deleted = query_builder
        .build_query_scalar::<i64>()
        .fetch_all(executor)
        .await
        .inspect_err(|e| {
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
