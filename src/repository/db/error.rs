use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Error, Debug, Serialize)]
pub enum Error {
    CreatePoolFailed(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
    ConnectionFailed(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}
