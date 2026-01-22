use sqlx::{PgPool, QueryBuilder};
use chrono::Utc;
use crate::{model::transaction::{CreateTransactionDto, TransactionEntity, TransactionQueryDto, UpdateTransactionDto}, repository::{Error, RepositoryManager, Result}};

/// Transaction repository for database operations
pub struct TransactionRepository;

// TODO: Extract this logic into base repository

impl TransactionRepository {
    fn format_query(query: &TransactionQueryDto, query_builder: &mut QueryBuilder<'_, sqlx::Postgres>) {
        if let Some(cid) = query.cid {
            query_builder.push(" AND cid = ").push_bind(cid);
        }
        if let Some(title) = &query.title {
            if !title.trim().is_empty() {
                query_builder.push(" AND title ILIKE ").push_bind(format!("%{}%", title));
            }
        }
    }

    /// Count transactions matching filters
    async fn count_transactions(rm: RepositoryManager, query: &TransactionQueryDto) -> Result<i64> {
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM transaction WHERE 1=1");

        Self::format_query(query, &mut query_builder);

        let count: (i64,) = query_builder.build_query_as().fetch_one(rm.pool()).await.inspect_err(|e| {
            tracing::error!("Database error counting transactions: {:?}", e);
        })?;
        
        tracing::info!("transaction count: {:?}", count);

        Ok(count.0)
    }

    /// Find transactions with pagination and filters
    pub async fn find_with_pagination(
        rm: RepositoryManager,
        offset: i64,
        limit: i64,
        query: TransactionQueryDto,
    ) -> Result<(Vec<TransactionEntity>, i64)> {
        tracing::debug!("Finding transactions with pagination and filters: {:?}", query);
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

        let transactions = query_builder.build_query_as().fetch_all(rm.pool()).await.inspect_err(|e| {
            tracing::error!("Database error in transaction pagination: {:?}", e);
        })?;

        Ok((transactions, total))
    }

    /// Find transaction by ID
    pub async fn find_by_id(
        rm: RepositoryManager,
        id: i64,
    ) -> Result<Option<TransactionEntity>> {
        let result =
            sqlx::query_as::<_, TransactionEntity>("SELECT * FROM transaction WHERE id = $1")
                .bind(id)
                .fetch_optional(rm.pool())
                .await
                .map_err(|e| {
                    tracing::error!("Database error finding transaction by ID {}: {:?}", id, e);
                    Error::NotFound { entity: "transaction", id }
                })?;

        Ok(result)
    }

    /// Create new transaction
    pub async fn create_transaction(rm: RepositoryManager, dto: &CreateTransactionDto) -> Result<i64> {
        let mut tx = rm.pool().begin().await.inspect_err(|e| {
            tracing::error!("Database error starting transaction for transaction creation: {:?}", e);
        })?;

        // Create transaction
        let transaction_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO transaction (cid, title, amount, currency)
             VALUES ($1, $2, $3, $4)
             RETURNING id",
        )
        .bind(dto.cid)
        .bind(&dto.title)
        .bind(dto.value.amount)
        .bind(&dto.value.currency)
        .fetch_one(&mut *tx)
        .await
        .inspect_err(|e| {
            tracing::error!("Database error creating transaction: {:?}", e);
        })?;

        tx.commit().await.inspect_err(|e| {
            tracing::error!("Database error committing transaction: {:?}", e);
        })?;

        Ok(transaction_id)
    }

    /// Update existing transaction
    pub async fn update_transaction(
        rm: RepositoryManager,
        id: i64,
        dto: &UpdateTransactionDto
    ) -> Result<i64> {
        let mut tx = rm.pool().begin().await.inspect_err(|e| {
            tracing::error!("Database error starting transaction for transaction update: {:?}", e);
        })?;

        let transaction_id = sqlx::query_scalar::<_, i64>(
            "UPDATE transaction
            SET title = $1, amount = $2, currency = $3
            WHERE id = $4
            RETURNING id",
        )
        .bind(&dto.title)
        .bind(dto.value.amount)
        .bind(&dto.value.currency)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .inspect_err(|e| {
            tracing::error!("Database error updating transaction ID {}: {:?}", id, e);
        })?;

        if let Some(id) = transaction_id {
            tx.commit().await.inspect_err(|e| {
                tracing::error!("Database error committing transaction: {:?}", e);
            })?;
            Ok(id)
        } else {
            Err(Error::NotFound { entity: "transaction", id})
        }
    }

    /// Delete transaction
    pub async fn delete(rm: RepositoryManager, id: i64) -> Result<bool> {
        let result =
            sqlx::query("DELETE FROM transactions WHERE id = $1")
                .bind(id)
                .execute(rm.pool())
                .await
                .map_err(|e| {
                    tracing::error!("Database error deleting transaction ID {}: {:?}", id, e);
                    Error::NotFound { entity: "transaction", id }
                })?;

        Ok(result.rows_affected() > 0)
    }

    // TODO: soft delete
}