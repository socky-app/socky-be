use crate::{
    model::refresh_token::{CreateRefreshTokenDto, RefreshTokenEntity},
    repository::{
        ops::{
            create::{Create, Insertable},
            delete::Delete,
            delete_strategy::HardDeleteStrategy,
            get::Get,
            DatabaseTable,
        },
        RepositoryError, RepositoryManager, Result,
    },
};

/// Token repository for database operations.
pub struct TokenRepository;

impl DatabaseTable for TokenRepository {
    const TABLE: &'static str = "refresh_token";
    type DeleteStrategy = HardDeleteStrategy;
}

impl Create for TokenRepository {
    type D<'a> = CreateRefreshTokenDto<'a>;
}

impl Get for TokenRepository {
    type T = RefreshTokenEntity;
}

impl Delete for TokenRepository {}

impl TokenRepository {
    pub async fn invalidate_token(rm: &RepositoryManager, id: i64) -> Result<i64> {
        let ret_option = sqlx::query_scalar::<_, i64>(&format!(
            "UPDATE {} SET is_revoked = $1 WHERE id = $2 RETURNING id",
            Self::TABLE
        ))
        .bind(true)
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

impl Insertable for CreateRefreshTokenDto<'_> {
    fn push_insert<'r>(&'r self, query_builder: &mut sqlx::QueryBuilder<'r, sqlx::Postgres>) {
        query_builder
            .push("(user_id, family_id, token_hash, expires_at) VALUES (")
            .push_bind(self.user_id)
            .push(", ")
            .push_bind(self.family_id)
            .push(", ")
            .push_bind(self.token_hash)
            .push(", ")
            .push_bind(self.expires_at)
            .push(")");
    }
}
