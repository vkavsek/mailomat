//! The middleware implementations

use std::sync::Arc;

use axum::{
    http::{Method, Uri},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, to_value};

use crate::web::{log, Error, Result, REQUEST_ID_HEADER};

/// The response mapper's current main function is to retrieve `web::Error` from response extensions (if it exists),
/// print it and convert it to `ClientError`, which is then sent back to the user.
pub async fn response_mapper(req_method: Method, uri: Uri, resp: Response) -> Result<Response> {
    // Get UUID from headers stored there by SetRequestIdLayer middleware from tower_http
    let uuid = resp
        .headers()
        .get(REQUEST_ID_HEADER)
        .ok_or_else(|| Error::UuidNotInHeader)?
        .to_str()
        .map_err(|_| Error::HeaderToStrFail)?;

    let web_error = resp.extensions().get::<Arc<Error>>().map(|er| {
        tracing::error!("WEB ERROR: {er:?}");
        er.as_ref()
    });
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

    // TODO: Should this be deleted? Probably...
    // log_request is currently infallible so we just ignore the resulting Ok(())
    let _ = log::log_request(
        uuid,
        req_method,
        uri,
        resp.status(),
        web_error,
        client_status_and_error,
    )
    .await;

    Ok(err_resp.unwrap_or(resp))
}
