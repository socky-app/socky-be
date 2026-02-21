//! Entrypoint for the the Socky backend.

use anyhow::Result;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use socky_be::{
    config::load_config,
    create_app,
    repository::RepositoryManager,
};

async fn run() -> Result<()> {
    // Load config
    let (net_config, app_config, db_config) = load_config()?.into();

    // Initialize RepositoryManager
    let rm = RepositoryManager::new(&db_config).await?;

    // Create app
    let app = create_app(rm, app_config);

    // Start server
    let listener = TcpListener::bind(&net_config.addr()).await?;
    tracing::info!("{:<12} - {}\n", "LISTENING", net_config.addr());
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
        tracing::error!("Application finished with error: {}", e);
        std::process::exit(1);
    }
}
