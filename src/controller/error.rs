use thiserror::Error;

use crate::{
    controller::auth_controller::AuthError, model::user::error::UserStatusError,
    repository::RepositoryError,
};

#[derive(Debug, Error)]
pub enum ControllerError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),

    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error("invalid user status")]
    UserStatus(#[from] UserStatusError),

    #[error("{0}")]
    Internal(String),
}
