use axum::{extract::State, http::HeaderMap, Json};

use crate::{
    database::DbManager,
    utils::b64_decode_to_string,
    web::{
        data::{Credentials, News, ValidEmail},
        WebResult,
    },
    AppState,
};

#[derive(Debug, thiserror::Error)]
pub enum NewsError {
    #[error("auth error: {0}")]
    Auth(#[from] AuthError),
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("email client error: {0}")]
    EmailClient(#[from] crate::email_client::Error),
}

pub async fn news(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    Json(news): Json<News>,
) -> WebResult<()> {
    let creds = basic_auth(headers).map_err(NewsError::Auth)?;

    // Get all subscribers that are eligible to receive the newsletter
    let emails: Vec<String> = sqlx::query_scalar(
        r#"SELECT email FROM subscriptions
    WHERE status = 'confirmed' "#,
    )
    .fetch_all(app_state.database_mgr.db())
    .await
    .map_err(NewsError::Sqlx)?;

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

async fn validate_credentials(credentials: Credentials, dm: DbManager) {}

fn basic_auth(headers: HeaderMap) -> core::result::Result<Credentials, AuthError> {
    let header_val = headers
        .get("Authorization")
        .ok_or(AuthError::MissingAuthHeader)?
        .to_str()
        .map_err(|e| AuthError::InvalidUtf(e.to_string()))?;
    let b64_encoded_seg = header_val
        .strip_prefix("Basic ")
        .ok_or(AuthError::WrongAuthSchema("Basic".to_string()))?;
    let decoded_creds = b64_decode_to_string(b64_encoded_seg)?;
    let Some((uname, pass)) = decoded_creds.split_once(':') else {
        return Err(AuthError::MissingColon);
    };

    Ok(Credentials::new(uname.into(), pass.to_string().into()))
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("header 'Authorization' is missing from the request")]
    MissingAuthHeader,
    #[error("got invalid utf-8 in 'Authorization' header: {0}")]
    InvalidUtf(String),
    #[error("missing colon in 'Authorization' header - can't split username and password")]
    MissingColon,
    #[error("received the wrong authentication schema. expected: {0}")]
    WrongAuthSchema(String),

    #[error("base64 decoding error: {0}")]
    Base64Decode(#[from] crate::utils::B64DecodeError),
}
