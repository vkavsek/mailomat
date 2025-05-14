use crate::{
    utils::{self, b64u_decode_to_string},
    web::{
        auth::{self, Credentials},
        WebResult, FLASH_ERROR_MSG,
    },
    AppState,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Form,
};
use secrecy::ExposeSecret;
use tower_cookies::{Cookie, Cookies, Key};
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("authentication error: {0}")]
    Auth(#[from] auth::AuthError),
    #[error("tera template render error: {0}")]
    Tera(#[from] tera::Error),
    #[error("utils error: {0}")]
    Utils(#[from] utils::UtilsError),
}

#[tracing::instrument(name = "login_get", skip(app_state, cookies))]
pub async fn login_get(
    State(app_state): State<AppState>,
    cookies: Cookies,
) -> WebResult<Html<String>> {
    let mut ctx = tera::Context::new();

    let secret_key = Key::from(app_state.cookie_secret.expose_secret());
    let signed_cookies = cookies.signed(&secret_key);

    if let Some(flash_err) = signed_cookies.get(FLASH_ERROR_MSG) {
        let error_msg = b64u_decode_to_string(flash_err.value()).map_err(LoginError::Utils)?;
        ctx.insert("error_message", &error_msg);
        // remove the cookie if we just used it
        signed_cookies.remove(Cookie::new(FLASH_ERROR_MSG, ""));
    }

    let body = app_state
        .templ_mgr
        .render_html_to_string(&ctx, "login_form.html")
        .map_err(LoginError::Tera)?;

    Ok(Html(body))
}

#[tracing::instrument(name = "login_post", skip(app_state, user_creds), fields(username = user_creds.username))]
pub async fn login_post(
    State(app_state): State<AppState>,
    Form(user_creds): Form<Credentials>,
) -> WebResult<Response> {
    // If we get an authentication error redirect to `login_form` is inserted to headers in response mapper
    // alongside the client error message as a signed cookie.
    let user_id = user_creds
        .authenticate(&app_state.database_mgr)
        .await
        .map_err(LoginError::Auth)?;

    // Otherwise redirect user to the home page.
    let mut resp = StatusCode::SEE_OTHER.into_response();
    resp.headers_mut().insert(
        axum::http::header::LOCATION,
        "/".parse().expect("valid parse"),
    );
    // TODO: keep logged in

    info!("user id: {:?}", user_id);
    Ok(resp)
}
