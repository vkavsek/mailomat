//! Contains user `Credentials` data structure, its implementations and tests for them.
//! `Credentials` is a deserializable struct containing username and password (based on `users` table in DB)
//! You can use `authenticate()` method to try and authenticate the user from the DB.

use axum::http::HeaderMap;
use secrecy::SecretString;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use crate::database::DbManager;
use crate::utils::b64_decode_to_string;

use super::{password, AuthError, Result};

/// User credentials
#[derive(Debug, serde::Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: SecretString,
}

impl Credentials {
    pub fn new(username: String, password: SecretString) -> Self {
        Credentials { username, password }
    }

    /// Try to authenticate the user using the information from the `users` table in the DB.
    pub async fn authenticate(self, dm: &DbManager) -> Result<Uuid> {
        let user_id_n_pwd_hash: Option<(Uuid, String)> = sqlx::query_as(
            r#"
    SELECT user_id, password_hash FROM users
    WHERE username = $1
    "#,
        )
        .bind(&self.username)
        .fetch_optional(dm.db())
        .await
        .map_err(|er| anyhow::anyhow!("authenticating credentials: {}", er))?;

        // Validate Password
        let mut hash = r#"$argon2id$v=19$m=19456,t=2,p=1$DqfdT4sWTiKO8R19hTTtyg$DWeO60WYNYRhAdju0/dzYNhrtmb0jZ6+/ceCHyNKNfk"#.to_string();
        let (user_id, expected_pwd_hash) = user_id_n_pwd_hash.unwrap_or_default();
        // Uuid defaults to NIL - all zeroes.
        // If user_id is NIL we will check against the default hash which should always fail.
        if !user_id.is_nil() {
            hash = expected_pwd_hash;
        }
        password::validate_async(self.password, SecretString::from(hash)).await?;
        // This should theoretically never happen, since the password validation should fail if the
        // user doesn't exist.
        if user_id.is_nil() {
            return Err(AuthError::UsernameNotFound {
                username: self.username,
            });
        }
        tracing::info!("Succesfull authentication!");

        Ok(user_id)
    }
}

/// Try to parse credentials from headers
pub async fn credentials_from_header_map_basic_schema(
    header_map: HeaderMap,
) -> Result<Credentials> {
    tokio::task::spawn_blocking(move || {
        let header_val = header_map
            .get("Authorization")
            .ok_or(AuthError::MissingAuthHeader)?
            .to_str()
            .map_err(|e| AuthError::InvalidUtf(e.to_string()))?;
        let b64_encoded_seg =
            header_val
                .strip_prefix("Basic ")
                .ok_or(AuthError::WrongAuthSchema {
                    schema: "Basic".to_string(),
                })?;
        let decoded_creds = b64_decode_to_string(b64_encoded_seg)
            .map_err(|er| anyhow::anyhow!("credentials from header map: {}", er))?;
        let Some((uname, pass)) = decoded_creds.split_once(':') else {
            return Err(AuthError::MissingColon);
        };
        if uname.graphemes(true).count() > 256 {
            return Err(AuthError::UsernameTooLong);
        }
        if pass.graphemes(true).count() > 256 {
            return Err(AuthError::PasswordTooLong);
        }

        Ok(Credentials::new(uname.into(), pass.to_string().into()))
    })
    .await
    .map_err(|er| anyhow::anyhow!("credentials from header map: {}", er))?
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::b64_encode;
    use anyhow::Result;
    use axum::http::HeaderMap;
    use claims::assert_err;
    use fake::Fake;
    use secrecy::{ExposeSecret, SecretString};
    use uuid::Uuid;

    const UTF: &str =
        "0123456789abcčdefghijklmnopqrsštuvwxyzžABCČDEFGHIJKLMNOPQRSŠTUVWXYZŽ!\"#$%&\'()*+,-./:;<=>?@~ˇ^˘°˛`˙´˝¨æ„|€“[]’‘¢{}§¶ŧ←↓→øþ÷×¤´ß—";

    fn fake_valid_pwd() -> String {
        let faker = fake::StringFaker::with(Vec::from(UTF), 12..256);
        faker.fake()
    }

    fn fake_invalid_pwd() -> String {
        let faker = fake::StringFaker::with(Vec::from(UTF), 256..1024);
        faker.fake()
    }

    #[tokio::test]
    async fn user_credentials_from_header_map_valid() -> Result<()> {
        for _ in 0..100 {
            let username = Uuid::new_v4().to_string();
            let password = fake_valid_pwd();
            let b64_encoded_uname_and_password = b64_encode(format!("{username}:{password}"));

            let basic_auth = format!("Basic {b64_encoded_uname_and_password}");
            let mut header_map = HeaderMap::new();
            header_map.append(axum::http::header::AUTHORIZATION, basic_auth.parse()?);

            let creds_from_schema = credentials_from_header_map_basic_schema(header_map).await?;
            let creds = Credentials::new(username, SecretString::from(password));

            assert_eq!(creds_from_schema.username, creds.username);
            assert_eq!(
                creds_from_schema.password.expose_secret(),
                creds.password.expose_secret()
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn user_credentials_from_header_map_invalid() -> Result<()> {
        let username = Uuid::new_v4().to_string();
        let password = fake_invalid_pwd();
        let b64_encoded_uname_and_password = b64_encode(format!("{username}:{password}"));

        let basic_auth = format!("Basic {b64_encoded_uname_and_password}");
        let mut header_map = HeaderMap::new();
        header_map.append(axum::http::header::AUTHORIZATION, basic_auth.parse()?);

        let creds_from_schema = credentials_from_header_map_basic_schema(header_map).await;
        assert_err!(creds_from_schema);

        Ok(())
    }
}
