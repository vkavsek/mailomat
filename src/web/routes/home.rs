use anyhow::Context;
use axum::{extract::State, response::Html};

use crate::{web::WebResult, AppState};

pub async fn home(State(app_state): State<AppState>) -> WebResult<Html<String>> {
    let tera = app_state.templ_mgr.tera();
    let body = tera
        .render("html/home.html", &tera::Context::new())
        .context("tera failed to render 'html/home.html' template")?;

    Ok(Html(body))
}
