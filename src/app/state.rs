//! Application state management.

use std::sync::Arc;

use axum::extract::FromRef;

use crate::{config::AppConfig, repository::RepositoryManager};

/// Shared state accessible by HTTP router handlers and middleware.
///
/// Contains cloning/sharing references to the database pool coordinator
/// and the static application configuration.
#[derive(Clone)]
pub struct AppState {
    /// The database repository manager containing the connection pool.
    pub rm: RepositoryManager,
    /// The parsed application configuration wrapped in an `Arc`.
    pub config: Arc<AppConfig>,
}

impl AppState {
    /// Creates a new `AppState` instance.
    pub fn new(rm: RepositoryManager, config: AppConfig) -> Self {
        Self {
            rm,
            config: Arc::new(config),
        }
    }
}

impl FromRef<AppState> for Arc<AppConfig> {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AppState> for RepositoryManager {
    fn from_ref(state: &AppState) -> Self {
        state.rm.clone()
    }
}
