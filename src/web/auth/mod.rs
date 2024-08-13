mod error;

use std::sync::OnceLock;

pub use error::{AuthError, Result};
use secrecy::{ExposeSecret, SecretString};

use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

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

#[derive(Clone)]
pub struct ToHash {
    password: SecretString,
    salt: SecretString,
}
impl ToHash {
    pub fn new(password: SecretString, salt: SecretString) -> Self {
        ToHash { password, salt }
    }
}

pub async fn hash_to_string_async(to_hash: ToHash) -> Result<String> {
    tokio::task::spawn_blocking(move || hash_to_string(to_hash)).await?
}

pub fn hash_to_string(to_hash: ToHash) -> Result<String> {
    let argon2 = get_argon2();

    let salt = SaltString::encode_b64(to_hash.salt.expose_secret().as_bytes())
        .map_err(|e| AuthError::Salting(e.to_string()))?;

    let hashed = argon2
        .hash_password(to_hash.password.expose_secret().as_bytes(), &salt)
        .map_err(|e| AuthError::Hashing(e.to_string()))?
        .to_string();

    Ok(hashed)
}

pub async fn validate_async(to_hash: ToHash, pwd_hash_ref: String) -> Result<()> {
    tokio::task::spawn_blocking(move || validate(to_hash, &pwd_hash_ref)).await?
}
pub fn validate(to_hash: ToHash, pwd_hash_ref: &str) -> Result<()> {
    let argon2 = get_argon2();

    let parsed_hash =
        PasswordHash::new(pwd_hash_ref).map_err(|e| AuthError::Hashing(e.to_string()))?;

    argon2
        .verify_password(to_hash.password.expose_secret().as_bytes(), &parsed_hash)
        .map_err(|_| AuthError::InvalidPassword)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use fake::Fake;
    use uuid::Uuid;

    const ASCII: &str =
        "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&\'()*+,-./:;<=>?@";

    fn _fake_pwd() -> String {
        let faker = fake::StringFaker::with(Vec::from(ASCII), 12..256);
        faker.fake()
    }

    #[test]
    fn pwd_hashing_and_validate_ok() -> Result<()> {
        for _ in 0..3 {
            let password = _fake_pwd();
            let salt = Uuid::new_v4();

            let to_hash = ToHash::new(
                SecretString::new(password),
                SecretString::new(salt.to_string()),
            );

            let hashed = hash_to_string(to_hash.clone())?;

            validate(to_hash, &hashed)?;
        }
        Ok(())
    }
}
