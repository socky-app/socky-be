use axum::{Json, Router, routing::post};
use serde::Deserialize;
use serde_json::{Value, json};
use tower_cookies::{Cookie, Cookies};

use crate::{Error, Result, web};

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

pub fn routes() -> Router {
    Router::new().route("/api/login", post(api_login))
}

async fn api_login(cookies: Cookies, payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");

    // TODO: Implement read db/auth logic.

    if payload.username != "test1" || payload.password != "welcome" {
        return Err(Error::LoginFail);
    }

    // FIXME: Implement real auth-token generation/signature 
    cookies.add(Cookie::new(web::AUTH_TOKEN, "use-1.exp.sign"));

    // Create the success body
    let body = Json(json!({
        "result": {
            "success": true
        }
    }));

    Ok(body)
}