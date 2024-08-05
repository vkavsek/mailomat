//! The middleware implementations

use std::sync::Arc;

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::web::{Error, Result, REQUEST_ID_HEADER};

/// The response mapper's current main function is to retrieve `web::Error` from response extensions (if it exists),
/// print it and convert it to `ClientError`, which is then sent back to the user.
pub async fn response_mapper(resp: Response) -> Result<Response> {
    // Get UUID from headers stored there by SetRequestIdLayer middleware from tower_http
    let uuid = resp
        .headers()
        .get(REQUEST_ID_HEADER)
        .ok_or_else(|| Error::UuidNotInHeader)?
        .to_str()
        .map_err(|e| Error::HeaderToStrFail(e.to_string()))?;

    let web_error = resp.extensions().get::<Arc<Error>>().map(|er| {
        tracing::error!("web error: {er}");
        er.as_ref()
    });
    let client_status_and_error = web_error.map(Error::status_code_and_client_error);

    let err_resp = client_status_and_error.as_ref().map(|(status, cl_err)| {
        let client_error_body = json!({
            "error": {
                "message": cl_err.to_string(),
                "req_id": uuid.to_string(),
            }
        });
        tracing::error!("client error: {client_error_body}");

        (*status, Json(client_error_body)).into_response()
    });

    Ok(err_resp.unwrap_or(resp))
}
