//! Web layer.

mod error;

pub mod middleware;
pub mod extractor;
pub mod auth_router;
pub mod health_router;

pub mod transaction_router;

pub use error::{ErrorDetails, WebError};
pub(in crate::web) type Result<T> = core::result::Result<T, WebError>;