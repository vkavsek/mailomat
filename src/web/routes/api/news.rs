use crate::{
    database::DbManager,
    web::{
        auth::{self, password, AuthError},
        data::UserCredentials,
    },
};
use axum::{extract::State, http::HeaderMap, Json};
use secrecy::SecretString;
use uuid::Uuid;

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
    let creds = auth::basic_schema_user_credentials_from_header_map(headers)
        .await
        .map_err(NewsError::Auth)?;
    news_authenticate(creds, &app_state.database_mgr)
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
        // info!("sending newsletter - user: {}, id: {}", username, user_id);
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

pub async fn news_authenticate(
    credentials: UserCredentials,
    dm: &DbManager,
) -> Result<Uuid, AuthError> {
    let user_id_n_pwd_hash: Option<(Uuid, String)> = sqlx::query_as(
        r#"
    SELECT user_id, password_hash FROM users
    WHERE username = $1
    "#,
    )
    .bind(&credentials.username)
    .fetch_optional(dm.db())
    .await?;

    let (user_id, expected_pwd_hash) = user_id_n_pwd_hash.ok_or(AuthError::UsernameNotFound {
        username: credentials.username,
    })?;

    password::validate_async(credentials.password, expected_pwd_hash).await?;

    Ok(user_id)
}
