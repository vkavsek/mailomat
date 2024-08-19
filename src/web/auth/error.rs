use axum::http::StatusCode;

use crate::web::error::ClientError;

pub type Result<T> = core::result::Result<T, AuthError>;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("username not found in the database: {username}")]
    UsernameNotFound { username: String },
    #[error("username too long")]
    UsernameTooLong,
    #[error("invalid password - doesn't match the user's password from the table")]
    PasswordInvalid,
    #[error("password too long")]
    PasswordTooLong,

    #[error("error parsing the user salt: {0}")]
    Salting(String),
    #[error("hashing error: {0}")]
    Hashing(String),

    #[error("header 'Authorization' is missing from the request")]
    MissingAuthHeader,
    #[error("got invalid utf-8 in 'Authorization' header: {0}")]
    InvalidUtf(String),
    #[error("missing colon in 'Authorization' header - can't split username and password")]
    MissingColon,
    #[error("received the wrong authentication schema. expected: {schema}")]
    WrongAuthSchema { schema: String },

    #[error("password_hash error: {0}")]
    PasswordHash(#[from] argon2::password_hash::Error),
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("base64 decoding error: {0}")]
    Base64Decode(#[from] crate::utils::B64DecodeError),
    #[error("tokio join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
}
impl AuthError {
    pub fn status_code_and_client_error(&self) -> (StatusCode, ClientError) {
        use AuthError::*;

        match self {
            UsernameTooLong | PasswordInvalid | PasswordTooLong | UsernameNotFound { .. } => (
                StatusCode::UNAUTHORIZED,
                ClientError::UsernameOrPasswordInvalid,
            ),
            MissingAuthHeader
            | InvalidUtf(_)
            | MissingColon
            | WrongAuthSchema { .. }
            | Salting(_)
            | Hashing(_) => (StatusCode::UNAUTHORIZED, ClientError::Unauthorized),
            _ => (StatusCode::UNAUTHORIZED, ClientError::ServiceError),
        }
    }
}
