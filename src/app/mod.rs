use axum::{middleware, response::Html, routing::get, Router};

use crate::{
    config::AppConfig,
    repository::RepositoryManager,
    web::{
        auth_router,
        middleware::{auth_middleware, request_middleware},
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

    Router::new()
        .nest("/api", protected_api.merge(public_api))
        .layer(middleware::from_fn(request_middleware)) // TODO: Implement logic to filter logs for dummy requests, maybe use TraceLayer
        .with_state(state.clone())
    // TODO: add fallback service returnin 404 and JSON body?
}

// TODO: Remove later
fn routes_hello() -> Router<AppState> {
    Router::new().route(
        "/",
        get(|| async {
            tracing::debug!("{:<12} - hello", "HANDLER");
            Html("Hello <strong>World!!!</strong>")
        }),
    )
}
