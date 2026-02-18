use axum::{middleware, response::Html, routing::get, Router};
use tower_cookies::CookieManagerLayer;

use crate::{
    config::AppConfig,
    repository::RepositoryManager,
    web::{
        middleware::{auth, request},
        routes_login, routes_static, routes_transaction,
    },
};

mod state;

pub use state::AppState;

pub fn create_app(rm: RepositoryManager, app_config: AppConfig) -> Router {
    let state = AppState::new(rm, app_config);

    let routes_api = routes_transaction::routes(state.clone()).route_layer(
        middleware::from_fn_with_state(state.clone(), auth::auth_middleware),
    );

    Router::new()
        .merge(routes_hello())
        .merge(routes_login::routes())
        .nest("/api", routes_api)
        .layer(middleware::from_fn(request::request_middleware))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static::serve_dir(&state.config.router))
}

// TODO: Remove later
fn routes_hello() -> Router {
    Router::new().route(
        "/hello",
        get(|| async {
            tracing::debug!("{:<12} - hello", "HANDLER");
            Html("Hello <strong>World!!!</strong>")
        }),
    )
}
