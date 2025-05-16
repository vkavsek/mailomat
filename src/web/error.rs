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
use routes::LoginError;

pub type WebResult<T> = core::result::Result<T, Error>;

#[derive(Debug, AsRefStr, thiserror::Error)]
pub enum Error {
    #[error("response mapper error: {0}")]
    ResponseMapper(#[from] midware::RespMapError),
    #[error("api news error: {0}")]
    News(#[from] routes::NewsError),
    #[error("api subscribe error: {0}")]
    Subscribe(#[from] routes::SubscribeError),
    #[error("api subscribe confirm error: {0}")]
    SubscribeConfirm(#[from] routes::SubscribeConfirmError),
    #[error("login error: {0}")]
    Login(#[from] routes::LoginError),
    #[error("admin error: {0}")]
    Admin(#[from] routes::AdminError),

    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

impl Error {
    pub fn status_code_and_client_error(&self) -> (StatusCode, ClientError) {
        use routes::{NewsError, SubscribeConfirmError, SubscribeError};
        use Error::*;

        match self {
            News(NewsError::Auth(e)) | Login(LoginError::Auth(e)) => {
                e.status_code_and_client_error()
            }
            SubscribeConfirm(SubscribeConfirmError::SubTokenInDbNotFound) => {
                (StatusCode::UNAUTHORIZED, ClientError::Unauthorized)
            }
            Subscribe(SubscribeError::ValidSubscriberParse(er))
            | SubscribeConfirm(SubscribeConfirmError::DataParsing(er)) => (
                StatusCode::BAD_REQUEST,
                ClientError::InputInvalid(er.to_string()),
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::ServiceError),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        tracing::debug!("{:<12} - into_response(web::Error: {self:?})", "INTO_RESP");

        // Construct a response
        let mut res = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into response so that it can be retrieved later by the middleware.
        res.extensions_mut().insert(Arc::new(self));

        res
    }
}

#[derive(Debug, derive_more::Display)]
pub enum ClientError {
    #[display("Service Error!")]
    ServiceError,
    #[display("Received invalid input: {}", _0)]
    InputInvalid(String),
    #[display("You provided an invalid username or password!")]
    UsernameOrPasswordInvalid,
    #[display("Unauthorized Access")]
    Unauthorized,
}
