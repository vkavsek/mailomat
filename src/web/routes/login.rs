use crate::{
    utils::{self, b64u_decode_to_string, hex_decode},
    web::{
        auth::{self, Credentials},
        types::LoginQueryParams,
        WebResult,
    },
    AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Form,
};
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

pub async fn login_get(
    State(app_state): State<AppState>,
    Query(query_params): Query<Option<LoginQueryParams>>,
) -> WebResult<Html<String>> {
    let mut ctx = tera::Context::new();

    if let Some(LoginQueryParams {
        error_b64u,
        tag_hex,
    }) = query_params
    {
        let error = b64u_decode_to_string(&error_b64u).map_err(LoginError::Utils)?;
        let tag = hex_decode(tag_hex).map_err(LoginError::Utils)?;
        ctx.insert("error_message", &error);
    }

    let body = app_state
        .templ_mgr
        .render_html_to_string(&ctx, "login_form.html")
        .map_err(LoginError::Tera)?;

    Ok(Html(body))
}

#[tracing::instrument(skip(app_state, user_creds), fields(username = user_creds.username))]
pub async fn login_post(
    State(app_state): State<AppState>,
    Form(user_creds): Form<Credentials>,
) -> WebResult<Response> {
    // If we get an authentication error redirect to `login_form` is inserted to headers in response mapper
    // alongside the client error message and an hmac tag to authenticate the error message.
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
