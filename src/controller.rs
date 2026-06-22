//! Controller layer.
//!
//! Orchestrates the business logic of the application by coordinating database
//! models, repositories, and helper utilities.

pub mod auth_controller;
mod error;
pub mod profile_controller;
pub mod user_controller;

// Re-export module error and result.
pub use error::ControllerError;
pub(in crate::controller) type Result<T> = core::result::Result<T, ControllerError>;
