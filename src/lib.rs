//! Backend of the Socky application

#![allow(unused)]

pub mod app;
pub mod common;

mod ctx;
mod error;
mod log;
mod model;
pub mod repository;
mod service;
mod web;
mod utils;

pub use self::error::{Error, Result};
