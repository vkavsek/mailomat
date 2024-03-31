use std::sync::Arc;

use axum::{
    http::{Method, Uri},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, to_value};
use tracing::debug;

use crate::web::{log, Error, Result, REQUEST_ID_HEADER};

pub async fn response_mapper(req_method: Method, uri: Uri, resp: Response) -> Result<Response> {
    // Get UUID from headers, stored there by SetRequestIdLayer
    let uuid = resp
        .headers()
        .get(REQUEST_ID_HEADER)
        .ok_or_else(|| Error::UuidNotInHeader)?
        .to_str()
        .map_err(|_| Error::HeaderToStrFail)?;

    debug!("{:<12} - response_mapper - {}", "MIDDLEWARE", uuid);

    let web_error = resp.extensions().get::<Arc<Error>>().map(|er| er.as_ref());
    let client_status_and_error = web_error.map(Error::status_code_and_client_error);

    let err_resp = client_status_and_error.as_ref().map(|(status, cl_err)| {
        let client_error = to_value(cl_err).ok();
        let message = client_error.as_ref().and_then(|v| v.get("message"));
        let detail = client_error.as_ref().and_then(|v| v.get("detail"));

        let client_error_body = json!({
            "error": {
                "message": message,
                "data": {
                    "req_id": uuid.to_string(),
                    "detail": detail,
                }
            }
        });
        tracing::error!("CLIENT ERROR: {client_error_body} ID: {uuid}");

        (*status, Json(client_error_body)).into_response()
    });

    #[allow(clippy::redundant_pattern_matching)]
    if let Ok(_) = log::log_request(
        uuid,
        req_method,
        uri,
        resp.status(),
        web_error,
        client_status_and_error,
    )
    .await
    {}

    Ok(err_resp.unwrap_or(resp))
}
