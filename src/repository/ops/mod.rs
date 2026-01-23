use crate::repository::{Error, RepositoryManager, Result};
use sqlx::postgres::Postgres;
use sqlx::{Executor, QueryBuilder};

use std::fmt::Debug;

pub mod create;
pub mod get;
pub mod update;
pub mod delete;

pub trait DatabaseTable {
    const TABLE: &'static str;
}
