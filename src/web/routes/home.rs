use anyhow::Context;
use axum::{extract::State, response::Html};

use crate::{web::WebResult, AppState};

pub async fn home(State(app_state): State<AppState>) -> WebResult<Html<String>> {
    let body = app_state
        .templ_mgr
        .render_html_to_string("home.html")
        .context("tera failed to render 'html/home.html' template")?;

    Ok(Html(body))
}
