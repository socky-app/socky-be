//! Entrypoint for the the Socky backend.

use std::net::SocketAddr;

use axum::{middleware, response::Html, routing::get, Router};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use socky_be::{
    common::{config::load_config, state::AppState},
    repository::RepositoryManager,
    web::{
        mw_auth::{mw_ctx_resolver, mw_require_auth},
        mw_res_map::mw_res_map,
        routes_login, routes_static, routes_transaction,
    },
    Result,
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

    // Initialize state
    let state = AppState::new(config, rm);

    let routes_api = routes_transaction::routes(state.rm.clone())
        .route_layer(middleware::from_fn(mw_require_auth)); // TODO: Fix state

    // Create app Router
    let app = Router::new()
        .merge(routes_hello())
        .merge(routes_login::routes())
        .nest("/api", routes_api)
        .layer(middleware::map_response(mw_res_map))
        .layer(middleware::from_fn_with_state(
            state.rm.clone(),
            mw_ctx_resolver,
        )) // TODO: Fix state
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static::serve_dir(&state.app_config));

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080)); // TODO: fix
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("{:<12} - {addr}\n", "LISTENING");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn routes_hello() -> Router {
    Router::new().route(
        "/hello",
        get(|| async {
            debug!("{:<12} - hello", "HANDLER");
            Html("Hello <strong>World!!!</strong>")
        }),
    )
}
