use std::sync::Arc;

use axum::{
    http::{Method, Uri},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, to_value};
use uuid::Uuid;

use crate::web::{log, Error};

pub async fn response_mapper(req_method: Method, uri: Uri, resp: Response) -> Response {
    let uuid = Uuid::new_v4();

    let web_error = resp.extensions().get::<Arc<Error>>().map(|er| {
        // TODO: Do you want to record server error in server logs.
        // tracing::error!("SERVER ERROR: {er:?} ID: {uuid}");
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
        // tracing::error!("CLIENT ERROR: {client_error_body} ID: {uuid}");

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

    err_resp.unwrap_or(resp)
}
