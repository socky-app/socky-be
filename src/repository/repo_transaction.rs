use crate::{
    model::transaction::{
        CreateTransactionDto, TransactionEntity, TransactionQueryDto, UpdateTransactionDto,
    },
    repository::{
        crud,
        helper::{commit_db_transaction, start_db_transaction},
        Error, RepositoryManager, Result,
    },
};
use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};

/// Transaction repository for database operations
pub struct TransactionRepository;

impl crud::Crud for TransactionRepository {
    const TABLE: &'static str = "transaction";
}

impl TransactionRepository {
    fn format_query(
        query: &TransactionQueryDto,
        query_builder: &mut QueryBuilder<'_, sqlx::Postgres>,
    ) {
        if let Some(cid) = query.cid {
            query_builder.push(" AND cid = ").push_bind(cid);
        }
        if let Some(title) = &query.title {
            if !title.trim().is_empty() {
                query_builder
                    .push(" AND title ILIKE ")
                    .push_bind(format!("%{}%", title));
            }
        }
    }

    /// Count transactions matching filters
    async fn count_transactions(rm: RepositoryManager, query: &TransactionQueryDto) -> Result<i64> {
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM transaction WHERE 1=1");

        Self::format_query(query, &mut query_builder);

        let count: (i64,) = query_builder
            .build_query_as()
            .fetch_one(rm.pool())
            .await
            .inspect_err(|e| {
                tracing::error!("Database error counting transactions: {:?}", e);
            })?;

        tracing::info!("transaction count: {:?}", count);

        Ok(count.0)
    }

    /// List transactions with pagination and filters
    pub async fn list_with_pagination(
        rm: RepositoryManager,
        offset: i64,
        limit: i64,
        query: TransactionQueryDto,
    ) -> Result<(Vec<TransactionEntity>, i64)> {
        tracing::debug!(
            "Finding transactions with pagination and filters: {:?}",
            query
        );
        let total = Self::count_transactions(rm.clone(), &query).await?;
        if total == 0 {
            return Ok((Vec::new(), total));
        }

        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT * FROM transaction WHERE 1=1");

        Self::format_query(&query, &mut query_builder);

        query_builder.push(" ORDER BY created_at DESC");
        query_builder.push(" LIMIT ").push_bind(limit);
        query_builder.push(" OFFSET ").push_bind(offset);

        let transactions = query_builder
            .build_query_as()
            .fetch_all(rm.pool())
            .await
            .inspect_err(|e| {
                tracing::error!("Database error in transaction pagination: {:?}", e);
            })?;

        Ok((transactions, total))
    }

    // TODO: soft delete
}

impl crud::Create for TransactionRepository {
    type D = CreateTransactionDto;
}

impl crud::Get for TransactionRepository {
    type T = TransactionEntity;
}

impl crud::Update for TransactionRepository {
    type D = UpdateTransactionDto;
}

impl crud::Delete for TransactionRepository {}

impl crud::Insertable for CreateTransactionDto {
    fn push_insert<'r>(&'r self, query_builder: &mut QueryBuilder<'r, sqlx::Postgres>) {
        query_builder
            .push("(cid, title, amount, currency) VALUES (")
            .push_bind(self.cid)
            .push(", ")
            .push_bind(&self.title)
            .push(", ")
            .push_bind(self.value.amount)
            .push(", ")
            .push_bind(&self.value.currency)
            .push(")");
    }
}

impl crud::Updatable for UpdateTransactionDto {
    fn push_update<'q>(&'q self, b: &mut QueryBuilder<'q, sqlx::Postgres>) {
        let mut first = true;

        if let Some(title) = &self.title {
            if !first {
                b.push(", ");
            }
            b.push("title = ").push_bind(title);
            first = false;
        }

        if let Some(value) = &self.value {
            // assume value has fields `amount` and `currency`
            if !first {
                b.push(", ");
            }
            b.push("amount = ").push_bind(value.amount);
            b.push(", ");
            b.push("currency = ").push_bind(&value.currency);
            first = false;
        }

        // TODO: always update updated_at
        // if !first { b.push(", "); }
        // b.push("updated_at = ").push_bind(Utc::now());
    }
}
