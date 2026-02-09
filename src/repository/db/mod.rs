use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

use crate::common::config;

pub mod error;

pub use error::{Error, Result};

/// Configuration for the database connection pool.
///
/// This struct holds all the settings required to establish a connection
/// pool with the PostgreSQL database.
#[derive(Debug)]
pub struct DatabaseConfig {
    /// The database connection URL.
    pub url: String,
    /// The maximum number of connections the pool is allowed to maintain.
    pub max_connections: u32,
    /// The minimum number of connections the pool should maintain.
    pub min_connections: u32,
    /// The timeout for a single connection attempt.
    pub connect_timeout: Duration,
    /// The timeout for an idle connection.
    pub idle_timeout: Duration,
}

impl Default for DatabaseConfig {
    /// Creates a default database configuration from environment variables.
    ///
    /// # Panics
    ///
    /// This function will panic if the `DATABASE_URL` environment variable is not set.
    /// A valid database URL is essential for the application to run.
    fn default() -> Self {
        Self {
            url: config().db_url.to_string(),
            max_connections: config().db_max_conn,
            min_connections: config().db_min_conn,
            connect_timeout: Duration::from_secs(config().db_conn_timeout_seconds),
            idle_timeout: Duration::from_secs(config().db_idle_timeout_seconds),
        }
    }
}

/// Creates a new database connection pool based on the provided configuration.
///
/// # Errors
///
/// Returns a `Error` if connecting to the database fails.
#[tracing::instrument(name = "create_db_pool", skip_all)]
pub async fn create_pool_from_config(config: &DatabaseConfig) -> Result<PgPool> {
    tracing::info!("Creating database connection pool...");
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.connect_timeout)
        .idle_timeout(config.idle_timeout)
        .connect(&config.url)
        .await
        .map_err(Error::CreatePoolFailed)?;

    tracing::info!("Database connection pool created successfully.");
    Ok(pool)
}

/// Creates a new database connection pool using the env/default configuration.
///
/// # Errors
///
/// Returns a `Error` if connecting to the database fails.
#[tracing::instrument(name = "create_default_db_pool")]
pub async fn create_pool() -> Result<PgPool> {
    let config = DatabaseConfig::default();
    create_pool_from_config(&config).await
}

/// Tests the database connection by executing a simple query.
///
/// # Errors
///
/// Returns a `Error` if the query fails, indicating a problem with the connection.
#[tracing::instrument(name = "test_db_connection", skip(pool))]
pub async fn test_connection(pool: &PgPool) -> std::result::Result<(), Error> {
    tracing::debug!("Executing database connection test query...");
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(Error::ConnectionFailed)?;
    tracing::info!("Database connection test successful.");
    Ok(())
}
