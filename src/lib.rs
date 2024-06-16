pub mod config;
mod error;
pub mod model;
pub mod web;

pub use error::{Error, Result};
pub use web::serve;

use tracing_subscriber::EnvFilter;

// Initialize tracing for DEV
pub fn init_dbg_tracing() {
    tracing_subscriber::fmt()
        .with_target(false)
        .without_time()
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();
}

// Initialize tracing for PRODUCTION
pub fn init_production_tracing() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
