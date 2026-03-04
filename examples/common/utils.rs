use reqwest::Response;
use serde_json::Value;

pub const BASE_URL: &str = "http://localhost:8080";

pub async fn print_and_return_body(res: Response) -> Option<Value> {
    println!("=== Response for {}", res.url());
    println!("=> Status: {}", res.status());
    println!("=> Headers:\n{:#?}", res.headers());

    let opt_body: Option<Value> = res.json().await.ok();

    if let Some(body) = &opt_body {
        println!(
            "=> Response Body:\n{}",
            serde_json::to_string_pretty(body).unwrap()
        );
        println!("===");
    }

    opt_body
}
