use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::{Method, Uri};
use serde::Serialize;
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use tracing::debug;
use uuid::Uuid;

use crate::web::ErrorDetails;

pub struct LogController;

impl LogController {
    pub async fn log_request(
        uuid: String,
        method: String,
        uri: String,
        // ctx: Option<Ctx>,
        error_details: Option<&ErrorDetails>,
    ) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let log_line = RequestLogLine {
            uuid,
            timestamp: timestamp.to_string(),
            uri,
            method,
            user_id: Some(0), // TODO: Fix

            client_message: error_details.map(|e| e.client_message.clone()),
            service_error_type: error_details.map(|e| e.error_type.clone()),
            service_error_data: error_details.map(|e| e.error_data.clone()),
        };

        tracing::info!("{:<15} - {}\n", "LOG LINE", json!(log_line));
    }
}

#[skip_serializing_none]
#[derive(Serialize)]
struct RequestLogLine {
    // General
    uuid: String,
    timestamp: String,

    // User and context attributes
    user_id: Option<u64>,

    // HTTP request attributes
    uri: String,
    method: String,

    // Error attributes
    client_message: Option<String>,
    service_error_type: Option<String>,
    service_error_data: Option<String>,
}