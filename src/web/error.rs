use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use strum_macros::AsRefStr;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, AsRefStr, thiserror::Error)]
pub enum Error {
    #[error("request id was not in the response header: 'x-request-id'")]
    UuidNotInHeader,
    #[error("failed to convert header to string: {0}")]
    HeaderToStrFail(String),
    #[error("subscriber token was not found in the database")]
    SubTokenInDbNotFound,

    #[error("data parsing error: {0}")]
    DataParsing(#[from] super::data::DataParsingError),

    #[error("email client error: {0}")]
    EmailClient(#[from] crate::email_client::Error),

    #[error("error awaiting a tokio task: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("templating error: {0}")]
    Tera(#[from] tera::Error),
}

impl Error {
    pub fn status_code_and_client_error(&self) -> (StatusCode, ClientError) {
        use ClientError::*;

        match self {
            Error::SubTokenInDbNotFound => (StatusCode::UNAUTHORIZED, Unauthorized),
            Error::DataParsing(data_er) => {
                (StatusCode::BAD_REQUEST, InvalidInput(data_er.to_string()))
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, ServiceError),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        tracing::debug!("{:<12} - into_response(Error: {self:?})", "INTO_RESP");

        // Construct a response
        let mut res = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into response so that it can be retrieved later.
        res.extensions_mut().insert(Arc::new(self));

        res
    }
}

#[derive(Debug, derive_more::Display)]
pub enum ClientError {
    #[display(fmt = "Received invalid input: {}", _0)]
    InvalidInput(String),
    #[display(fmt = "Service Error!")]
    ServiceError,
    #[display(fmt = "Unauthorized Access")]
    Unauthorized,
}
