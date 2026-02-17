//! Backend of the Socky application

#![allow(unused)] // TODO: Remove this later

pub mod app;
pub mod config;

mod ctx;
mod error;
mod log;
mod model;
mod repository;
mod controller;
mod web;
mod utils;

// Re-export repository manager
pub use repository::{RepositoryManager, RepositoryManagerError};

// TODO: remove main crate error
use error::{Error, Result};
