pub mod config;
pub mod email_client;
mod error;
pub mod model;
pub mod web;

use std::sync::Arc;

use model::ModelManager;

pub use email_client::EmailClient;
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

// ###################################
// ->   APP STATE
// ###################################
pub struct AppState {
    pub mm: ModelManager,
    pub email_client: EmailClient,
}
impl AppState {
    pub fn new(mm: ModelManager, email_client: EmailClient) -> Arc<Self> {
        Arc::new(AppState { mm, email_client })
    }
}
