pub mod api;

use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};

use crate::AppState;

async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// All the routes of the server
pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .nest("/api", api_routes(app_state.clone()))
        .route("/health-check", get(health_check))
}

/// API - Routes nested under "/api" path
fn api_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/news", post(api::news))
        .with_state(app_state.clone())
        .nest("/subscribe", subscribe_routes(app_state))
}

/// SUBSCRIBE - Routes nested under "/subscribe" path
fn subscribe_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", post(api::subscribe))
        .route("/confirm", get(api::subscribe_confirm))
        .with_state(app_state)
}
