use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::sync::Arc;
use strum_macros::AsRefStr;

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, AsRefStr, thiserror::Error)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    #[error("UUID was not in the response header")]
    UuidNotInHeader,
    #[error("Failed to convert header to string. Source {0}")]
    HeaderToStrFail(String),
    #[error("Unauthorized Access")]
    Unauthorized,

    #[error("Data Parsing Error: {0}")]
    DataParsing(#[from] super::data::DataParsingError),

    #[error("Email Client Error: {0}")]
    EmailClient(#[from] crate::email_client::Error),

    #[error("Error awaiting a Tokio task. Src: {0}")]
    TokioJoin(
        #[from]
        #[serde_as(as = "DisplayFromStr")]
        tokio::task::JoinError,
    ),
    #[error("IO error. Src: {0}")]
    Io(
        #[from]
        #[serde_as(as = "DisplayFromStr")]
        std::io::Error,
    ),
    #[error("SQLX error. Src: {0}")]
    Sqlx(
        #[from]
        #[serde_as(as = "DisplayFromStr")]
        sqlx::Error,
    ),
    #[error("Templating error. Src: {0}")]
    Tera(
        #[from]
        #[serde_as(as = "DisplayFromStr")]
        tera::Error,
    ),
}

impl Error {
    pub fn status_code_and_client_error(&self) -> (StatusCode, ClientError) {
        use ClientError::*;

        match self {
            Error::Unauthorized => (StatusCode::UNAUTHORIZED, Unauthorized),
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

#[derive(Debug, Serialize, derive_more::Display)]
#[serde(tag = "message", content = "detail")]
pub enum ClientError {
    #[display(fmt = "Received invalid input: {}", _0)]
    InvalidInput(String),
    #[display(fmt = "Service Error!")]
    ServiceError,
    #[display(fmt = "Unauthorized Access")]
    Unauthorized,
}
