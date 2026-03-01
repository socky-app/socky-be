use chrono::Utc;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

use crate::{
    model::{
        auth::{
            RefreshTokenEntity, RevokedTokenEntity, TokenRotationEntity, UserCredentialsEntity, dto::CreateRefreshTokenDto
        },
        user::{UserRole, UserStatus},
    },
    repository::{
        RepositoryError, RepositoryManager, Result, helper::{commit_db_transaction, start_db_transaction}, ops::{
            DatabaseTable, create::{Create, Insertable, create}, delete::{Delete, delete}, delete_strategy::HardDeleteStrategy, get::{Get, get}
        }
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

/// Token CRUD implementation
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
}

/// Credentials and login implementation
impl AuthRepository {
    /// Gets user credentials for authentication (only essential fields)
    pub async fn get_user_credentials(
        rm: &RepositoryManager,
        id: i64,
    ) -> Result<UserCredentialsEntity> {
        let user_option = sqlx::query_as!(
            UserCredentialsEntity,
            r#"
            SELECT
                id,
                password_hash,
                role AS "role: UserRole",
                status AS "status: UserStatus"
            FROM users
            WHERE id = $1
            AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(rm.pool())
        .await?;

        if let Some(entity) = user_option {
            Ok(entity)
        } else {
            Err(RepositoryError::NotFound {
                entity: "users",
                id,
            })
        }
    }

    /// Gets user credentials for authentication by email (only essential fields)
    pub async fn get_user_credentials_by_email(
        rm: &RepositoryManager,
        email: &str,
    ) -> Result<Option<UserCredentialsEntity>> {
        let user = sqlx::query_as!(
            UserCredentialsEntity,
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

    /// Updates last login timestamp
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

/// Logout implementation
impl AuthRepository {
    /// Revokes the family of the token that has `token_hash`. Returns `Ok(Some)`` containing
    /// the token entity for that 'token_hash', so that the service can differentiate the use
    /// of previously revoked tokens for logout operations. If `token_hash` is not present,
    /// returns `Ok(None)`, and if the query fails, returns the corresponding `RepositoryError`.
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

    /// Revokes all tokens for a given user. If the query suceeds, returns `Ok`,
    /// otherwise returns the corresponding `RepositoryError`.
    pub async fn revoke_tokens_for_user(rm: &RepositoryManager, user_id: i64) -> Result<()> {
        Self::trans_revoke_tokens_for_user(user_id, rm.pool()).await
    }
}

/// Password changing implementation
impl AuthRepository {
    /// Change user password and revoke all refresh tokens.
    pub async fn change_password_and_revoke_tokens(
        rm: &RepositoryManager,
        user_id: i64,
        new_password_hash: &str,
    ) -> Result<()> {
        let mut tx = start_db_transaction(rm).await?;

        sqlx::query!(
            r#"UPDATE users SET password_hash = $1 WHERE id = $2"#,
            new_password_hash,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        Self::trans_revoke_tokens_for_user(user_id, &mut *tx).await?;

        commit_db_transaction(tx).await?;

        Ok(())
    }
}

/// A guard that holds an exclusive database lock on a token family.
/// If this struct is dropped before calling `rotate` or `revoke`,
/// the database transaction is automatically rolled back.
pub struct TokenRotationLock<'a> {
    pub entity: TokenRotationEntity,
    tx: sqlx::Transaction<'a, sqlx::Postgres>,
}

/// Token rotation implementation
impl AuthRepository {
    /// Acquires a row-level lock on the token to prevent concurrent rotations.
    pub async fn acquire_rotation_lock<'a>(
        rm: &'a RepositoryManager,
        token_hash: &[u8],
    ) -> Result<Option<TokenRotationLock<'a>>> {
        let mut tx = start_db_transaction(rm).await?;

        let entity_option = sqlx::query_as!(
            TokenRotationEntity,
            r#"
            SELECT
                t.*,
                u.email AS user_email,
                u.role AS "user_role: UserRole",
                u.status AS "user_status: UserStatus"
            FROM refresh_tokens t
            JOIN users u ON t.user_id = u.id
            WHERE t.token_hash = $1
            FOR NO KEY UPDATE OF t
            "#,
            token_hash
        )
        .fetch_optional(&mut *tx)
        .await?;

        Ok(entity_option.map(|entity| TokenRotationLock { entity, tx }))
    }
}

impl<'a> TokenRotationLock<'a> {
    /// Consumes the lock, revokes the old token, inserts the new one, and commits.
    pub async fn rotate(mut self, dto: &CreateRefreshTokenDto<'_>) -> Result<i64> {
        // 1. Revoke the specific old token
        sqlx::query!(
            "UPDATE refresh_tokens SET is_revoked = true WHERE id = $1",
            self.entity.id
        )
        .execute(&mut *self.tx)
        .await?;

        // 2. Insert the new token
        let new_id = create::<TokenTable, _, _>(dto, &mut *self.tx).await?;

        // 3. Commit the transaction, releasing the lock
        commit_db_transaction(self.tx).await?;

        Ok(new_id)
    }

    /// Consumes the lock, revokes all the live tokens of the family, and commits.
    pub async fn revoke_family(mut self) -> Result<()> {
        sqlx::query!(
            "UPDATE refresh_tokens SET is_revoked = true WHERE family_id = $1 AND is_revoked = false",
            self.entity.family_id
        )
        .execute(&mut *self.tx)
        .await?;

        commit_db_transaction(self.tx).await?;

        Ok(())
    }
}

/// Helper methods for operations that require DB transactions.
impl AuthRepository {
    async fn trans_revoke_tokens_for_user<'c, E>(user_id: i64, executor: E) -> Result<()>
    where
        E: Executor<'c, Database = Postgres> + Send,
    {
        sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET is_revoked = true
            WHERE user_id = $1 AND is_revoked = false
            "#,
            user_id
        )
        .execute(executor)
        .await?;

        Ok(())
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
