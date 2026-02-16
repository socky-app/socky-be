use axum::{
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    routing::{any_service, MethodRouter},
};
use tower_http::services::ServeDir;

use crate::config::RouterConfig;

pub fn serve_dir(app_config: &RouterConfig) -> MethodRouter {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Resource not found.")
    }

    any_service(ServeDir::new(&app_config.web_folder).not_found_service(handle_404.into_service()))
}
