use axum::{http::StatusCode, Json};
use serde::Deserialize;

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[derive(Deserialize, Debug)]
pub struct Subscriber {
    pub name: String,
    pub email: String,
}

pub async fn api_subscribe(Json(_subscriber): Json<Subscriber>) -> StatusCode {
    // TODO: Do something with subscriber

    StatusCode::OK
}
