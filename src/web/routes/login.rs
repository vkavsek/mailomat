use crate::{
    utils::{self, b64u_decode_to_string},
    web::{
        auth::{self, Credentials},
        data::QueryError,
        WebResult,
    },
    AppState,
};
use anyhow::Context;
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
    #[error("base64-url decoding error: {0}")]
    Base64(#[from] utils::B64DecodeError),
}

pub async fn login_form(
    State(app_state): State<AppState>,
    Query(query_err_str): Query<QueryError>,
) -> WebResult<Html<String>> {
    let file = "login_form.html";
    let mut ctx = tera::Context::new();

    if let Some(er) = query_err_str.error {
        let er = b64u_decode_to_string(&er).map_err(LoginError::Base64)?;
        ctx.insert("error_message", &er);
    }

    let body = app_state
        .templ_mgr
        .render_html_to_string(&ctx, file)
        .map_err(LoginError::Tera)?;

    Ok(Html(body))
}

#[tracing::instrument(skip(app_state, user_creds), fields(username = user_creds.username))]
pub async fn login(
    State(app_state): State<AppState>,
    Form(user_creds): Form<Credentials>,
) -> WebResult<Response> {
    // If we get an authentication error redirect is inserted to headers in response mapper
    let user_id = user_creds
        .authenticate(&app_state.database_mgr)
        .await
        .map_err(LoginError::Auth)?;

    // Otherwise redirect user to the home page.
    let mut resp = StatusCode::SEE_OTHER.into_response();
    resp.headers_mut().insert(
        axum::http::header::LOCATION,
        "/".parse()
            .context("login: failed to parse location as header value")?,
    );
    // TODO: keep logged in

    info!("user id: {:?}", user_id);
    Ok(resp)
}
