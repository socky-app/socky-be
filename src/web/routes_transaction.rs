use axum::Router;
use axum::extract::Path;
use axum::routing::{delete, post};
use axum::{extract::State, Json};

use crate::ctx::Ctx;
use crate::error::Result;
use crate::model::{ModelController, Transaction, TransactionForCreate};

pub fn routes(mc: ModelController) -> Router {
    Router::new()
        .route("/transaction", post(create_transaction).get(list_transactions))
        .route("/transaction/{id}", delete(delete_transaction).get(get_transaction))
        .with_state(mc)
    }

async fn create_transaction(
    State(mc): State<ModelController>,
    ctx: Ctx,
    Json(transaction_fc): Json<TransactionForCreate>,
) -> Result<Json<Transaction>> {
    println!("->> {:<12} - create_transaction", "HANDLER");

    let trasaction = mc.create_transaction(ctx, transaction_fc).await?;

    Ok(Json(trasaction))
}

async fn list_transactions(
    State(mc): State<ModelController>,
    ctx: Ctx,
) -> Result<Json<Vec<Transaction>>> {
    println!("->> {:<12} - list_transactions", "HANDLER");

    let trasactions = mc.list_transactions(ctx).await?;

    Ok(Json(trasactions))
}

async fn delete_transaction(
    State(mc): State<ModelController>,
    ctx: Ctx,
    Path(id): Path<u64>,
) -> Result<Json<Transaction>> {
    println!("->> {:<12} - delete_transaction", "HANDLER");

    let trasaction = mc.delete_transaction(ctx, id).await?;

    Ok(Json(trasaction))
}

async fn get_transaction(
    State(mc): State<ModelController>,
    ctx: Ctx,
    Path(id): Path<u64>,
) -> Result<Json<Transaction>> {
    println!("->> {:<12} - get_transaction", "HANDLER");

    let trasaction = mc.get_transaction(ctx, id).await?;

    Ok(Json(trasaction))
}