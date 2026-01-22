use serde::{Deserialize, Serialize};

use crate::model::money::Money;

/// Transaction definition.
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TransactionEntity {
    pub id: i64,
    pub cid: i64,
    pub title: String,
    #[sqlx(flatten)]
    pub value: Money,
}

/// Transaction query parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct TransactionQueryDto {
    pub cid: Option<i64>,
    pub title: Option<String>,
}

/// Transaction creation parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTransactionDto {
    pub cid: i64, // TODO: Remove later and use logged user
    pub title: String,
    pub value: Money,
}

/// Transaction update parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTransactionDto {
    pub title: Option<String>,
    pub value: Option<Money>,
}

/// Transaction for list display.
#[derive(Debug, Serialize)]
pub struct TransactionVo {
    pub id: i64,
    pub cid: i64,
    pub title: String,
    pub value: Money,
}

/// Transaction option
pub type TransactionOptionVo = Option<i64>;

impl From<TransactionEntity> for TransactionVo {
    fn from(value: TransactionEntity) -> Self {
        Self {
            id: value.id,
            cid: value.cid,
            title: value.title,
            value: value.value,
        }
    }
}