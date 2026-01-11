use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let client = httpc_test::new_client("http://localhost:8080")?;

    client.do_get("/hello").await?.print().await?;

    let req_login = client.do_post(
        "/api/login",
        json!({
            "username": "test1",
            "password": "welcome"
        }),
    );
    req_login.await?.print().await?;

    Ok(())
}
