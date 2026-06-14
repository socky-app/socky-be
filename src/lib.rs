//! # Socky Backend
//!
//! The backend engine for the Socky financial tracking and expense-splitting application.
//!
//! ## Architecture Overview
//! The application is structured in layered modules to enforce clean separation of concerns:
//!
//! * **`config`**: Application configuration management utilizing Figment for environment-based parsing.
//! * **`model`**: Internal database entities, Data Transfer Objects (DTOs), and Value Objects (VOs).
//! * **`repository`**: Database persistence layer using SQLx.
//! * **`utils`**: Core helper modules for cryptography (HMAC, passwords, JWT tokens).
//! * **`worker`**: Background tasks (e.g. cleaning up expired refresh tokens).
//! * **`app`**: AppState initialization and route-middleware stitching.
//! * **`context`**: Context extraction structs like `CurrentUser` extracted during request lifecycles.
//! * **`controller`**: Domain controllers managing core business logics.
//! * **`web`**: Routing endpoints, HTTP error mappings, and OpenAPI setup.

#![allow(unused)] // TODO: Remove this later

pub mod config;
pub mod model;
pub mod repository;
pub mod utils;
pub mod worker;

mod app;
mod context;
mod controller;
mod web;

// Re-export create_app function
pub use app::create_app;
