use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Money {
    pub amount: Decimal,
    pub currency: String, // e.g., "BRL", "EUR"
}
