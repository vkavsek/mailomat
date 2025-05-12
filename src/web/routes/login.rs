use crate::{
    web::{
        self,
        auth::{self, Credentials},
        types::LoginQueryParams,
        WebResult,
    },
    AppState,
};
use axum::{
    extract::{rejection::QueryRejection, Query, State},
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
    #[error("data parsing error: {0}")]
    DataParsing(#[from] web::types::DataParsingError),
}

pub async fn login_get(
    State(app_state): State<AppState>,
    query_params: Result<Query<LoginQueryParams>, QueryRejection>,
) -> WebResult<Html<String>> {
    let mut ctx = tera::Context::new();

    if let Ok(query) = query_params {
        match query.0.verify(&app_state) {
            // on succesful hmac verification insert the user error msg into the template
            Ok(error_msg) => ctx.insert("error_message", &error_msg),
            // otherwise emit a warning log and do not show the error to the user.
            Err(e) => {
                tracing::warn!(error.message = %e, error.cause_chain = ?e, "failed to verify query parameters with the HMAC tag");
            }
        }
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
