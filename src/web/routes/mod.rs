mod api_news;
mod api_subscribe;
mod subscriptions_confirm;

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
        .nest("/subscriptions", subscriptions_routes(app_state))
        .route("/health-check", get(health_check))
}

/// API - Routes nested under "/api" path
fn api_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/subscribe", post(api_subscribe::subscribe))
        .route("/news", post(api_news::news))
        .with_state(app_state)
}

/// SUBSCRIPTIONS - Routes nested under "/subscriptions" path
fn subscriptions_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/confirm", get(subscriptions_confirm::confirm))
        .with_state(app_state)
}
