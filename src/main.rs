//! Entrypoint for the the Socky backend.

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

use socky_be::{
    Result, app::{AppConfig, create_app}, config::load_config, repository::RepositoryManager,
};

/// Entrypoint for the backend service
#[tokio::main]
async fn main() -> Result<()> {
    // Load env
    dotenvy::dotenv().ok();

    // Initialize logger
    tracing_subscriber::fmt()
        .without_time() // TODO: For early local development.
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Load config
    let config = load_config();

    // Initialize RepositoryManager
    let rm = RepositoryManager::new(&config.db).await.unwrap(); // TODO: fix

    // Create app
    let app_config = AppConfig { router: config.router, auth: config.auth };
    let app = create_app(rm, app_config);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080)); // TODO: fix
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("{:<12} - {addr}\n", "LISTENING");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
