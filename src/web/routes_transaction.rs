use axum::extract::Path;
use axum::routing::{delete, post};
use axum::Router;
use axum::{extract::State, Json};
use tracing::debug;

use crate::ctx::Ctx;
use crate::error::Result;
use crate::model::transaction::{CreateTransactionDto, TransactionEntity, TransactionVo};
use crate::repository::RepositoryManager;

pub fn routes(mc: RepositoryManager) -> Router {
    Router::new()
        .route(
            "/transaction",
            post(create_transaction).get(list_transactions),
        )
        .route(
            "/transaction/{id}",
            delete(delete_transaction).get(get_transaction),
        )
        .with_state(mc)
}

async fn create_transaction(
    State(mc): State<RepositoryManager>,
    ctx: Ctx,
    Json(transaction_fc): Json<CreateTransactionDto>,
) -> Result<Json<TransactionVo>> {
    debug!("{:<12} - create_transaction", "HANDLER");

    todo!()
}

async fn list_transactions(
    State(mc): State<RepositoryManager>,
    ctx: Ctx,
) -> Result<Json<Vec<TransactionVo>>> {
    debug!("{:<12} - list_transactions", "HANDLER");

    todo!()
}

async fn delete_transaction(
    State(mc): State<RepositoryManager>,
    ctx: Ctx,
    Path(id): Path<i64>,
) -> Result<Json<()>> {
    debug!("{:<12} - delete_transaction", "HANDLER");

    todo!()
}

async fn get_transaction(
    State(mc): State<RepositoryManager>,
    ctx: Ctx,
    Path(id): Path<i64>,
) -> Result<Json<TransactionVo>> {
    debug!("{:<12} - get_transaction", "HANDLER");

    todo!()
}
