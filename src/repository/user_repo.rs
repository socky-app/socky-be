//! User database repository implementation.

use chrono::Utc;
use sqlx::QueryBuilder;

use crate::{
    model::{
        pagination::{calculate_offset, PaginatedResponse},
        user::{
            dto::{
                CreateGhostUserDto, CreateRegisteredUserDto, UpdateProfileDto, UpdateUserStatusDto,
                UserQueryDto,
            },
            RegisteredUser, UserEntity, UserRole, UserRow, UserStatus,
        },
    },
    repository::{
        ops::{
            create, delete, delete_strategy::SoftDeleteStrategy, get, soft_delete, update,
            DatabaseTable,
        },
        RepositoryError, RepositoryManager, Result,
    },
};

pub struct UserRepository;

/// Create methods
impl UserRepository {
    pub async fn create_registered(
        rm: &RepositoryManager,
        dto: &CreateRegisteredUserDto,
    ) -> Result<i64> {
        let row = sqlx::query!(
            r#"
            INSERT INTO users (email, username, full_name, password_hash, is_ghost, role, status)
            VALUES ($1, $2, $3, $4, false, $5, $6)
            RETURNING id
            "#,
            dto.email,
            dto.username,
            dto.full_name,
            dto.password_hash,
            dto.role as _,
            dto.status as _
        )
        .fetch_one(rm.pool())
        .await
        .map_err(RepositoryError::DatabaseQueryFailed)?;
        Ok(row.id)
    }

    pub async fn create_ghost(rm: &RepositoryManager, dto: &CreateGhostUserDto) -> Result<i64> {
        let row = sqlx::query!(
            r#"
            INSERT INTO users (full_name, is_ghost)
            VALUES ($1, true)
            RETURNING id
            "#,
            dto.full_name
        )
        .fetch_one(rm.pool())
        .await
        .map_err(RepositoryError::DatabaseQueryFailed)?;
        Ok(row.id)
    }
}

/// Read methods
impl UserRepository {
    pub async fn get(rm: &RepositoryManager, id: i64) -> Result<UserEntity> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT 
                id, email, username, full_name, password_hash, is_ghost, 
                role as "role: _", status as "status: _", 
                last_login_at, created_at, updated_at, deleted_at 
            FROM users WHERE id = $1
            "#,
            id
        )
        .fetch_optional(rm.pool())
        .await
        .map_err(RepositoryError::DatabaseQueryFailed)?;

        if let Some(r) = row {
            UserEntity::try_from(r).map_err(RepositoryError::ConsistencyViolation)
        } else {
            Err(RepositoryError::NotFound {
                entity: "users",
                id,
            })
        }
    }

    /// Gets a registered user by user ID. Only returns active, non-ghost users.
    pub async fn get_active(rm: &RepositoryManager, user_id: i64) -> Result<RegisteredUser> {
        let row = sqlx::query_as!(
            RegisteredUser,
            r#"
            SELECT 
                id, 
                email as "email!", 
                username as "username!", 
                full_name, 
                password_hash as "password_hash!", 
                role as "role!: _", 
                status as "status!: _", 
                last_login_at, 
                created_at, 
                updated_at, 
                deleted_at
            FROM users
            WHERE id = $1
              AND is_ghost = false
              AND status = 1 -- Active
            "#,
            user_id
        )
        .fetch_optional(rm.pool())
        .await
        .map_err(RepositoryError::DatabaseQueryFailed)?;

        row.ok_or(RepositoryError::NotFound {
            entity: "users",
            id: user_id,
        })
    }

    /// Searches for active users by username. Only returns active, non-ghost users.
    pub async fn search_active(
        rm: &RepositoryManager,
        query_term: &str,
    ) -> Result<Vec<RegisteredUser>> {
        let pattern = format!("%{}%", query_term);
        sqlx::query_as!(
            RegisteredUser,
            r#"
            SELECT 
                id, 
                email as "email!", 
                username as "username!", 
                full_name, 
                password_hash as "password_hash!", 
                role as "role!: _", 
                status as "status!: _", 
                last_login_at, 
                created_at, 
                updated_at, 
                deleted_at
            FROM users
            WHERE username ILIKE $1
              AND is_ghost = false
              AND status = 1 -- Active
            ORDER BY id ASC
            LIMIT 20
            "#,
            pattern
        )
        .fetch_all(rm.pool())
        .await
        .map_err(RepositoryError::DatabaseQueryFailed)
    }

    async fn count_all(rm: &RepositoryManager, query: &UserQueryDto) -> Result<i64> {
        let status = query.status.map(|s| s as i16);
        let role = query.role.map(|r| r as i16);

        let count_row = sqlx::query!(
            r#"
            SELECT COUNT(*) as "total!"
            FROM users
            WHERE ($1::boolean IS NULL OR is_ghost = $1)
              AND ($2::smallint IS NULL OR status = $2)
              AND ($3::smallint IS NULL OR role = $3)
            "#,
            query.is_ghost,
            status,
            role
        )
        .fetch_one(rm.pool())
        .await
        .map_err(RepositoryError::DatabaseQueryFailed)?;

        Ok(count_row.total)
    }

    pub async fn list_all(
        rm: &RepositoryManager,
        query: &UserQueryDto,
    ) -> Result<PaginatedResponse<UserEntity>> {
        let offset = calculate_offset(query.page, query.limit);
        let total = Self::count_all(rm, query).await?;
        if total == 0 {
            return Ok(PaginatedResponse::new(
                Vec::new(),
                0,
                query.page.max(1),
                query.limit,
            ));
        }

        let status = query.status.map(|s| s as i16);
        let role = query.role.map(|r| r as i16);

        let rows = sqlx::query_as!(
            UserRow,
            r#"
            SELECT 
                id, email, username, full_name, password_hash, is_ghost, 
                role as "role: _", status as "status: _", 
                last_login_at, created_at, updated_at, deleted_at 
            FROM users
            WHERE ($1::boolean IS NULL OR is_ghost = $1)
              AND ($2::smallint IS NULL OR status = $2)
              AND ($3::smallint IS NULL OR role = $3)
            ORDER BY id ASC
            LIMIT $4 OFFSET $5
            "#,
            query.is_ghost,
            status,
            role,
            query.limit,
            offset
        )
        .fetch_all(rm.pool())
        .await
        .map_err(RepositoryError::DatabaseQueryFailed)?;

        let entities = rows
            .into_iter()
            .map(|r| UserEntity::try_from(r).map_err(RepositoryError::ConsistencyViolation))
            .collect::<Result<Vec<_>>>()?;

        Ok(PaginatedResponse::new(
            entities,
            total,
            query.page.max(1),
            query.limit,
        ))
    }
}

/// Update methods
impl UserRepository {
    /// Updates the user's public profile fields.
    pub async fn update_profile(
        rm: &RepositoryManager,
        user_id: i64,
        dto: &UpdateProfileDto,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET full_name = $1, updated_at = now() WHERE id = $2 AND is_ghost = false",
            dto.full_name,
            user_id
        )
        .execute(rm.pool())
        .await
        .map_err(RepositoryError::DatabaseQueryFailed)?;
        Ok(())
    }

    /// Updates the user's username handle.
    pub async fn update_username(
        rm: &RepositoryManager,
        user_id: i64,
        username: &str,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET username = $1, updated_at = now() WHERE id = $2 AND is_ghost = false",
            username,
            user_id
        )
        .execute(rm.pool())
        .await
        .map_err(RepositoryError::DatabaseQueryFailed)?;
        Ok(())
    }

    pub async fn update_status(
        rm: &RepositoryManager,
        id: i64,
        dto: &UpdateUserStatusDto,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET status = $1, deleted_at = CASE WHEN $1 = 1::smallint THEN NULL ELSE deleted_at END, updated_at = now() WHERE id = $2 AND is_ghost = false",
            dto.status as _,
            id
        )
        .execute(rm.pool())
        .await
        .map_err(RepositoryError::DatabaseQueryFailed)?;
        Ok(())
    }
}

/// Existence checks
impl UserRepository {
    pub async fn email_exists(rm: &RepositoryManager, email: &str) -> Result<bool> {
        let count = sqlx::query!("SELECT count(*) FROM users WHERE email = $1", email)
            .fetch_one(rm.pool())
            .await
            .map_err(RepositoryError::DatabaseQueryFailed)?;
        Ok(count.count.unwrap_or(0) > 0)
    }

    /// Checks if a user record with the given `username` exists in the database.
    pub async fn username_exists(rm: &RepositoryManager, username: &str) -> Result<bool> {
        let count = sqlx::query!("SELECT count(*) FROM users WHERE username = $1", username)
            .fetch_one(rm.pool())
            .await
            .map_err(RepositoryError::DatabaseQueryFailed)?;
        Ok(count.count.unwrap_or(0) > 0)
    }
}

/// Delete and anonymize methods
impl UserRepository {
    pub async fn soft_delete(rm: &RepositoryManager, id: i64) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET status = 2, deleted_at = now(), updated_at = now() WHERE id = $1 AND is_ghost = false",
            id
        )
        .execute(rm.pool())
        .await
        .map_err(RepositoryError::DatabaseQueryFailed)?;
        Ok(())
    }
}
