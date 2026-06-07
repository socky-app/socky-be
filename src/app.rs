use axum::{middleware, Router};
use tower_http::services::ServeDir;

use crate::{
    config::AppConfig,
    repository::RepositoryManager,
    web::{
        auth_router, docs_router, fallback_router, health_router,
        middleware::{apply_core_middleware, auth_middleware},
        static_router,
    },
};

mod state;

pub use state::AppState;

pub fn create_app(rm: RepositoryManager, app_config: AppConfig) -> Router {
    let assets_service = static_router::static_assets(&app_config.router.web_folder);

    let state = AppState::new(rm, app_config);

    // TODO: Configure CORS, see [here](https://github.com/idaibin/rustzen-admin/blob/main/src/core/app.rs)

    let protected_api = Router::new()
        .nest("/auth", auth_router::protected())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let public_api = Router::new().nest("/auth", auth_router::public());

    let api_router = protected_api
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
