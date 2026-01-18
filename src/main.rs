//! Backend of the Socky application

use std::net::SocketAddr;

use axum::{middleware, response::Html, routing::get, Router};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use crate::{
    model::ModelController,
    web::{
        mw_auth::{mw_ctx_resolver, mw_require_auth},
        mw_res_map::mw_res_map,
    },
};

pub use self::error::{Error, Result};

mod config;
mod ctx;
mod error;
mod log;
mod model;
mod web;

/// Entrypoint for the backend service
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .without_time() // For early local development.
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Initialize ModelController
    let mc = ModelController::new().await?;

    let routes_api = web::routes_transaction::routes(mc.clone())
        .route_layer(middleware::from_fn(mw_require_auth));

    // Create app Router
    let app = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .nest("/api", routes_api)
        .layer(middleware::map_response(mw_res_map))
        .layer(middleware::from_fn_with_state(mc.clone(), mw_ctx_resolver))
        .layer(CookieManagerLayer::new());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
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
