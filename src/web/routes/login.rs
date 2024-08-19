use crate::{
    web::{
        auth::{self, Credentials},
        WebResult,
    },
    AppState,
};
use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Form,
};

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("authentication error: {0}")]
    Auth(#[from] auth::AuthError),
    #[error("tera template render error: {0}")]
    Tera(#[from] tera::Error),
}

pub async fn login_form(State(app_state): State<AppState>) -> WebResult<Html<String>> {
    let body = app_state
        .templ_mgr
        .render_html_to_string("login_form.html")
        .map_err(LoginError::Tera)?;
    Ok(Html(body))
}

pub async fn login(
    State(app_state): State<AppState>,
    Form(user_creds): Form<Credentials>,
) -> WebResult<Response> {
    user_creds
        .authenticate(&app_state.database_mgr)
        .await
        .map_err(LoginError::Auth)?;

    // TODO: keep logged in

    let mut resp = StatusCode::SEE_OTHER.into_response();
    resp.headers_mut().insert(
        axum::http::header::LOCATION,
        "/".parse().context("login: failed to parse header value")?,
    );

    Ok(resp)
}
