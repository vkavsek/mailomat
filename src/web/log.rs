use axum::http::{Method, Uri};
use serde::Serialize;
use serde_json::{json, to_value, Value};
use serde_with::skip_serializing_none;
use tracing::debug;
use uuid::Uuid;

use super::error::ClientError;
use crate::web::{Error, Result};

pub async fn log_request(
    uuid: Uuid,
    req_method: Method,
    uri: Uri,
    web_error: Option<&Error>,
    client_error: Option<&ClientError>,
) -> Result<()> {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let uuid = uuid.to_string();
    let req_method = req_method.to_string();
    let uri = uri.to_string();
    let client_error_type = client_error.map(|ce| ce.as_ref().to_string());
    let web_error_type = web_error.map(|we| we.as_ref().to_string());
    let web_error_data = to_value(web_error)
        .ok()
        .and_then(|mut we| we.get_mut("data").map(|v| v.take()));

    let logline = LogLine {
        timestamp,
        uuid,
        req_method,
        uri,
        client_error_type,
        web_error_type,
        web_error_data,
    };

    // TODO: send logline
    debug!("LOGLINE: {}", json!(logline));

    Ok(())
}

#[skip_serializing_none]
#[derive(Serialize)]
struct LogLine {
    timestamp: String,
    uuid: String,

    req_method: String,
    uri: String,

    client_error_type: Option<String>,
    web_error_type: Option<String>,
    web_error_data: Option<Value>,
}
