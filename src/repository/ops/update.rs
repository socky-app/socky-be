use std::future::Future;

use crate::repository::ops::DatabaseTable;
use crate::repository::{RepositoryError, RepositoryManager, Result};
use sqlx::postgres::Postgres;
use sqlx::{Executor, QueryBuilder};

pub trait Updatable {
    /// Push SET clause fragments for UPDATE (no leading "SET").
    /// Example: b.push("col = ").push_bind(&self.col).push(", ");
    fn push_update<'r>(&'r self, b: &mut QueryBuilder<'r, Postgres>);
}

// TODO: Fix this trait so that it is generic, not trait type,
// so that I can implement it for multiple DTOs.
pub trait Update: DatabaseTable + Sized {
    type D: Updatable + Sync;

    fn update(
        rm: &RepositoryManager,
        id: i64,
        dto: &Self::D,
    ) -> impl Future<Output = Result<i64>> + Send {
        update::<Self, _, _>(id, dto, rm.pool())
    }
}

pub async fn update<'c, R, D, E>(id: i64, dto: &D, executor: E) -> Result<i64>
where
    R: DatabaseTable,
    D: Updatable,
    E: Executor<'c, Database = sqlx::Postgres>,
{
    let mut query_builder = QueryBuilder::<Postgres>::new(&format!("UPDATE {} SET ", R::TABLE));
    dto.push_update(&mut query_builder);
    query_builder.push(" ");
    R::push_where(id, &mut query_builder);
    query_builder.push(" RETURNING id");

    let ret_option = query_builder
        .build_query_scalar::<i64>()
        .fetch_optional(executor)
        .await
        .inspect_err(|e| {
            tracing::error!("Database error updating {}: {:?}", R::TABLE, e);
        })?;

    if let Some(ret_id) = ret_option {
        Ok(ret_id)
    } else {
        Err(RepositoryError::NotFound {
            entity: R::TABLE,
            id,
        })
    }
}
