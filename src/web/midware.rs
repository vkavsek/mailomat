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

    let resp_status_code = resp.status();
    if resp_status_code.as_u16() >= 400 {
        tracing::error!("{resp_status_code}");
    }

    let web_error = resp.extensions().get::<Arc<Error>>().map(|er| {
        // TODO: Do you want to record server error in server logs.
        tracing::error!("SERVER ERROR: {er:?} ID: {uuid}");
        er.as_ref()
    });
    let status_and_cl_err = web_error.map(Error::status_code_and_client_error);

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
        tracing::error!("CLIENT ERROR: {client_error_body} ID: {uuid}");

        (*status, Json(client_error_body)).into_response()
    });

    #[allow(clippy::redundant_pattern_matching)]
    if let Ok(_) = log::log_request(
        uuid,
        req_method,
        uri,
        web_error,
        status_and_cl_err.as_ref().map(|(_, er)| er),
    )
    .await
    {}

    err_resp.unwrap_or(resp)
}
