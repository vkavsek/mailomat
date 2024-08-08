//! The middleware implementations

use std::sync::Arc;

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::web::{self, WebResult, REQUEST_ID_HEADER};

/// The response mapper's current main function is to retrieve `web::Error` from response extensions (if it exists),
/// print it, manipulate the response based on the error and convert it to `ClientError` which is then sent back to the user.
pub async fn response_mapper(resp: Response) -> WebResult<Response> {
    // Get UUID from headers stored there by SetRequestIdLayer middleware from tower_http
    let uuid = resp
        .headers()
        .get(REQUEST_ID_HEADER)
        .ok_or(RespMapError::UuidNotInHeader)?
        .to_str()
        .map_err(|e| RespMapError::HeaderToStrFail(e.to_string()))?;

    let web_error = resp.extensions().get::<Arc<web::Error>>().map(|er| {
        tracing::error!("web error: {er}");
        er.as_ref()
    });
    let client_status_and_error = web_error.map(web::Error::status_code_and_client_error);

    let mut err_resp = client_status_and_error.as_ref().map(|(status, cl_err)| {
        let client_error_body = json!({
            "error": {
                "message": cl_err.to_string(),
                "req_id": uuid.to_string(),
            }
        });
        tracing::error!("client error: {client_error_body}");

        (*status, Json(client_error_body)).into_response()
    });

    if let Some(resp) = err_resp.as_mut() {
        if let Some(er) = web_error {
            use web::routes::api::news::NewsError;
            if matches!(er, web::Error::News(NewsError::Auth(_))) {
                resp.headers_mut().insert(
                    axum::http::header::WWW_AUTHENTICATE,
                    r#"Basic realm="publish""#.parse().expect("valid parse"),
                );
            }
        }
    }
    Ok(err_resp.unwrap_or(resp))
}

#[derive(Debug, thiserror::Error)]
pub enum RespMapError {
    #[error("request id was not in the response header: 'x-request-id'")]
    UuidNotInHeader,
    #[error("failed to convert header to string: {0}")]
    HeaderToStrFail(String),
}
