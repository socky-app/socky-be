use std::time::Duration;

use axum::{Router, extract::State, routing::get};
use tokio::time::timeout;

use crate::{app::AppState, repository::RepositoryManager, web::Result};

// The check should pass only if the DB responds within this time
const HEALTH_READY_TIMEOUT: Duration = Duration::from_secs(1);

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
        Err(_) => {
            todo!()
            // (StatusCode::SERVICE_UNAVAILABLE, "Database timeout"),
        },

        // The query finished, but returned a database error
        Ok(Err(e)) => {
            todo!()
            // (StatusCode::SERVICE_UNAVAILABLE, "Database error")
        },

        // 3. The query finished successfully within the timeout
        Ok(Ok(_)) => Ok(()),
    }
}
