use axum::extract::Path;
use axum::routing::{delete, post};
use axum::Router;
use axum::{extract::State, Json};
use tracing::trace;

use crate::app::AppState;
use crate::model::transaction::{CreateTransactionDto, TransactionVo};
use crate::repository::RepositoryManager;
use crate::web::{ClientError, Result};

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

#[utoipa::path(
    post,
    path = "/api/transaction",
    tag = "Transaction",
    summary = "Create transaction",
    request_body = CreateTransactionDto,
    responses(
        (status = 200, description = "Transaction created", body = TransactionVo),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn create_transaction(
    State(_mc): State<RepositoryManager>,
    Json(_transaction_fc): Json<CreateTransactionDto>,
) -> Result<Json<TransactionVo>> {
    trace!("Handler transaction create");

    todo!()
}

#[utoipa::path(
    get,
    path = "/api/transaction",
    tag = "Transaction",
    summary = "List transactions",
    responses(
        (status = 200, description = "List of transactions", body = Vec<TransactionVo>),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn list_transactions(
    State(_mc): State<RepositoryManager>,
) -> Result<Json<Vec<TransactionVo>>> {
    trace!("Handler transaction list");

    todo!()
}

#[utoipa::path(
    delete,
    path = "/api/transaction/{id}",
    tag = "Transaction",
    summary = "Delete transaction",
    params(
        ("id" = i64, Path, description = "Transaction id")
    ),
    responses(
        (status = 200, description = "Transaction deleted"),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 404, description = "Not found", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn delete_transaction(
    State(_mc): State<RepositoryManager>,
    Path(_id): Path<i64>,
) -> Result<Json<()>> {
    trace!("Handler transaction delete");

    todo!()
}

#[utoipa::path(
    get,
    path = "/api/transaction/{id}",
    tag = "Transaction",
    summary = "Get transaction",
    params(
        ("id" = i64, Path, description = "Transaction id")
    ),
    responses(
        (status = 200, description = "Transaction found", body = TransactionVo),
        (status = 401, description = "Unauthorized", body = ClientError),
        (status = 403, description = "Forbidden", body = ClientError),
        (status = 404, description = "Not found", body = ClientError),
        (status = 500, description = "Internal server error", body = ClientError)
    ),
    security(
        ("bearer-jwt" = [])
    )
)]
async fn get_transaction(
    State(_mc): State<RepositoryManager>,
    Path(_id): Path<i64>,
) -> Result<Json<TransactionVo>> {
    trace!("Handler transaction get");

    todo!()
}
