use crate::{
    utils::{self, b64u_decode_to_string, b64u_encode},
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
    let result_user_id = user_creds.authenticate(&app_state.database_mgr).await;

    // If we get an authentication error redirect back to the login form but insert the client
    // error as a query parameter in the request so it can be displayed to the user.
    // Otherwise redirect user to the home page.
    let mut resp = StatusCode::SEE_OTHER.into_response();
    if let Err(e) = &result_user_id {
        let (_, client_error) = e.status_code_and_client_error();
        let b64u_client_error_str = b64u_encode(client_error.to_string().as_bytes());
        insert_location_into_response(
            &mut resp,
            &format!("/login?error={}", b64u_client_error_str),
        )?;
    } else {
        insert_location_into_response(&mut resp, "/")?;
    };

    // TODO: keep logged in

    info!("user id: {:?}", result_user_id.ok());
    Ok(resp)
}

fn insert_location_into_response(resp: &mut Response, location: &str) -> WebResult<()> {
    resp.headers_mut().insert(
        axum::http::header::LOCATION,
        location
            .parse()
            .context("login: failed to parse location as header value")?,
    );
    Ok(())
}
