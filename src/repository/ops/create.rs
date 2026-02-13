use std::future::Future;

use crate::repository::ops::DatabaseTable;
use crate::repository::{RepositoryManager, Result};
use sqlx::postgres::Postgres;
use sqlx::{Executor, QueryBuilder};

pub trait Insertable {
    /// Push VALUES bindings for INSERT
    fn push_insert<'r>(&'r self, query_builder: &mut QueryBuilder<'r, Postgres>);
}

pub trait Create: DatabaseTable + Sized {
    type D: Insertable + Sync + ?Sized;

    fn create(rm: &RepositoryManager, dto: &Self::D) -> impl Future<Output = Result<i64>> + Send {
        create::<Self, _, _>(dto, rm.pool())
    }
}

pub async fn create<'c, R, D, E>(dto: &D, executor: E) -> Result<i64>
where
    R: DatabaseTable,
    D: Insertable + ?Sized,
    E: Executor<'c, Database = Postgres> + Send,
{
    let mut query_builder = QueryBuilder::<Postgres>::new(&format!("INSERT INTO {} ", R::TABLE));
    dto.push_insert(&mut query_builder);
    query_builder.push(" RETURNING id");

    let id = query_builder
        .build_query_scalar::<i64>()
        .fetch_one(executor)
        .await
        .inspect_err(|e| {
            tracing::error!("Database error creating {}: {:?}", R::TABLE, e);
        })?;

    Ok(id)
}
