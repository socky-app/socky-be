use std::sync::Arc;

use axum::extract::FromRef;

use crate::{
    common::config::{AppConfig, AuthConfig, Config, DbConfig},
    repository::RepositoryManager,
};

#[derive(Clone)]
pub struct AppState {
    pub app_config: Arc<AppConfig>,
    pub db_config: Arc<DbConfig>,
    pub auth_config: Arc<AuthConfig>,
    pub rm: RepositoryManager,
}

impl AppState {
    pub fn new(config: Config, rm: RepositoryManager) -> Self {
        let Config { app, db, auth } = config;
        Self {
            app_config: Arc::new(app),
            db_config: Arc::new(db),
            auth_config: Arc::new(auth),
            rm,
        }
    }
}

impl FromRef<AppState> for Arc<AppConfig> {
    fn from_ref(state: &AppState) -> Self {
        state.app_config.clone()
    }
}

impl FromRef<AppState> for Arc<DbConfig> {
    fn from_ref(state: &AppState) -> Self {
        state.db_config.clone()
    }
}

impl FromRef<AppState> for Arc<AuthConfig> {
    fn from_ref(state: &AppState) -> Self {
        state.auth_config.clone()
    }
}

impl FromRef<AppState> for RepositoryManager {
    fn from_ref(state: &AppState) -> Self {
        state.rm.clone()
    }
}
