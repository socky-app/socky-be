//! Web layer.

mod error;

pub mod auth_router;
pub mod docs_router;
pub mod extractor;
pub mod fallback_router;
pub mod health_router;
pub mod middleware;
pub mod openapi;
pub mod static_router;
pub mod transaction_router;

pub use error::WebError;
pub(in crate::web) type Result<T> = core::result::Result<T, WebError>;

use error::ClientError;
