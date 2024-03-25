use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::debug;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Io Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error while creating configuration: {0}")]
    Config(#[from] config::ConfigError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        debug!("{:<12} - into_response(Error: {self:?})", "INTO_RESP");

        // Construct a response
        let mut res = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into response so that it can be retrieved later.
        res.extensions_mut().insert(Arc::new(self));

        res
    }
}
