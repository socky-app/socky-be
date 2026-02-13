use crate::repository::ops::DatabaseTable;
use crate::repository::{RepositoryError, RepositoryManager, Result};
use sqlx::postgres::Postgres;
use sqlx::{Executor, QueryBuilder};


pub trait Delete: DatabaseTable + Sized {
    async fn delete(rm: &RepositoryManager, id: i64) -> Result<()> {
        delete::<Self, _>(id, rm.pool()).await
    }
}

pub async fn delete<'c, R, E>(id: i64, executor: E) -> Result<()>
where
    R: DatabaseTable,
    E: Executor<'c, Database = Postgres> + Send,
{
    let mut query_builder =
        QueryBuilder::<Postgres>::new(&format!("DELETE FROM {} WHERE id = ", R::TABLE));
    query_builder.push_bind(id).push(" RETURNING id");

    let deleted = query_builder
        .build_query_scalar::<i64>()
        .fetch_all(executor)
        .await
        .inspect_err(|e| {
            tracing::error!("Database error deleting {}: {:?}", R::TABLE, e);
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
