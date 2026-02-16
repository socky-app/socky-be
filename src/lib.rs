//! Backend of the Socky application

pub mod app;
pub mod common;

pub mod ctx;
pub mod error;
pub mod log;
pub mod model;
pub mod repository;
pub mod service;
pub mod web;
mod utils;

pub use self::error::{Error, Result};
