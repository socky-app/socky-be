use axum::Router;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::web::openapi::ApiDoc;

/// Public documentation routes.
pub fn public() -> Router {
    Router::new()
        .merge(Scalar::with_url("/", ApiDoc::openapi()))
        .route(
            "/openapi.json",
            axum::routing::get(|| async { axum::Json(ApiDoc::openapi()) }),
        )
}
