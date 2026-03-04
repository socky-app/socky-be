//! Repository layer

mod error;
mod helper;
mod manager;
mod ops;

pub mod auth_repo;
pub mod transaction_repo;
pub mod user_repo;

// Re-export the public traits.
pub use ops::{create::Create, delete::Delete, get::Get, soft_delete::SoftDelete, update::Update};

// Re-export module error and result.
pub use error::RepositoryError;
pub(in crate::repository) type Result<T> = core::result::Result<T, RepositoryError>;

// Re-export manager
pub use manager::{RepositoryManager, RepositoryManagerError};
