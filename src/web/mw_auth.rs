use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::{extract::Request, middleware::Next, response::Response};
use lazy_regex::regex_captures;
use tower_cookies::{Cookie, Cookies};

use crate::ctx::Ctx;
use crate::error::{Error, Result};
use crate::web::AUTH_TOKEN;

pub async fn mw_require_auth(ctx: Result<Ctx>, req: Request, next: Next) -> Result<Response> {
    println!("->> {:<12} - mw_require_auth", "MIDDLEWARE");
    
    ctx?;

    Ok(next.run(req).await)
}

pub async fn mw_ctx_resolver(cookies: Cookies, mut req: Request, next: Next) -> Result<Response> {
    println!("->> {:<12} - mw_ctx_resolver", "MIDDLEWARE");

    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    let ctx_result = match auth_token
        .ok_or(Error::AuthFailNoAuthTokenCookie)
        .and_then(parse_token)
    {
        Ok((user_id, _exp, _sign)) => {
            // TODO: Real auth-token validation
            Ok(Ctx::new(user_id))
        },
        Err(e) => Err(e),
    };

    // Remove cookie if failed because it wasn't present
    if let Err(Error::AuthFailNoAuthTokenCookie) = ctx_result {
        cookies.remove(Cookie::from(AUTH_TOKEN))
    }

    // Store context in the request extensions
    req.extensions_mut().insert(ctx_result);

    Ok(next.run(req).await)
}

impl<S> FromRequestParts<S> for Ctx
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        println!("->> {:<12} - ctx", "EXTRACTOR");

        parts
            .extensions
            .get::<Result<Ctx>>()
            .ok_or(Error::AuthFailCtxNotInRequestExt)?
            .clone()
    }
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