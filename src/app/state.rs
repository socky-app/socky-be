use std::sync::Arc;

use axum::extract::FromRef;

use crate::{config::AppConfig, repository::RepositoryManager};

#[derive(Clone)]
pub struct AppState {
    pub rm: RepositoryManager,
    pub config: Arc<AppConfig>,
}

impl AppState {
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
