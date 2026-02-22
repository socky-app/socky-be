use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use tracing::debug;
use uuid::Uuid;

use crate::context::{CurrentUser, ErrorDetails};

pub struct LogController;

impl LogController {
    pub async fn log_request(
        uuid: String,
        method: String,
        uri: String,
        curr_user: Option<&CurrentUser>,
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
            user: curr_user.cloned(),

            client_message: error_details.map(|e| e.client_message.clone()),
            error_type: error_details.map(|e| e.error_type.clone()),
            error_data: error_details.map(|e| e.error_data.clone()),
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
    user: Option<CurrentUser>,

    // HTTP request attributes
    uri: String,
    method: String,

    // Error attributes
    client_message: Option<String>,
    error_type: Option<String>,
    error_data: Option<String>,
}