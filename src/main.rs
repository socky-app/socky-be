//! Backend of the Socky application

use std::net::SocketAddr;

use axum::{
    http::{Method, Uri},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

use crate::{ctx::Ctx, log::log_request, model::ModelController};

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
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    // Create app Router
    let app = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .nest("/api", routes_api)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("{:<12} - {addr}\n", "LISTENING");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn main_response_mapper(
    ctx: Result<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    debug!("{:<12} - main_response_mapper", "RES_MAPPER");

    let uuid = Uuid::new_v4();

    // Get eventual response error
    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    // Build new response
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });

            debug!("{:<12}\n{client_error_body}", "CLIENT ERROR BODY");
            (*status_code, Json(client_error_body)).into_response()
        });

    let client_error = client_status_error.unzip().1;
    let _ = log_request(uuid, req_method, uri, ctx.ok(), service_error, client_error).await;

    debug!("\n");
    error_response.unwrap_or(res)
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
