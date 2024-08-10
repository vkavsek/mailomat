use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use axum::{extract::State, http::HeaderMap, Json};
use secrecy::ExposeSecret;
use tracing::info;
use uuid::Uuid;

use crate::{
    database::DbManager,
    utils::b64_decode_to_string,
    web::{
        data::{News, UserCredentials, ValidEmail},
        WebResult,
    },
    AppState,
};

#[derive(Debug, thiserror::Error)]
pub enum NewsError {
    #[error("auth error: {0}")]
    Auth(#[from] AuthError),
    #[error("email client error: {0}")]
    EmailClient(#[from] crate::email_client::Error),
}

#[tracing::instrument(name = "publishing newsletter issue", skip(headers, app_state, news))]
pub async fn news_publish(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    Json(news): Json<News>,
) -> WebResult<()> {
    let creds = basic_auth(headers).map_err(NewsError::Auth)?;
    let username = creds.username().to_owned();
    let user_id = validate_user_credentials(creds, &app_state.database_mgr)
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

async fn validate_user_credentials(
    credentials: UserCredentials,
    dm: &DbManager,
) -> Result<Uuid, AuthError> {
    let user_id_n_pwd_salt: Option<(Uuid, Uuid)> = sqlx::query_as(
        r#"
    SELECT user_id, pwd_salt FROM users
    WHERE username = $1 AND password = $2
    "#,
    )
    .bind(credentials.username())
    .bind(credentials.password().expose_secret())
    .fetch_optional(dm.db())
    .await?;

    let (user_id, pwd_salt) = user_id_n_pwd_salt.ok_or(AuthError::InvalidLoginParams(format!(
        "no user with matching credentials could be found in the database - username: {}",
        credentials.username()
    )))?;

    // let argon2 = Argon2::default();
    // argon2.hash_password_into(pwd, salt, out)
    // let password_hash =
    //     argon2.hash_password(credentials.password().expose_secret().as_bytes(), &pwd_salt);
    Ok(user_id)
}

fn basic_auth(headers: HeaderMap) -> core::result::Result<UserCredentials, AuthError> {
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

    Ok(UserCredentials::new(uname.into(), pass.to_string().into()))
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("the user doesn't have authorization: {0}")]
    InvalidLoginParams(String),

    #[error("header 'Authorization' is missing from the request")]
    MissingAuthHeader,
    #[error("got invalid utf-8 in 'Authorization' header: {0}")]
    InvalidUtf(String),
    #[error("missing colon in 'Authorization' header - can't split username and password")]
    MissingColon,
    #[error("received the wrong authentication schema. expected: {0}")]
    WrongAuthSchema(String),

    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("base64 decoding error: {0}")]
    Base64Decode(#[from] crate::utils::B64DecodeError),
}
