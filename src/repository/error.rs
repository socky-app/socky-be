use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use thiserror::Error;

use crate::repository::db;

#[serde_as]
#[derive(Error, Debug, Serialize)]
pub enum RepositoryError {
    NotFound {
        entity: &'static str,
        id: i64,
    },
    DatabaseConnectionFailed(#[from] db::Error),
    DatabaseQueryFailed(
        #[serde_as(as = "DisplayFromStr")]
        #[from]
        sqlx::Error,
    ),
}

impl core::fmt::Display for RepositoryError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}
