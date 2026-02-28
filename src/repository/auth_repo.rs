use chrono::Utc;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

use crate::{
    model::{
        auth::{
            dto::CreateRefreshTokenDto, LoginCredentialsEntity, RefreshTokenEntity,
            RevokedTokenEntity,
        },
        user::{UserRole, UserStatus},
    },
    repository::{
        helper::{commit_db_transaction, start_db_transaction},
        ops::{
            create::{create, Create, Insertable},
            delete::{delete, Delete},
            delete_strategy::HardDeleteStrategy,
            get::{get, Get},
            DatabaseTable,
        },
        RepositoryError, RepositoryManager, Result,
    },
};

/// Auth repository for database operations.
pub struct AuthRepository;

/// Token repository for database operations.
struct TokenTable;

impl DatabaseTable for TokenTable {
    const TABLE: &'static str = "refresh_tokens";
    // Note: The refresh_tokens table must not implement SoftDeleteStrategy,
    // otherwise the logic in get_by_hash and revoke_family would have to change.
    type DeleteStrategy = HardDeleteStrategy;
}

/// User related implementation
impl AuthRepository {
    /// Get user by email for authentication (only essential fields)
    pub async fn get_login_credentials(
        rm: &RepositoryManager,
        email: &str,
    ) -> Result<Option<LoginCredentialsEntity>> {
        let user = sqlx::query_as!(
            LoginCredentialsEntity,
            r#"
            SELECT 
                id, 
                password_hash, 
                role AS "role: UserRole", 
                status AS "status: UserStatus"
            FROM users 
            WHERE email = $1 
            AND deleted_at IS NULL
            "#,
            email
        )
        .fetch_optional(rm.pool())
        .await?;

        Ok(user)
    }

    /// Update last login timestamp
    pub async fn update_last_login(rm: &RepositoryManager, id: i64) -> Result<()> {
        sqlx::query!(
            r#"UPDATE users SET last_login_at = $1 WHERE id = $2"#,
            Utc::now().naive_utc(),
            id
        )
        .execute(rm.pool())
        .await?;

        Ok(())
    }
}

/// Token related implementation
impl AuthRepository {
    pub async fn create_token<'a>(
        rm: &RepositoryManager,
        dto: &CreateRefreshTokenDto<'a>,
    ) -> Result<i64> {
        create::<TokenTable, _, _>(dto, rm.pool()).await
    }

    pub async fn get_token(rm: &RepositoryManager, id: i64) -> Result<RefreshTokenEntity> {
        get::<TokenTable, _, _>(id, rm.pool()).await
    }

    pub async fn delete_token(rm: &RepositoryManager, id: i64) -> Result<()> {
        delete::<TokenTable, _>(id, rm.pool()).await
    }

    pub async fn get_token_by_hash(
        rm: &RepositoryManager,
        hash: &[u8],
    ) -> Result<Option<RefreshTokenEntity>> {
        let result = sqlx::query_as!(
            RefreshTokenEntity,
            r#"
            SELECT * FROM refresh_tokens 
            WHERE token_hash = $1
            "#,
            hash
        )
        .fetch_optional(rm.pool())
        .await?;

        Ok(result)
    }

    pub async fn revoke_token_family(rm: &RepositoryManager, family_id: &Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE refresh_tokens 
            SET is_revoked = true 
            WHERE family_id = $1 AND is_revoked = false
            "#,
            family_id
        )
        .execute(rm.pool())
        .await?;

        Ok(())
    }

    pub async fn revoke_token_family_by_hash(
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

    pub async fn rotate_token(
        rm: &RepositoryManager,
        dto: &CreateRefreshTokenDto<'_>,
        old_token_id: i64,
    ) -> Result<i64> {
        let mut tx = start_db_transaction(rm).await?;

        Self::revoke_token(old_token_id, &mut *tx).await?;
        let new_token_id = create::<TokenTable, _, _>(dto, &mut *tx).await?;

        commit_db_transaction(tx).await?;

        Ok(new_token_id)
    }

    async fn revoke_token<'c, E>(id: i64, executor: E) -> Result<i64>
    where
        E: Executor<'c, Database = Postgres> + Send,
    {
        let ret_option = sqlx::query_scalar!(
            r#"
            UPDATE refresh_tokens
            SET is_revoked = true
            WHERE id = $1 AND is_revoked = false
            RETURNING id
            "#,
            id
        )
        .fetch_optional(executor)
        .await?;

        if let Some(ret_id) = ret_option {
            Ok(ret_id)
        } else {
            Err(RepositoryError::NotFound {
                entity: "refresh_tokens",
                id,
            })
        }
    }
}

impl Insertable for CreateRefreshTokenDto<'_> {
    fn push_insert<'r>(&'r self, query_builder: &mut sqlx::QueryBuilder<'r, sqlx::Postgres>) {
        query_builder
            .push("(token_hash, user_id, family_id, access_id, expires_at) VALUES (")
            .push_bind(self.token_hash)
            .push(", ")
            .push_bind(self.user_id)
            .push(", ")
            .push_bind(self.family_id)
            .push(", ")
            .push_bind(self.access_id)
            .push(", ")
            .push_bind(self.expires_at)
            .push(")");
    }
}
