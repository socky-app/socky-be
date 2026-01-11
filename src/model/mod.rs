//! Simplistic model layer

use crate::{Error, Result, ctx::{self, Ctx}};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Money {
    pub amount: Decimal,
    pub currency: String, // e.g., "USD", "EUR"
}

#[derive(Clone, Debug, Serialize)]
pub struct Transaction {
    pub id: u64,
    pub cid: u64,
    pub title: String,
    pub value: Money,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TransactionForCreate {
    pub title: String,
    pub value: Money,
}

#[derive(Clone)]
pub struct ModelController {
    transactions_store: Arc<Mutex<Vec<Option<Transaction>>>>,
}

impl ModelController {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            transactions_store: Arc::default(),
        })
    }
}

impl ModelController {
    pub async fn create_transaction(
        &self,
        ctx: Ctx,
        transaction_fc: TransactionForCreate,
    ) -> Result<Transaction> {
        let mut store = self.transactions_store.lock().unwrap();

        let id = store.len() as u64;
        let transaction = Transaction {
            id,
            cid: ctx.user_id(),
            title: transaction_fc.title,
            value: transaction_fc.value,
        };
        store.push(Some(transaction.clone()));

        Ok(transaction)
    }

    pub async fn list_transactions(&self, _ctx: Ctx) -> Result<Vec<Transaction>> {
        let store = self.transactions_store.lock().unwrap();

        let transactions = store.iter().filter_map(|t| t.clone()).collect();

        Ok(transactions)
    }

    pub async fn delete_transaction(&self, _ctx: Ctx, transaction_id: u64) -> Result<Transaction> {
        let mut store = self.transactions_store.lock().unwrap();

        let transaction = store
            .get_mut(transaction_id as usize)
            .and_then(|t| t.take());

        transaction.ok_or(Error::TransactionDeleteFailIdNotFound { id: transaction_id })
    }

    pub async fn get_transaction(&self, _ctx: Ctx, transaction_id: u64) -> Result<Transaction> {
        let store = self.transactions_store.lock().unwrap();

        let transaction = store.get(transaction_id as usize).and_then(|t| t.clone());

        transaction.ok_or(Error::TransactionDeleteFailIdNotFound { id: transaction_id })
    }
}
