use std::time::Duration;

use axum::{extract::State, routing::get, Router};
use thiserror::Error;
use tokio::time::timeout;
use tracing::trace;

use crate::{
    app::AppState,
    repository::{RepositoryManager, RepositoryManagerError},
    web::{ClientError, Result},
};

// The check should pass only if the DB responds within this time
const HEALTH_READY_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug, Error)]
pub enum HealthError {
    #[error("health checked timed out")]
    Timeout,

    #[error("health check failed due to repository")]
    Repository(#[from] RepositoryManagerError),
}

/// Public health routes.
pub fn public() -> Router<AppState> {
    Router::new()
        .route("/live", get(live_handler))
        .route("/ready", get(ready_handler))
}

/// Check if service is live.
#[utoipa::path(
    get,
    path = "/health/live",
    tag = "Health",
    summary = "Check service live",
    responses(
        (status = 200, description = "Service is live")
    )
)]
async fn live_handler() -> Result<()> {
    trace!("Handler health live");
    Ok(())
}

/// Check if service is live and dependencies are ready.
#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "Health",
    summary = "Check service ready",
    responses(
        (status = 200, description = "Service is ready"),
        (status = 503, description = "Service unavailable", body = ClientError)
    )
)]
async fn ready_handler(State(rm): State<RepositoryManager>) -> Result<()> {
    trace!("Handler health ready");

    match timeout(HEALTH_READY_TIMEOUT, rm.test_connection()).await {
        // The timeout elapsed before the query finished
        Err(_) => Err(HealthError::Timeout.into()),

        // The query finished, but returned a database error
        Ok(Err(e)) => Err(HealthError::Repository(e).into()),

        // 3. The query finished successfully within the timeout
        Ok(Ok(_)) => Ok(()),
    }
}
