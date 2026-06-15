//! Database repository for authentication, user sessions, and token rotation security.
//!
//! Provides SQL queries for refresh token tracking, family revocation, credentials loading,
//! and last login timestamps.

use chrono::Utc;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    model::{
        auth::{
            dto::CreateRefreshTokenDto, RefreshTokenEntity, RevokedTokenEntity,
            TokenRotationEntity, UserCredentialsEntity,
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

/// Token table definition.
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
                password_hash AS "password_hash!",
                role AS "role!: UserRole",
                status AS "status!: UserStatus"
            FROM users
            WHERE id = $1
            AND is_ghost = false
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
                password_hash as "password_hash!",
                role AS "role!: UserRole",
                status AS "status!: UserStatus"
            FROM users
            WHERE email = $1
            AND is_ghost = false
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
    ///
    /// NOTE: This function only works because of the assumption that there should be strictly one
    /// non-revoked token per family_id (last one that was issued during a rotation). If an
    /// application-level bug breaks this invariant, this request could deadlock. To solve this,
    /// we could read the target token without a lock first, then acquire locks on all non-revoked
    /// tokens in the family, and then check if the target token is revoked or not. This would add
    /// some complexity, but it would be more robust.
    pub async fn revoke_token_family_by_hash(
        rm: &RepositoryManager,
        token_hash: &[u8],
    ) -> Result<Option<RevokedTokenEntity>> {
        let mut tx = start_db_transaction(rm).await?;

        // Lock the specific token first
        // If there is a concurrent refresh, this forces the logout to wait here.
        let target_token = sqlx::query_as!(
            RevokedTokenEntity,
            r#"
            SELECT
                id,
                user_id,
                family_id,
                is_revoked AS was_already_revoked
            FROM refresh_tokens
            WHERE token_hash = $1
            FOR NO KEY UPDATE
            "#,
            token_hash
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(token) = target_token {
            // Revoke the family in a separate statement
            // Because this executes AFTER the lock is acquired, it gets a fresh
            // snapshot and will successfully catch any newly minted concurrent tokens.
            Self::trans_revoke_tokens_for_family(&token.family_id, &mut tx).await?;

            commit_db_transaction(tx).await?;

            Ok(Some(token))
        } else {
            // Token literally does not exist
            Ok(None)
        }
    }

    /// Revokes all tokens for a given user. If the query suceeds, returns `Ok`,
    /// otherwise returns the corresponding `RepositoryError`.
    pub async fn revoke_tokens_for_user(rm: &RepositoryManager, user_id: i64) -> Result<()> {
        let mut tx = start_db_transaction(rm).await?;
        Self::trans_revoke_tokens_for_user(user_id, &mut tx).await?;
        commit_db_transaction(tx).await?;
        Ok(())
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

        Self::trans_revoke_tokens_for_user(user_id, &mut tx).await?;

        commit_db_transaction(tx).await?;

        Ok(())
    }
}

/// A guard that holds an exclusive database lock on a token family.
/// If this struct is dropped before calling `rotate` or `revoke`,
/// the database transaction is automatically rolled back.
pub struct TokenRotationLock<'a> {
    pub entity: TokenRotationEntity,
    tx: Transaction<'a, sqlx::Postgres>,
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
                u.email AS "user_email!",
                u.role AS "user_role!: UserRole",
                u.status AS "user_status!: UserStatus"
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
        AuthRepository::trans_revoke_tokens_for_family(&self.entity.family_id, &mut self.tx)
            .await?;

        commit_db_transaction(self.tx).await?;

        Ok(())
    }
}

/// Helper methods for operations that require DB transactions.
impl AuthRepository {
    /// Revokes the entire token family. It only updates tokens that are currently not
    /// revoked, to avoid any deadlocks.
    async fn trans_revoke_tokens_for_family<'c>(
        family_id: &Uuid,
        tx: &mut Transaction<'c, sqlx::Postgres>,
    ) -> Result<()> {
        // Lock the currently active token(s) in the family.
        // If an attacker is actively refreshing the valid token right now,
        // this forces our penalty thread to wait until they finish.
        let _ = sqlx::query!(
            r#"
            SELECT id FROM refresh_tokens
            WHERE family_id = $1 AND is_revoked = false
            ORDER BY id
            FOR NO KEY UPDATE
            "#,
            family_id
        )
        .fetch_all(&mut **tx)
        .await?;

        // Because this executes AFTER acquiring the lock, Postgres generates a
        // fresh snapshot.
        sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET is_revoked = true
            WHERE family_id = $1 AND is_revoked = false
            "#,
            family_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Revokes all the tokens for a user. It only updates tokens that are currently not
    /// revoked, to avoid any deadlocks.
    async fn trans_revoke_tokens_for_user<'c>(
        user_id: i64,
        tx: &mut Transaction<'c, sqlx::Postgres>,
    ) -> Result<()> {
        // Lock all active tokens for this user sequentially.
        // If an attacker is currently refreshing any token, this forces the
        // revoke to wait here until they finish.
        let _ = sqlx::query!(
            r#"
            SELECT id FROM refresh_tokens
            WHERE user_id = $1 AND is_revoked = false
            ORDER BY id
            FOR NO KEY UPDATE
            "#,
            user_id
        )
        .fetch_all(&mut **tx)
        .await?;

        // The Actual Revocation
        // Because this is a NEW statement executed AFTER acquiring the locks,
        // Postgres generates a fresh snapshot. If an attacker managed to insert
        // a new token while we were waiting at Step 1, this query will see it and destroy it.
        sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET is_revoked = true
            WHERE user_id = $1 AND is_revoked = false
            "#,
            user_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

/// Token clean-up implementation
impl AuthRepository {
    /// Deletes expired tokens in batches of 1000.
    /// Returns the number of rows deleted in this specific batch.
    pub async fn delete_expired_token_batch(rm: &RepositoryManager, limit: u64) -> Result<u64> {
        // We use a subquery to select a chunk, then delete that chunk.
        // This prevents massive lock escalation.
        let rows_affected = sqlx::query!(
            r#"
            DELETE FROM refresh_tokens
            WHERE id IN (
                SELECT id FROM refresh_tokens
                WHERE expires_at < $1
                LIMIT $2
            )
            "#,
            chrono::Utc::now().naive_utc(),
            limit as i64
        )
        .execute(rm.pool())
        .await?
        .rows_affected();

        Ok(rows_affected)
    }
}

/// Delete and anonymize methods
impl AuthRepository {
    /// Anonymizes a user account (converts to ghost) and permanently deletes all their sessions.
    pub async fn anonymize_user(rm: &RepositoryManager, user_id: i64) -> Result<()> {
        let mut tx = start_db_transaction(rm).await?;

        // 1. Delete all refresh tokens
        sqlx::query!("DELETE FROM refresh_tokens WHERE user_id = $1", user_id)
            .execute(&mut *tx)
            .await
            .map_err(RepositoryError::DatabaseQueryFailed)?;

        // 2. Anonymize user record
        sqlx::query!(
            r#"
            UPDATE users
            SET email = NULL,
                password_hash = NULL,
                username = NULL,
                is_ghost = true,
                role = NULL,
                status = NULL,
                updated_at = now()
            WHERE id = $1
            "#,
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(RepositoryError::DatabaseQueryFailed)?;

        commit_db_transaction(tx).await?;
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
