use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use thiserror::Error;

use crate::repository::db;

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Error, Debug, Serialize)]
pub enum Error {
    NotFound { entity: &'static str, id: i64 },
    DatabaseConnectionFailed(#[from] db::Error),
    DatabaseQueryFailed(
        #[serde_as(as = "DisplayFromStr")]
        #[from]
        sqlx::Error,
    ),
}

impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
    }
}