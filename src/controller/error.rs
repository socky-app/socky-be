use strum_macros::AsRefStr;
use thiserror::Error;

use crate::{
    common::ErrorType, controller::auth_controller::AuthError, impl_error_type,
    model::user::error::UserStatusError, repository::RepositoryError,
};

#[derive(Debug, Error, AsRefStr)]
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

impl_error_type!(ControllerError {
    delegate: [Repository, Auth, UserStatus],
    terminal: [Internal(_)]
});
