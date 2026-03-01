//! Entrypoint for the the Socky backend.

use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use socky_be::{
    config::load_config,
    create_app,
    repository::RepositoryManager, worker::spawn_token_cleanup_worker,
};

async fn run() -> Result<()> {
    // Load config
    let (net_config, app_config, db_config) = load_config()?.into();

    // Initialize RepositoryManager
    let rm = RepositoryManager::new(&db_config).await?;

    // Create app
    let app = create_app(rm.clone(), app_config);

    // Create helper workers
    spawn_token_cleanup_worker(rm.clone());

    // Start server
    let listener = TcpListener::bind(&net_config.addr()).await?;
    info!("Listening on {}", net_config.addr());
    axum::serve(listener, app).await?;

    Ok(())
}

/// Entrypoint for the backend service
#[tokio::main]
async fn main() {
    // Load env
    dotenvy::dotenv().ok();

    if cfg!(debug_assertions) {
        // Local development: human-readable, pretty console output
        tracing_subscriber::fmt()
            .pretty() // Formats multi-line, colorized output
            .without_time() // Fine for local dev!
            .with_target(false)
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    } else {
        // Production: strict, flat JSON for log aggregators
        tracing_subscriber::fmt()
            .json()
            .flatten_event(true)
            .with_current_span(true)
            .with_span_list(false)
            .with_target(false)
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    }

    if let Err(e) = run().await {
        error!("Application finished with error: {}", e);
        std::process::exit(1);
    }
}
