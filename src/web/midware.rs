//! The middleware implementations

use crate::{
    utils::b64u_encode,
    web::{self, WebResult, FLASH_ERROR_MSG, REQUEST_ID_HEADER},
    AppState,
};

use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use secrecy::ExposeSecret;
use serde_json::json;

use tower_cookies::{Cookie, Cookies, Key};
use web::routes::{api::news::NewsError, login::LoginError};

#[derive(Debug, thiserror::Error)]
pub enum RespMapError {
    #[error("request id was not in the response header: 'x-request-id'")]
    UuidNotInHeader,
    #[error("failed to convert header to string: {0}")]
    HeaderToStrFail(String),
}

/// This response mapper's current main function is to retrieve `web::Error` from response extensions (if it exists),
/// print it, convert it to `ClientError` and use it to manipulate the response, which is then sent back to the user.
pub async fn error_handle_response_mapper(
    State(app_state): State<AppState>,
    cookies: Cookies,
    resp: Response,
) -> WebResult<Response> {
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

    let err_resp = match web_error {
        // If LoginError::AuthError is encountered create a redirect response back to the login form
        // but insert the client error as a query parameter in the request so it can be displayed to the user
        Some(web::Error::Login(LoginError::Auth(_))) => {
            let client_error = client_status_and_error
                .map(|(_, cl_er)| cl_er)
                .expect("checked above");

            tracing::error!("client error: {client_error:?}");

            // NOTE: it would probably be fine to just block here
            let b64u_client_error_str = tokio::task::spawn_blocking(move || {
                b64u_encode(client_error.to_string().as_bytes())
            })
            .await
            .map_err(|er| anyhow::anyhow!("midware: {er}"))?;

            // insert error message as a signed cookie
            let mut resp = StatusCode::SEE_OTHER.into_response();
            let headers = resp.headers_mut();
            // insert the redirection location
            headers.insert(
                header::LOCATION,
                "/login"
                    .parse()
                    .context("midware: failed to parse header value")?,
            );
            // insert the error msg signed cookie
            let key = Key::from(app_state.cookie_secret.expose_secret());
            cookies
                .signed(&key)
                .add(Cookie::new(FLASH_ERROR_MSG, b64u_client_error_str));
            Some(resp)
        }
        // otherwise create default response
        Some(er) => client_status_and_error.as_ref().map(|(status, cl_err)| {
            tracing::error!("client error: {cl_err:?}");
            let client_error_body = json!({
                "error": {
                    "message": cl_err.to_string(),
                    "req_id": uuid.to_string(),
                }
            });

            // Check if authentication error was encountered on the news path and insert appropriate
            // headers if so.
            let mut resp = (*status, Json(client_error_body)).into_response();
            if matches!(er, web::Error::News(NewsError::Auth(_))) {
                resp.headers_mut().insert(
                    header::WWW_AUTHENTICATE,
                    r#"Basic realm="publish""#.parse().expect("valid parse"),
                );
            }
            resp
        }),
        None => None,
    };

    Ok(err_resp.unwrap_or(resp))
}
