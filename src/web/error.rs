use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use derive_more::From;
use serde::Serialize;
use std::sync::Arc;
use strum_macros::AsRefStr;
use tracing::debug;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    SqlxCore(sqlx::Error),
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
        #[allow(clippy::match_single_binding)]
        match self {
            _ => (StatusCode::INTERNAL_SERVER_ERROR, ServiceError),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        debug!("{:<20} - into_response(Error: {self:?})", "INTO_RESP");

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
    ConfigurationError,
    ServiceError,
}
