use thiserror::Error;

use crate::{
    model::user::error::UserStatusError, repository::RepositoryError,
    controller::auth_controller::AuthError,
};

#[derive(Debug, Error)]
pub enum ControllerError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Auth error")]
    Auth(#[from] AuthError),

    #[error("Invalid status: {0}")]
    UserStatus(#[from] UserStatusError),

    #[error("Internal server error: {0}")]
    Internal(String),
}
