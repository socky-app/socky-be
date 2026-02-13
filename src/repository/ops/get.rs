use std::future::Future;

use crate::repository::ops::DatabaseTable;
use crate::repository::{RepositoryError, RepositoryManager, Result};
use sqlx::postgres::Postgres;
use sqlx::{Executor, QueryBuilder};

pub trait Get: DatabaseTable + Sized {
    type T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin;

    fn get(rm: &RepositoryManager, id: i64) -> impl Future<Output = Result<Self::T>> + Send {
        get::<Self, _, _>(id, rm.pool())
    }
}

pub async fn get<'c, R, T, E>(id: i64, executor: E) -> Result<T>
where
    R: DatabaseTable,
    T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    E: Executor<'c, Database = Postgres> + Send,
{
    let mut query_builder = QueryBuilder::<Postgres>::new(&format!("SELECT * FROM {} ", R::TABLE));
    R::push_where(id, &mut query_builder);

    let ret_option = query_builder
        .build_query_as::<T>()
        .fetch_optional(executor)
        .await
        .inspect_err(|e| {
            tracing::error!("Database error getting {}: {:?}", R::TABLE, e);
        })?;

    if let Some(entity) = ret_option {
        Ok(entity)
    } else {
        Err(RepositoryError::NotFound {
            entity: R::TABLE,
            id,
        })
    }
}
