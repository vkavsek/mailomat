//! `web::Error` is the only error that implements IntoResponse.
//! All the other errors that can happen when dealing with requests and responses
//! bubble up to this error.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use strum_macros::AsRefStr;

use super::*;

pub type WebResult<T> = core::result::Result<T, Error>;

#[derive(Debug, AsRefStr, thiserror::Error)]
pub enum Error {
    #[error("response mapper error: {0}")]
    ResponseMapper(#[from] midware::RespMapError),
    #[error("routes error: {0}")]
    News(#[from] routes::api::news::NewsError),
    #[error("routes error: {0}")]
    Subscribe(#[from] routes::api::subscribe::SubscribeError),
    #[error("routes error: {0}")]
    SubscribeConfirm(#[from] routes::api::subscribe_confirm::SubscribeConfirmError),

    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl Error {
    pub fn status_code_and_client_error(&self) -> (StatusCode, ClientError) {
        use routes::api::{
            news::NewsError, subscribe::SubscribeError, subscribe_confirm::SubscribeConfirmError,
        };
        use Error::*;

        match self {
            News(NewsError::Auth(_))
            | SubscribeConfirm(SubscribeConfirmError::SubTokenInDbNotFound) => {
                (StatusCode::UNAUTHORIZED, ClientError::Unauthorized)
            }
            Subscribe(SubscribeError::ValidSubscriberParse(er))
            | SubscribeConfirm(SubscribeConfirmError::DataParsing(er)) => (
                StatusCode::BAD_REQUEST,
                ClientError::InvalidInput(er.to_string()),
            ),
            //  => {
            // }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::ServiceError),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        tracing::debug!("{:<12} - into_response(web::Error: {self:?})", "INTO_RESP");

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
