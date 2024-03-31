use axum::http::{Method, StatusCode, Uri};
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
    status_code: StatusCode,
    web_error: Option<&Error>,
    client_status_and_error: Option<(StatusCode, ClientError)>,
) -> Result<()> {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let uuid = uuid.to_string();
    let req_method = req_method.to_string();
    let uri = uri.to_string();
    let client_error_type = client_status_and_error
        .as_ref()
        .map(|(_, ce)| ce.as_ref().to_string());
    let status_code = client_status_and_error
        .map(|(sc, _)| sc.to_string())
        .unwrap_or(status_code.to_string());
    let web_error_type = web_error.map(|we| we.as_ref().to_string());
    let web_error_data = to_value(web_error)
        .ok()
        .and_then(|mut we| we.get_mut("data").map(|v| v.take()));

    let logline = LogLine {
        timestamp,
        uuid,
        req_method,
        uri,
        status_code,
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
    status_code: String,

    client_error_type: Option<String>,
    web_error_type: Option<String>,
    web_error_data: Option<Value>,
}
