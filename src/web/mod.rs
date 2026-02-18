//! Web layer.

mod error;

pub mod middleware;
pub mod routes_login;
pub mod routes_static;
pub mod routes_transaction;

pub use error::{ErrorDetails, WebError};
pub(in crate::web) type Result<T> = core::result::Result<T, WebError>;