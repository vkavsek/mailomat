use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new()
        .route("/health-check", get(health_check))
        .route("/api/subscribe", post(api_subscribe))
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[derive(Deserialize, Debug)]
struct Subscriber {
    pub name: String,
    pub email: String,
}

async fn api_subscribe(Json(_subscriber): Json<Subscriber>) -> StatusCode {
    // TODO: Do something with subscriber

    StatusCode::OK
}
