//! Controller layer

mod error;
pub mod auth_controller;
pub mod log_controller;

// Re-export module error and result.
pub use error::ServiceError;
pub(in crate::service) type Result<T> = core::result::Result<T, ServiceError>;