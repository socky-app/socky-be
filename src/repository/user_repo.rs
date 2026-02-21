use chrono::Utc;
use sqlx::QueryBuilder;

use crate::{
    model::user::{
        dto::{CreateUserDto, UpdateUserPasswordDto, UpdateUserStatusDto},
        LoginCredentialsEntity, UserEntity, UserRole, UserStatus,
    },
    repository::{
        ops::{
            create, delete, delete_strategy::SoftDeleteStrategy, get, soft_delete, update,
            DatabaseTable,
        },
        RepositoryManager, Result,
    },
};

/// User repository for database operations.
pub struct UserRepository;

impl DatabaseTable for UserRepository {
    const TABLE: &'static str = "users";
    type DeleteStrategy = SoftDeleteStrategy;
}

impl create::Create for UserRepository {
    type D<'a> = CreateUserDto;
}

impl get::Get for UserRepository {
    type T = UserEntity;
}

impl update::Update for UserRepository {
    type D = UpdateUserStatusDto;
}

impl delete::Delete for UserRepository {}

impl soft_delete::SoftDelete for UserRepository {}

/// Implement password update.
// TODO: Fix that, since this update must happen within a transaction while
// invalidating all the user's token. This could be moved to an AuthRepo,
// that can call UserRepo and TokenRepo, for example.
impl UserRepository {
    pub async fn update_user_password(
        rm: &RepositoryManager,
        id: i64,
        dto: &UpdateUserPasswordDto,
    ) -> Result<i64> {
        update::update::<Self, _, _>(id, dto, rm.pool()).await
    }
}

/// Implement check existence by columns.
impl UserRepository {
    /// Check if email exists
    pub async fn email_exists(rm: &RepositoryManager, email: &str) -> Result<bool> {
        Self::exists_by_column(rm, "email", email).await
    }
}

/// Implement login and auth operations.
impl UserRepository {
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
        .await
        .inspect_err(|e| {
            tracing::error!("Database error getting login credentials: {:?}", e);
        })?;

        Ok(user)
    }

    /// Update last login timestamp
    pub async fn update_last_login(rm: &RepositoryManager, id: i64) -> Result<()> {
        sqlx::query("UPDATE users SET last_login_at = $1 WHERE id = $2")
            .bind(Utc::now().naive_utc())
            .bind(id)
            .execute(rm.pool())
            .await
            .inspect_err(|e| {
                tracing::error!(
                    "Database error in update_last_login, user_id={}: {:?}",
                    id,
                    e
                );
            })?;

        Ok(())
    }
}

impl create::Insertable for CreateUserDto {
    fn push_insert<'r>(&'r self, query_builder: &mut QueryBuilder<'r, sqlx::Postgres>) {
        query_builder
            .push("(email, password_hash, role, status) VALUES (")
            .push_bind(&self.email)
            .push(", ")
            .push_bind(&self.password)
            .push(", ")
            .push_bind(self.role)
            .push(", ")
            .push_bind(self.status)
            .push(")");
    }
}

impl update::Updatable for UpdateUserStatusDto {
    fn push_update<'q>(&'q self, b: &mut QueryBuilder<'q, sqlx::Postgres>) {
        b.push("status = ").push_bind(self.status);
    }
}

impl update::Updatable for UpdateUserPasswordDto {
    fn push_update<'r>(&'r self, b: &mut QueryBuilder<'r, sqlx::Postgres>) {
        b.push("password_hash = ").push_bind(&self.password);
    }
}
