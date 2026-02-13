use sqlx::{Postgres, Transaction};


use crate::repository::RepositoryManager;

pub async fn _start_db_transaction<'a>(
    rm: &'a RepositoryManager,
) -> sqlx::Result<Transaction<'a, Postgres>> {
    rm.pool().begin().await.inspect_err(|e| {
        tracing::error!("Database error starting transaction: {:?}", e);
    })
}

pub async fn _commit_db_transaction(tx: Transaction<'_, Postgres>) -> sqlx::Result<()> {
    tx.commit().await.inspect_err(|e| {
        tracing::error!("Database error committing transaction: {:?}", e);
    })
}
