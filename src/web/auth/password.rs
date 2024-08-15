use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use secrecy::{ExposeSecret, SecretString};
use std::sync::OnceLock;
use uuid::Uuid;

use super::{AuthError, Result};

pub fn get_argon2() -> &'static Argon2<'static> {
    static AUTH_MAN: OnceLock<Argon2> = OnceLock::new();
    AUTH_MAN.get_or_init(|| {
        Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::DEFAULT,
        )
    })
}

pub async fn hash_new_to_string_async(raw_password: SecretString) -> Result<String> {
    tokio::task::spawn_blocking(move || hash_new_to_string(raw_password)).await?
}

pub fn hash_new_to_string(raw_password: SecretString) -> Result<String> {
    let argon2 = get_argon2();

    let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes())
        .map_err(|e| AuthError::Salting(e.to_string()))?;

    let hashed = argon2
        .hash_password(raw_password.expose_secret().as_bytes(), &salt)
        .map_err(|e| AuthError::Hashing(e.to_string()))?
        .to_string();

    Ok(hashed)
}

pub async fn validate_async(raw_password: SecretString, pwd_hash: String) -> Result<()> {
    tokio::task::spawn_blocking(move || validate(raw_password, &pwd_hash)).await?
}
pub fn validate(raw_password: SecretString, pwd_hash_ref: &str) -> Result<()> {
    let argon2 = get_argon2();

    let parsed_hash = PasswordHash::new(pwd_hash_ref)?;

    argon2
        .verify_password(raw_password.expose_secret().as_bytes(), &parsed_hash)
        .map_err(|_| AuthError::PasswordInvalid)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use claims::assert_err;
    use fake::Fake;

    const UTF: &str =
        "0123456789abcčdefghijklmnopqrsštuvwxyzžABCČDEFGHIJKLMNOPQRSŠTUVWXYZŽ!\"#$%&\'()*+,-./:;<=>?@~ˇ^˘°˛`˙´˝¨æ„|€“[]’‘¢{}§¶ŧ←↓→øþ÷×¤´ß—";

    fn fake_valid_pwd() -> String {
        let faker = fake::StringFaker::with(Vec::from(UTF), 12..256);
        faker.fake()
    }

    #[test]
    fn pwd_hashing_and_validate_ok() -> Result<()> {
        let password = SecretString::new(fake_valid_pwd());
        let hashed = hash_new_to_string(password.clone())?;

        validate(password, &hashed)?;
        Ok(())
    }

    #[test]
    fn pwd_hashing_and_validate_not_ok() -> Result<()> {
        let password = SecretString::new(fake_valid_pwd());
        let hashed = hash_new_to_string(password.clone())?;
        let new_password = SecretString::new(fake_valid_pwd());

        let res = validate(new_password, &hashed);

        assert_err!(res);

        Ok(())
    }
}
