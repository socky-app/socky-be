use axum::{Router, middleware, response::Html, routing::get};
use tower_cookies::CookieManagerLayer;
use tracing::debug;

use crate::{
    common::config::{AuthConfig, RouterConfig},
    repository::RepositoryManager,
    web::{mw_auth, mw_res_map, routes_login, routes_static, routes_transaction},
};

mod state;

pub(in crate) use state::AppState;

pub struct AppConfig {
    pub router: RouterConfig,
    pub auth: AuthConfig,
}

pub fn create_app(rm: RepositoryManager, app_config: AppConfig) -> Router {
    let state = AppState::new(rm, app_config);

    let routes_api = routes_transaction::routes(state.clone())
        .route_layer(middleware::from_fn(mw_auth::mw_require_auth)); // TODO: Fix state

    Router::new()
        .merge(routes_hello())
        .merge(routes_login::routes())
        .nest("/api", routes_api)
        .layer(middleware::map_response(mw_res_map::mw_res_map))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            mw_auth::mw_ctx_resolver,
        )) // TODO: Fix state
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static::serve_dir(&state.config.router))
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