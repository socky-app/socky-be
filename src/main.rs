//! Backend of the Socky application

use std::net::SocketAddr;

use axum::{
    middleware,
    response::{Html, Response},
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::model::ModelController;

pub use self::error::{Error, Result};

mod config;
mod error;
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
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    // Create app Router
    let app = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .nest("/api", routes_api)
        .layer(middleware::map_response(main_response_mapper))
        .layer(CookieManagerLayer::new());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("{:<12} - {addr}\n", "LISTENING");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn main_response_mapper(res: Response) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    println!();
    res
}

fn routes_hello() -> Router {
    Router::new().route(
        "/hello",
        get(|| async {
            println!("->> {:<12} - hello", "HANDLER");
            Html("Hello <strong>World!!!</strong>")
        }),
    )
}
