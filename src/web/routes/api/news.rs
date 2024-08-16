use crate::{
    database::DbManager,
    web::{
        auth::{self, password, AuthError},
        data::UserCredentials,
    },
};
use axum::{extract::State, http::HeaderMap, Json};
use secrecy::SecretString;
use tracing::info;
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

#[tracing::instrument(name = "Publishing newsletter issue", skip(headers, app_state, news))]
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

    info!("Batch email succesfully sent!");
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

    // Validate Password
    let mut hash = r#"$argon2id$v=19$m=19456,t=2,p=1$DqfdT4sWTiKO8R19hTTtyg$DWeO60WYNYRhAdju0/dzYNhrtmb0jZ6+/ceCHyNKNfk"#.to_string();
    let (user_id, expected_pwd_hash) = user_id_n_pwd_hash.unwrap_or_default();
    // Uuid defaults to NIL - all zeroes.
    // If user_id is NIL we check against the default hash which should always fail.
    if !user_id.is_nil() {
        hash = expected_pwd_hash;
    }
    password::validate_async(credentials.password, SecretString::new(hash)).await?;
    // This should theoretically never happen, since the password validation should fail if the
    // user doesn't exist.
    if user_id.is_nil() {
        return Err(AuthError::UsernameNotFound {
            username: credentials.username,
        });
    }
    info!("Succesfull authentication!");

    Ok(user_id)
}
