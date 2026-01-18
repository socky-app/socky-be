use axum::{
    http::{Method, Uri},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::debug;
use uuid::Uuid;

use crate::ctx::Ctx;
use crate::{log::log_request, Error, Result};

pub async fn mw_res_map(ctx: Result<Ctx>, uri: Uri, req_method: Method, res: Response) -> Response {
    debug!("{:<12} - main_response_mapper", "RES_MAPPER");

    let uuid = Uuid::new_v4();

    // Get eventual response error
    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    // Build new response
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });

            debug!("{:<12}\n{client_error_body}", "CLIENT ERROR BODY");
            (*status_code, Json(client_error_body)).into_response()
        });

    let client_error = client_status_error.unzip().1;
    let _ = log_request(uuid, req_method, uri, ctx.ok(), service_error, client_error).await;

    debug!("\n");
    error_response.unwrap_or(res)
}
