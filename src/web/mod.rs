pub mod data;
mod error;
mod midware;
pub mod routes;
mod serve;

pub use error::{Error, Result};
pub use serve::serve;

const REQUEST_ID_HEADER: &str = "x-request-id";
