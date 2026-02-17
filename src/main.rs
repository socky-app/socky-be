//! Entrypoint for the the Socky backend.

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{info, error};
use tracing_subscriber::EnvFilter;
use thiserror::Error;

use socky_be::{
    app::{AppConfig, create_app}, config::{load_config, ConfigError}, RepositoryManager, RepositoryManagerError,
};

#[derive(Debug, Error)]
enum RunError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    
    #[error(transparent)]
    Repository(#[from] RepositoryManagerError),

    #[error(transparent)]
    Network(#[from] std::io::Error),
}

async fn run() -> Result<(), RunError> {
    // Load config
    let config = load_config()?;

    // Initialize RepositoryManager
    let rm = RepositoryManager::new(&config.db).await?;

    // Create app
    let app_config = AppConfig { router: config.router, auth: config.auth };
    let app = create_app(rm, app_config);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080)); // TODO: fix
    let listener = TcpListener::bind(addr).await?;
    info!("{:<12} - {addr}\n", "LISTENING");
    axum::serve(listener, app).await?;

    Ok(())
}

/// Entrypoint for the backend service
#[tokio::main]
async fn main() {
    // Load env
    dotenvy::dotenv().ok();

    // Initialize logger
    tracing_subscriber::fmt()
        .without_time() // TODO: For early local development.
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    if let Err(e) = run().await {
        error!("Application finished with error: {}", e);
        std::process::exit(1);
    }
}