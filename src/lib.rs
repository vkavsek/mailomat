pub mod config;
mod error;
pub mod model;
mod routes;
pub mod serve;

pub use error::{Error, Result};
pub use serve::serve;
