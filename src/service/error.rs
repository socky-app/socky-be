use thiserror::Error;

use crate::{
    model::user::error::UserError, repository::RepositoryError, service::auth_controller::AuthError,
};

#[derive(Debug, Error, strum_macros::AsRefStr)]
pub enum ServiceError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Auth error")]
    Auth(#[from] AuthError),

    #[error("Invalid user error")]
    User(#[from] UserError),

    #[error("Current user not in request parts")]
    CurrentUserExtractionError,

    #[error("Internal server error: {0}")]
    Internal(String),
}
