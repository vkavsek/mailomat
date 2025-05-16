//! Contains all the routes that this application can handle.

mod admin;
mod api;
mod home;
mod login;

// re-export errors
pub use admin::AdminError;
pub use api::{
    news::NewsError, subscribe::SubscribeError, subscribe_confirm::SubscribeConfirmError,
};
pub use login::LoginError;

use crate::AppState;
use home::home;
use login::{login_get, login_post};

use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};

async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// All the routes of the server
pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/login", get(login_get).post(login_post))
        .with_state(app_state.clone())
        .nest("/api", api_routes(app_state.clone()))
        .nest("/admin", admin_routes(app_state))
        .route("/health-check", get(health_check))
}

/// API - Routes nested under "/api" path
fn api_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/news", post(api::news_publish))
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

fn admin_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/dashboard", get(admin::dashboard))
        .with_state(app_state)
}
