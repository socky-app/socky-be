use axum::response::IntoResponse;

use crate::web::WebError;

pub async fn fallback() -> impl IntoResponse {
    WebError::NotFound("the requested route does not exist.".to_string()).into_response()
}
