use crate::repository::{Error, Result};
use sqlx::postgres::Postgres;
use sqlx::{Executor, QueryBuilder};

use std::fmt::Debug;

pub trait Crud {
    const TABLE: &'static str;
}

pub trait Insertable {
    /// Push VALUES bindings for INSERT
    fn push_insert<'r>(&'r self, query_builder: &mut QueryBuilder<'r, Postgres>);
}

pub trait Updatable {
    /// Push SET clause fragments for UPDATE (no leading "SET").
    /// Example: b.push("col = ").push_bind(&self.col).push(", ");
    fn push_update<'r>(&'r self, b: &mut QueryBuilder<'r, Postgres>);
}

pub async fn create<'c, R, D, E>(dto: &D, executor: E) -> Result<i64>
where
    R: Crud,
    D: Insertable + ?Sized,
    E: Executor<'c, Database = Postgres> + Send,
{
    let mut query_builder =
        QueryBuilder::<Postgres>::new(&format!("INSERT INTO {} ", R::TABLE));
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

pub async fn get<'c, R, T, E>(id: i64, executor: E) -> Result<T>
where
    R: Crud,
    T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    E: Executor<'c, Database = Postgres> + Send,
{
    let mut query_builder =
        QueryBuilder::<Postgres>::new(&format!("SELECT * FROM {} WHERE id = ", R::TABLE));
    query_builder.push_bind(id);

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
        Err(Error::NotFound {
            entity: R::TABLE,
            id,
        })
    }
}

pub async fn update<'c, R, D, E>(id: i64, dto: &D, executor: E) -> Result<i64>
where
    R: Crud,
    D: Updatable,
    E: Executor<'c, Database = sqlx::Postgres>,
{
    let mut query_builder = QueryBuilder::<Postgres>::new(&format!("UPDATE {} SET ", R::TABLE));
    dto.push_update(&mut query_builder);
    query_builder
        .push(" WHERE id = ")
        .push_bind(id)
        .push(" RETURNING id");

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
        Err(Error::NotFound {
            entity: R::TABLE,
            id,
        })
    }
}

pub async fn delete<'c, R, E>(id: i64, executor: E) -> Result<()>
where
    R: Crud,
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
        Err(Error::NotFound {
            entity: R::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}
