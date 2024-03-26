pub mod config;
mod error;
pub mod model;
pub mod web;

pub use error::{Error, Result};
pub use web::serve;
