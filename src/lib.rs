//! Backend of the Socky application

#![allow(unused)] // TODO: Remove this later

pub mod config;

mod app;
mod context;
mod model;
mod repository;
mod service;
mod utils;
mod web;

// Re-export create_app function
pub use app::create_app;

// Re-export startup functions
pub mod startup {
    pub use crate::repository::{RepositoryManager, RepositoryManagerError};
}
