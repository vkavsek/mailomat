pub mod auth;
mod error;
pub mod midware;
pub mod routes;
pub mod types;

pub use error::{ClientError, Error, WebResult};

pub const REQUEST_ID_HEADER: &str = "x-request-id";
pub const FLASH_ERROR_MSG: &str = "_flasherr";
