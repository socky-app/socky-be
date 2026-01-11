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

    let req_create_0 = client.do_post(
        "/api/transaction",
        json!({
            "title": "Pão de Açúcar",
            "value": {
                "amount": 133.65,
                "currency": "BRL",
            }
        }),
    );
    req_create_0.await?.print().await?;

    let req_create_1 = client.do_post(
        "/api/transaction",
        json!({
            "title": "Uber",
            "value": {
                "amount": 34.12,
                "currency": "BRL",
            }
        }),
    );
    req_create_1.await?.print().await?;

    let req_list = client.do_get("/api/transaction");
    req_list.await?.print().await?;

    let req_del = client.do_delete("/api/transaction/0");
    req_del.await?.print().await?;

    let req_get = client.do_get("/api/transaction/1");
    req_get.await?.print().await?;

    Ok(())
}
