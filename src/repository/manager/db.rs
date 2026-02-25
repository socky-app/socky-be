use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{debug, info};
use std::time::Duration;

use crate::config::DbConfig;
use super::RepositoryManagerError;

/// Creates a new database connection pool based on the provided configuration.
///
/// # Errors
///
/// Returns a `Error` if connecting to the database fails.
#[tracing::instrument(name = "create_db_pool", skip_all)]
pub async fn create_pool(config: &DbConfig) -> Result<PgPool, RepositoryManagerError> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_conn)
        .min_connections(config.min_conn)
        .acquire_timeout(Duration::from_secs(config.conn_timeout_seconds))
        .idle_timeout(Duration::from_secs(config.idle_timeout_seconds))
        .connect(&config.url())
        .await
        .map_err(RepositoryManagerError::CreatePoolFailed)?;
    
    info!("Database connection pool created successfully");
    Ok(pool)
}

/// Tests the database connection by executing a simple query.
///
/// # Errors
///
/// Returns a `Error` if the query fails, indicating a problem with the connection.
#[tracing::instrument(name = "test_db_connection", skip(pool))]
pub async fn test_connection(pool: &PgPool) -> Result<(), RepositoryManagerError> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(RepositoryManagerError::ConnectionFailed)?;
    debug!("Database connection test successful");
    Ok(())
}
