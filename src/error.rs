use std::fmt;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Main crate Result alias
pub type Result<T> = core::result::Result<T, Error>;

/// Main crate error
#[derive(Debug)]
pub enum Error {
    // Config
    ConfigMissingEnv(&'static str),
    ConfigWrongFormat(&'static str),

    // Login
    LoginFail,

    // Model
    TransactionDeleteFailIdNotFound{ id: u64 },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> core::result::Result<(), fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

/// Convert the internal server error into the client error
/// Here it is important to never pass through the server error directly to
/// the client, to avoid leaking information
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "INTO_RES");

        (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
    }
}
