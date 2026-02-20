use thiserror::Error;

use crate::{
    model::user::error::{UserRoleError, UserStatusError},
    repository::RepositoryError,
    service::auth_controller::AuthError,
};

#[derive(Debug, Error, strum_macros::AsRefStr)]
pub enum ServiceError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Auth error")]
    Auth(#[from] AuthError),

    // TODO: Rename to user status
    #[error("Invalid user error")]
    UserStatus(#[from] UserStatusError),

    #[error("Invalid user role")]
    UserRole(#[from] UserRoleError),

    #[error("Current user not in request parts")]
    CurrentUserExtractionError,

    #[error("Internal server error: {0}")]
    Internal(String),
}
