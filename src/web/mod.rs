pub mod data;
mod error;
mod midware;
mod serve;
mod subscriptions;
mod subscriptions_confirm;

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
pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/api/subscribe", post(subscriptions::api_subscribe))
        .route(
            "/subscriptions/confirm",
            get(subscriptions_confirm::confirm),
        )
        .with_state(app_state.clone())
        .route("/health-check", get(health_check))
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}
