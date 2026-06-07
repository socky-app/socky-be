use tower_http::services::ServeDir;

/// Routes for serving static assets.
pub fn static_assets(web_folder: &str) -> ServeDir {
    ServeDir::new(web_folder)
}
