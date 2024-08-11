mod error;

pub use error::AuthError;

use super::data::UserCredentials;
use crate::{database::DbManager, utils::b64_decode_to_string};

use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use axum::http::HeaderMap;
use secrecy::ExposeSecret;
use uuid::Uuid;

pub fn basic_auth(headers: HeaderMap) -> core::result::Result<UserCredentials, AuthError> {
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

pub async fn validate_user_credentials(
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

    let argon2 = Argon2::default();
    // argon2.hash_password_into(pwd, salt, out);
    // let password_hash =
    //     argon2.hash_password(credentials.password().expose_secret().as_bytes(), &pwd_salt);
    Ok(user_id)
}
