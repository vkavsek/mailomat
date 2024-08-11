use axum::{extract::State, http::HeaderMap, Json};
use tracing::info;

use crate::{
    web::{
        self,
        data::{News, ValidEmail},
        WebResult,
    },
    AppState,
};

#[derive(Debug, thiserror::Error)]
pub enum NewsError {
    #[error("auth error: {0}")]
    Auth(#[from] web::auth::AuthError),
    #[error("email client error: {0}")]
    EmailClient(#[from] crate::email_client::Error),
}

#[tracing::instrument(name = "publishing newsletter issue", skip(headers, app_state, news))]
pub async fn news_publish(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    Json(news): Json<News>,
) -> WebResult<()> {
    let creds = web::auth::basic_auth(headers).map_err(NewsError::Auth)?;
    let username = creds.username().to_owned();
    let user_id = web::auth::validate_user_credentials(creds, &app_state.database_mgr)
        .await
        .map_err(NewsError::Auth)?;

    // Get all subscribers that are eligible to receive the newsletter
    let emails: Vec<String> = sqlx::query_scalar(
        r#"SELECT email FROM subscriptions
    WHERE status = 'confirmed' "#,
    )
    .fetch_all(app_state.database_mgr.db())
    .await?;

    let subscribers = emails
        .into_iter()
        .filter_map(|email| {
            let res = ValidEmail::parse(&email);
            // NOTE: this should never happen since we validate before we store to DB. 
            // But we still check it if implementation changes.
            if let Err(e) = &res {
                tracing::error!(
                error = ?e,
                    "THIS IS A BUG: a confirmed subscriber is using an invalid email address - email: {email}"
                );
            }
            res.ok()
        })
        .collect::<Vec<_>>();

    tracing::debug!("{subscribers:?}");

    if !subscribers.is_empty() {
        info!("sending newsletter - user: {}, id: {}", username, user_id);
        // Send batch email newsletter to the subscribers
        app_state
            .email_client
            .send_batch_emails(
                &subscribers,
                news.title,
                news.content.html,
                news.content.text,
            )
            .await
            .map_err(NewsError::EmailClient)?;
    }

    Ok(())
}
