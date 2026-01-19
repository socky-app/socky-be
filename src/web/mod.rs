//! Web utilities and route registrations for the application.

pub mod mw_auth;
pub mod mw_res_map;
pub mod routes_login;
pub mod routes_static;
pub mod routes_transaction;

pub const AUTH_TOKEN: &str = "auth-token";
