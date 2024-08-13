pub type Result<T> = core::result::Result<T, AuthError>;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("the user wasn't found: {0}")]
    UserNotFound(String),
    #[error("invalid password input")]
    InvalidPassword,
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
    #[error("received the wrong authentication schema. expected: {0}")]
    WrongAuthSchema(String),

    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("base64 decoding error: {0}")]
    Base64Decode(#[from] crate::utils::B64DecodeError),
    #[error("tokio join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
}
