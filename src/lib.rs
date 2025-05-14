mod app;
pub mod config;
pub mod database;
pub mod email_client;
mod error;
pub mod redis_manager;
pub mod templ_manager;
pub mod utils;
pub mod web;

pub use app::*;
pub use email_client::EmailClient;
pub use error::{Error, Result};

use tracing_subscriber::EnvFilter;

// Initialize tracing
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();
}
