use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use tera::Context;

use crate::{web::WebResult, AppState};

pub async fn get_change_password(State(app_state): State<AppState>) -> WebResult<Html<String>> {
    let ctx = Context::new();
    let html = app_state
        .templ_mgr
        .render_html_to_string(&ctx, "change_admin_password.html");

    todo!()
}

pub async fn post_change_password() -> impl IntoResponse {
    todo!()
}
