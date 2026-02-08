use chrono::Utc;

use crate::{
    model::refresh_token::{CreateRefreshTokenDto, RefreshTokenEntity},
    repository::{
        RepositoryError, RepositoryManager, Result, ops::{
            DatabaseTable, create::{Create, Insertable}, delete::Delete, delete_strategy::HardDeleteStrategy, get::Get
        }
    },
};

/// Token repository for database operations.
pub struct TokenRepository;

impl DatabaseTable for TokenRepository {
    const TABLE: &'static str = "refresh_token";
    type DeleteStrategy = HardDeleteStrategy;
}

impl Create for TokenRepository {
    type D = CreateRefreshTokenDto;
}

impl Get for TokenRepository {
    type T = RefreshTokenEntity;
}

impl Delete for TokenRepository {}

impl TokenRepository {
    pub async fn invalidate_token(rm: RepositoryManager, id: i64) -> Result<i64> {
        let ret_option = sqlx::query_scalar::<_, i64>(&format!(
            "UPDATE {} SET is_revoked = $1, updated_at = $2 WHERE id = $3 RETURNING id",
            Self::TABLE
        ))
        .bind(true)
        .bind(Utc::now().naive_utc())
        .bind(id)
        .fetch_optional(rm.pool())
        .await
        .inspect_err(|e| {
            tracing::error!(
                "Database error in invalidate_token, token_id={}: {:?}",
                id,
                e
            );
        })?;

        if let Some(ret_id) = ret_option {
            Ok(ret_id)
        } else {
            Err(RepositoryError::NotFound {
                entity: Self::TABLE,
                id,
            })
        }
    }
}

impl Insertable for CreateRefreshTokenDto {
    fn push_insert<'r>(&'r self, query_builder: &mut sqlx::QueryBuilder<'r, sqlx::Postgres>) {
        todo!()
    }
}
