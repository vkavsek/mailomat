use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use derive_more::From;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::sync::Arc;
use strum_macros::AsRefStr;
// use tracing::debug;

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, From, AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    UuidNotInHeader,
    HeaderToStrFail(String),
    Unauthorized,

    #[from]
    DataParsing(super::data::DataParsingError),
    #[from]
    EmailClient(crate::email_client::Error),

    #[from]
    TokioJoin(#[serde_as(as = "DisplayFromStr")] tokio::task::JoinError),
    #[from]
    Io(#[serde_as(as = "DisplayFromStr")] std::io::Error),
    #[from]
    Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
    #[from]
    Tera(#[serde_as(as = "DisplayFromStr")] tera::Error),
}

// Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

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

#[derive(Debug, Serialize, AsRefStr)]
#[serde(tag = "message", content = "detail")]
pub enum ClientError {
    InvalidInput(String),
    ServiceError,
    Unauthorized,
}
