use sqlx::{Executor, Postgres};
use uuid::Uuid;

use crate::{
    model::refresh_token::{CreateRefreshTokenDto, RefreshTokenEntity, RevokedTokenEntity},
    repository::{
        helper::{commit_db_transaction, start_db_transaction},
        ops::{
            create::{create, Create, Insertable},
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
    const TABLE: &'static str = "refresh_tokens";
    // Note: The refresh_token table must not implement SoftDeleteStrategy,
    // otherwise the logic in get_by_hash and revoke_family would have to change.
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
    pub async fn get_by_hash(
        rm: &RepositoryManager,
        hash: &[u8],
    ) -> Result<Option<RefreshTokenEntity>> {
        let result = sqlx::query_as::<_, RefreshTokenEntity>(&format!(
            "SELECT * FROM {} WHERE token_hash = $1",
            Self::TABLE
        ))
        .bind(hash)
        .fetch_optional(rm.pool())
        .await?;

        Ok(result)
    }

    pub async fn revoke_family(rm: &RepositoryManager, family_id: &Uuid) -> Result<()> {
        sqlx::query(&format!(
            "UPDATE {} SET is_revoked = $1 WHERE family_id = $2",
            Self::TABLE
        ))
        .bind(true)
        .bind(family_id)
        .execute(rm.pool())
        .await?;

        Ok(())
    }

    pub async fn revoke_family_by_hash(
        rm: &RepositoryManager,
        token_hash: &[u8],
    ) -> Result<Option<RevokedTokenEntity>> {
        let result = sqlx::query_as!(
            RevokedTokenEntity,
            r#"
            WITH target_token AS (
                SELECT
                    id,
                    user_id,
                    family_id,
                    is_revoked AS was_already_revoked
                FROM refresh_tokens
                WHERE token_hash = $1
                LIMIT 1
            ),
            update_family AS (
                UPDATE refresh_tokens
                SET is_revoked = true
                WHERE family_id = (SELECT family_id FROM target_token)
                AND is_revoked = false 
            )
            -- Return the initial state captured in the first CTE
            SELECT
                id,
                user_id,
                family_id,
                was_already_revoked
            FROM target_token;
            "#,
            token_hash
        )
        .fetch_optional(rm.pool())
        .await?;

        Ok(result)
    }

    pub async fn rotate(
        rm: &RepositoryManager,
        dto: &CreateRefreshTokenDto<'_>,
        old_token_id: i64,
    ) -> Result<i64> {
        let mut tx = start_db_transaction(rm).await?;

        Self::revoke(old_token_id, &mut *tx).await?;
        let new_token_id = create::<Self, _, _>(dto, &mut *tx).await?;

        commit_db_transaction(tx).await?;

        Ok(new_token_id)
    }

    async fn revoke<'c, E>(id: i64, executor: E) -> Result<i64>
    where
        E: Executor<'c, Database = Postgres> + Send,
    {
        let ret_option = sqlx::query_scalar::<_, i64>(&format!(
            "UPDATE {} SET is_revoked = $1 WHERE id = $2 RETURNING id",
            Self::TABLE
        ))
        .bind(true)
        .bind(id)
        .fetch_optional(executor)
        .await?;

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
