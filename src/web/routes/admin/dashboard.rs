use anyhow::anyhow;
use axum::{extract::State, response::Html};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{web::WebResult, AppState};

use super::{AdminError, AdminSession};

#[tracing::instrument(name = "admin_dashboard", skip_all)]
pub async fn dashboard(
    State(app_state): State<AppState>,
    admin_session: AdminSession,
) -> WebResult<Html<String>> {
    let mut ctx = tera::Context::new();

    // TODO: could this be stored in the session and retrieved from session ?
    let username = get_username(app_state.database_mgr.db(), admin_session.user_id()).await?;

    ctx.insert("username", &username);
    let html_body = app_state
        .templ_mgr
        .render_html_to_string(&ctx, "admin_dashboard.html")
        .map_err(|e| anyhow!("template rendering error: {}", e.to_string()))?;

    Ok(Html(html_body))
}

#[tracing::instrument(
    name = "get_username_postgres",
    fields(descr = "Retrieving username from Postgres"),
    skip(pool),
    ret
)]
pub async fn get_username(pool: &PgPool, user_id: Uuid) -> Result<String, AdminError> {
    let username: String = sqlx::query_scalar(
        r#"
        SELECT username 
        FROM users
        WHERE user_id = $1;
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| anyhow!("database error: {}", e.to_string()))?;

    Ok(username)
}
