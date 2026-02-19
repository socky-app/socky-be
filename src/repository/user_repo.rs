use chrono::Utc;
use sqlx::QueryBuilder;

use crate::{
    model::user::{
        dto::{CreateUserDto, UpdateUserDto, UpdateUserPasswordDto},
        LoginCredentialsEntity, UserEntity,
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
    type D = UpdateUserDto;
}

impl delete::Delete for UserRepository {}

impl soft_delete::SoftDelete for UserRepository {}

/// Implement password update.
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

    /// Check if username exists
    pub async fn username_exists(rm: &RepositoryManager, username: &str) -> Result<bool> {
        Self::exists_by_column(rm, "username", username).await
    }
}

/// Implement login and auth operations.
impl UserRepository {
    /// Get user by username for authentication (only essential fields)
    pub async fn get_login_credentials(
        rm: &RepositoryManager,
        username: &str,
    ) -> Result<Option<LoginCredentialsEntity>> {
        Ok(
            sqlx::query_as::<_, LoginCredentialsEntity>("SELECT * FROM get_login_credentials($1)")
                .bind(username)
                .fetch_optional(rm.pool())
                .await
                .inspect_err(|e| {
                    tracing::error!("Database error getting login credentials: {:?}", e);
                })?,
        )
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
            .push("(username, email, password_hash, status) VALUES (")
            .push_bind(&self.username)
            .push(", ")
            .push_bind(&self.email)
            .push(", ")
            .push_bind(&self.password)
            .push(", ")
            .push_bind(self.status)
            .push(")");
    }
}

impl update::Updatable for UpdateUserDto {
    fn push_update<'q>(&'q self, b: &mut QueryBuilder<'q, sqlx::Postgres>) {
        b.push("email = ").push_bind(&self.email);
    }
}

impl update::Updatable for UpdateUserPasswordDto {
    fn push_update<'r>(&'r self, b: &mut QueryBuilder<'r, sqlx::Postgres>) {
        b.push("password_hash = ").push_bind(&self.password);
    }
}
