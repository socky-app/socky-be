//! Backend of the Socky application

#![allow(unused)] // TODO: Remove this later

pub mod config;
pub mod model;
pub mod repository;
pub mod utils;

mod app;
mod common;
mod context;
mod controller;
mod web;

// Re-export create_app function
pub use app::create_app;
