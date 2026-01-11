use axum::{extract::Request, middleware::Next, response::Response};
use lazy_regex::regex_captures;
use tower_cookies::Cookies;

use crate::error::{Error, Result};
use crate::web::AUTH_TOKEN;

pub async fn mw_require_auth(cookies: Cookies, req: Request, next: Next) -> Result<Response> {
    println!("->> {:<12} - mw_require_auth", "MIDDLEWARE");
    
    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    let (user_id, exp, sign) = auth_token
        .ok_or(Error::AuthFailNoAuthTokenCookie)
        .and_then(parse_token)?;

    // TODO: Real auth-token validation

    Ok(next.run(req).await)
}

fn parse_token(token: String) -> Result<(u64, String, String)> {
    let (_whole, user_id, exp, sign) = regex_captures!(
        r#"^user-(\d+)\.(.+)\.(.+)"#,
        &token
    ).ok_or(Error::AuthFailWrongTokenFormat)?;

    let user_id: u64 = user_id
        .parse()
        .map_err(|_| Error::AuthFailWrongTokenFormat)?;

    Ok((user_id, exp.to_string(), sign.to_string()))
}