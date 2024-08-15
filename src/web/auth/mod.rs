mod error;
pub mod password;

pub use error::{AuthError, Result};

use unicode_segmentation::UnicodeSegmentation;

use crate::utils::b64_decode_to_string;
use crate::web::data::UserCredentials;

use axum::http::HeaderMap;
pub async fn basic_schema_user_credentials_from_header_map(
    header_map: HeaderMap,
) -> Result<UserCredentials> {
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
        let decoded_creds = b64_decode_to_string(b64_encoded_seg)?;
        let Some((uname, pass)) = decoded_creds.split_once(':') else {
            return Err(AuthError::MissingColon);
        };
        if uname.graphemes(true).count() > 256 {
            return Err(AuthError::UsernameTooLong);
        }
        if pass.graphemes(true).count() > 256 {
            return Err(AuthError::PasswordTooLong);
        }

        Ok(UserCredentials::new(uname.into(), pass.to_string().into()))
    })
    .await?
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::b64_encode;
    use crate::web::data::UserCredentials;
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

            let creds_from_schema =
                basic_schema_user_credentials_from_header_map(header_map).await?;
            let creds = UserCredentials::new(username, SecretString::new(password));

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

        let creds_from_schema = basic_schema_user_credentials_from_header_map(header_map).await;
        assert_err!(creds_from_schema);

        Ok(())
    }
}
