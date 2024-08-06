use axum::{extract::State, Json};

use crate::{
    web::{
        data::{News, ValidEmail},
        Result,
    },
    AppState,
};

pub async fn news(State(app_state): State<AppState>, Json(news): Json<News>) -> Result<()> {
    // Get all subscribers that are eligible to receive the newsletter
    // TODO: Add limits
    let emails: Vec<String> = sqlx::query_scalar(
        r#"SELECT email FROM subscriptions
    WHERE status = 'confirmed' "#,
    )
    .fetch_all(app_state.model_mgr.db())
    .await?;

    let subscribers = emails
        .into_iter()
        .filter_map(|email| {
            let res = ValidEmail::parse(email);
            // NOTE: this should never happen, since we validate before we store to DB.
            if let Err(e) = &res {
                tracing::error!(
                    "THIS IS A BUG: trying to parse email from database to ValidEmail: {e}"
                );
            }
            res.ok()
        })
        .collect::<Vec<_>>();

    tracing::debug!("{subscribers:?}");

    if !subscribers.is_empty() {
        // Send batch email newsletter to the subscribers
        app_state
            .email_client
            .send_batch_email(
                &subscribers,
                news.title,
                news.content.html,
                news.content.text,
            )
            .await?;
    }

    Ok(())
}
