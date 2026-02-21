use anyhow::Result;
use reqwest::Client;
use serde_json::json;

mod common;

use common::{print_and_return_body, BASE_URL};
#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new();

    // POST /login
    let login_res = client
        .post(format!("{BASE_URL}/api/auth/login"))
        .json(&json!({
                "email": "admin@socky.com",
                "password": "123"
        }))
        .send()
        .await?;

    print_and_return_body(login_res).await.unwrap();

    Ok(())
}
