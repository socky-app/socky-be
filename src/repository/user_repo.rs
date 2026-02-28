use chrono::Utc;
use sqlx::QueryBuilder;

use crate::{
    model::user::{
        dto::{CreateUserDto, UpdateUserStatusDto},
        UserEntity, UserRole, UserStatus,
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

/// Implement check existence by columns.
impl UserRepository {
    /// Check if email exists
    pub async fn email_exists(rm: &RepositoryManager, email: &str) -> Result<bool> {
        Self::exists_by_column(rm, "email", email).await
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
