//! Web layer.

mod error;

pub mod middleware;
pub mod auth_router;
pub mod routes_transaction;

pub use error::{ErrorDetails, WebError};
pub(in crate::web) type Result<T> = core::result::Result<T, WebError>;