use anyhow::Result;
use reqwest::Client;
use serde_json::json;

mod common;

use common::{print_and_return_body, BASE_URL};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new();

    // GET /me (invalid access token)
    let invalid_access_token = "abcdef12345";

    let me_res = client
        .get(format!("{BASE_URL}/api/auth/me"))
        .bearer_auth(invalid_access_token)
        .send()
        .await?;

    print_and_return_body(me_res).await;

    // POST /login
    let login_res = client
        .post(format!("{BASE_URL}/api/auth/login"))
        .json(&json!({
                "email": "test@socky.com",
                "password": "test"
        }))
        .send()
        .await?;

    let login_body = print_and_return_body(login_res).await.unwrap();

    let access_token = login_body["access_token"]
        .as_str()
        .expect("Response did not contain a valid 'access_token' string")
        .to_string();

    let refresh_token = login_body["refresh_token"]
        .as_str()
        .expect("Response did not contain a valid 'refresh_token' string")
        .to_string();

    // GET /me
    let me_res = client
        .get(format!("{BASE_URL}/api/auth/me"))
        .bearer_auth(access_token)
        .send()
        .await?;

    print_and_return_body(me_res).await;

    // POST /refresh
    let refresh_res = client
        .post(format!("{BASE_URL}/api/auth/refresh"))
        .json(&serde_json::json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await?;

    let refresh_body = print_and_return_body(refresh_res).await.unwrap();

    let _access_token = refresh_body["access_token"]
        .as_str()
        .expect("Response did not contain a valid 'access_token' string")
        .to_string();

    let refresh_token = refresh_body["refresh_token"]
        .as_str()
        .expect("Response did not contain a valid 'refresh_token' string")
        .to_string();

    // POST /logout
    let logout_res = client
        .post(format!("{BASE_URL}/api/auth/logout"))
        .json(&serde_json::json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await?;

    print_and_return_body(logout_res).await;

    Ok(())
}

