use axum::{routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};

use crate::{web::Result, model::user::dto::LoginRequestDto};

pub fn routes() -> Router {
    Router::new().route("/api/login", post(api_login))
}

async fn api_login(cookies: Cookies, payload: Json<LoginRequestDto>) -> Result<Json<Value>> {
    tracing::debug!("{:<12} - api_login", "HANDLER");

    todo!();
}
