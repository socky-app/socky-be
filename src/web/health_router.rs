use std::time::Duration;

use axum::{extract::State, routing::get, Router};
use thiserror::Error;
use tokio::time::timeout;

use crate::{
    app::AppState,
    repository::{RepositoryManager, RepositoryManagerError},
    web::Result,
};

// The check should pass only if the DB responds within this time
const HEALTH_READY_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug, Error)]
pub enum HealthError {
    #[error("Health checked timed out")]
    Timeout,
    
    #[error("Health check failed: {0}")]
    Repository(RepositoryManagerError),
}

/// Public health routes.
pub fn public() -> Router<AppState> {
    Router::new()
        .route("/live", get(live_handler))
        .route("/ready", get(ready_handler))
}

/// Check if service is alive.
async fn live_handler() -> Result<()> {
    tracing::debug!("{:<15} - live_handler", "HANDLER");
    Ok(())
}

/// Chec if service is live and dependencies are ready.
async fn ready_handler(State(rm): State<RepositoryManager>) -> Result<()> {
    tracing::debug!("{:<15} - ready_handler", "HANDLER");

    match timeout(HEALTH_READY_TIMEOUT, rm.test_connection()).await {
        // The timeout elapsed before the query finished
        Err(_) => Err(HealthError::Timeout.into()),

        // The query finished, but returned a database error
        Ok(Err(e)) => Err(HealthError::Repository(e).into()),

        // 3. The query finished successfully within the timeout
        Ok(Ok(_)) => Ok(()),
    }
}
