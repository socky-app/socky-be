//! Controller layer

pub mod auth_controller;
mod error;

// Re-export module error and result.
pub use error::ControllerError;
pub(in crate::controller) type Result<T> = core::result::Result<T, ControllerError>;
