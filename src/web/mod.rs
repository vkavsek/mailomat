pub mod data;
mod error;
mod log;
mod midware;
mod serve;
mod subscriptions;

use std::sync::Arc;

use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};

pub use error::{Error, Result};
pub use serve::serve;

use crate::AppState;

const REQUEST_ID_HEADER: &str = "x-request-id";

// ###################################
// ->   ROUTES
// ###################################
pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/subscribe", post(subscriptions::api_subscribe))
        .with_state(app_state.clone())
        .route("/health-check", get(health_check))
}

#[tracing::instrument(name = "HEALTHCHECK")]
async fn health_check() -> StatusCode {
    StatusCode::OK
}
