use std::sync::Arc;

use axum::{
    http::{Method, Uri},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, to_value};
use tracing::debug;
use uuid::Uuid;

use crate::web::Error;

// FIXME:
pub async fn response_mapper(_req_method: Method, _uri: Uri, resp: Response) -> Response {
    let uuid = Uuid::new_v4();

    let err = resp.extensions().get::<Arc<Error>>().map(|er| er.as_ref());
    let status_and_cl_err = err.map(Error::status_code_and_client_error);

    let err_resp = status_and_cl_err.as_ref().map(|(status, cl_err)| {
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
        debug!("CLIENT ERROR: {client_error_body}");

        (*status, Json(client_error_body)).into_response()
    });

    // TODO: LogLine

    err_resp.unwrap_or(resp)
}
