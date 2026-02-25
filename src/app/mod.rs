use axum::{middleware, response::Html, routing::get, Router};

use crate::{
    config::AppConfig,
    repository::RepositoryManager,
    web::{
        auth_router, health_router, middleware::{apply_trace_middleware, auth_middleware}
    },
};

mod state;

pub use state::AppState;

pub fn create_app(rm: RepositoryManager, app_config: AppConfig) -> Router {
    let state = AppState::new(rm, app_config);

    // TODO: Configure CORS, see [here](https://github.com/idaibin/rustzen-admin/blob/main/src/core/app.rs)

    let protected_api = Router::new()
        .nest("/auth", auth_router::protected())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let public_api = Router::new()
        .nest("/auth", auth_router::public())
        .nest("/hello", routes_hello());

    let router = Router::new()
        .nest("/api", protected_api.merge(public_api))
        .nest("/health", health_router::public())
        .with_state(state.clone()); // TODO: add fallback service returning 404 and JSON body?

    apply_trace_middleware(router)
}

// TODO: Remove later
fn routes_hello() -> Router<AppState> {
    Router::new().route(
        "/",
        get(|| async {
            tracing::debug!("{:<15} - hello", "HANDLER");
            Html("Hello <strong>World!!!</strong>")
        }),
    )
}
