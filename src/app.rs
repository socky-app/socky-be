//! Application bootstrap and router configuration.
//!
//! Re-exports the AppState and provides `create_app` to wire up protected
//! routes, public routes, middleware pipelines, fallback routers, and static assets.

use axum::{middleware, Router};
use tower_http::services::ServeDir;

use crate::{
    config::AppConfig,
    repository::RepositoryManager,
    web::{
        auth_router, docs_router, fallback_router, health_router,
        middleware::{active_middleware, apply_core_middleware, auth_middleware},
        profile_router, static_router, user_router,
    },
};

mod state;

pub use state::AppState;

/// Instantiates and configures the core Axum `Router` for the application.
///
/// Combines public routes, middleware-protected routes, static files, openapi documentation,
/// and applies core tracking, tracing, and middleware pipelines.
pub fn create_app(rm: RepositoryManager, app_config: AppConfig) -> Router {
    let assets_service = static_router::static_assets(&app_config.router.web_folder);

    let state = AppState::new(rm, app_config);

    // TODO: Configure CORS, see [here](https://github.com/idaibin/rustzen-admin/blob/main/src/core/app.rs)

    let public_api = build_public_api();
    let auth_api = build_auth_api(state.clone());
    let active_api = build_active_api(state.clone());

    let api_router = auth_api
        .merge(active_api)
        .merge(public_api)
        .fallback(fallback_router::fallback);

    let router = Router::new()
        .nest("/api", api_router)
        .nest(
            "/health",
            health_router::public().fallback(fallback_router::fallback),
        )
        .with_state(state.clone())
        .nest("/docs", docs_router::public())
        .fallback_service(assets_service);

    apply_core_middleware(router)

    // TODO: Handle endpoints finishing with a slash.
    // Example, /docs works, but /docs/ is 404.
}

fn build_public_api() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth_router::public())
        .nest("/profile", profile_router::public())
}

fn build_auth_api(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth_router::requires_auth())
        .nest("/user", user_router::requires_auth())
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}

fn build_active_api(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/user", user_router::requires_active())
        .nest("/profile", profile_router::requires_active())
        .route_layer(middleware::from_fn(active_middleware))
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}
