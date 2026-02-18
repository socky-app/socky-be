use axum::extract::Path;
use axum::routing::{delete, post};
use axum::Router;
use axum::{extract::State, Json};

use crate::app::AppState;
use crate::model::transaction::{CreateTransactionDto, TransactionVo};
use crate::repository::RepositoryManager;
use crate::web::Result;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route(
            "/transaction",
            post(create_transaction).get(list_transactions),
        )
        .route(
            "/transaction/{id}",
            delete(delete_transaction).get(get_transaction),
        )
        .with_state(app_state)
}

async fn create_transaction(
    State(_mc): State<RepositoryManager>,
    Json(_transaction_fc): Json<CreateTransactionDto>,
) -> Result<Json<TransactionVo>> {
    tracing::debug!("{:<12} - create_transaction", "HANDLER");

    todo!()
}

async fn list_transactions(
    State(_mc): State<RepositoryManager>,
) -> Result<Json<Vec<TransactionVo>>> {
    tracing::debug!("{:<12} - list_transactions", "HANDLER");

    todo!()
}

async fn delete_transaction(
    State(_mc): State<RepositoryManager>,
    Path(_id): Path<i64>,
) -> Result<Json<()>> {
    tracing::debug!("{:<12} - delete_transaction", "HANDLER");

    todo!()
}

async fn get_transaction(
    State(_mc): State<RepositoryManager>,
    Path(_id): Path<i64>,
) -> Result<Json<TransactionVo>> {
    tracing::debug!("{:<12} - get_transaction", "HANDLER");

    todo!()
}
