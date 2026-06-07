use axum::Router;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::web::openapi::ApiDoc;

const SCALAR_HTML: &str = r#"
<!DOCTYPE html>
<html>
  <head>
    <title>Socky API Docs</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1">
  </head>
  <body>
    <script id="api-reference" data-url="/docs/openapi.json"></script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
  </body>
</html>
"#;

/// Public documentation routes.
pub fn public() -> Router {
    let scalar_service = Scalar::with_url("/", ApiDoc::openapi()).custom_html(SCALAR_HTML);

    Router::new().merge(scalar_service).route(
        "/openapi.json",
        axum::routing::get(|| async { axum::Json(ApiDoc::openapi()) }),
    )
}
