pub mod auth;
pub mod data;
mod error;
pub mod midware;
pub mod routes;

pub use error::{Error, WebResult};

pub const REQUEST_ID_HEADER: &str = "x-request-id";
