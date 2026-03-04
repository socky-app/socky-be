use anyhow::Result;
use reqwest::Client;

#[path = "common/utils.rs"]
mod utils;

use utils::{print_and_return_body, BASE_URL};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new();

    // GET /health/live
    let login_res = client.get(format!("{BASE_URL}/health/live")).send().await?;

    print_and_return_body(login_res).await;

    // GET /health/ready
    let login_res = client
        .get(format!("{BASE_URL}/health/ready"))
        .send()
        .await?;

    print_and_return_body(login_res).await;

    Ok(())
}
