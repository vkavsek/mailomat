use axum::{
    http::StatusCode,
    routing::{get, post},
    serve::Serve,
    Json, Router,
};
use serde::Deserialize;
use tokio::net::TcpListener;

mod error;

use error::Result;

/// SERVE
/// The core function serving this application. Accepts a "TcpListener" and tries to create an App Router,
/// it returns a `Result` containing a `Serve` future. Needs to be awaited like so:
/// ```ignore
/// mailer::serve(listener).unwrap().await;
/// ```
///
/// Currently infallible!
/// TODO: Should it even return a Result ?
pub fn serve(listener: TcpListener) -> Result<Serve<Router, Router>> {
    let app = Router::new()
        .route("/health-check", get(health_check))
        .route("/api/subscribe", post(api_subscribe));

    Ok(axum::serve(listener, app))
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[derive(Deserialize, Debug)]
pub struct Subscriber {
    pub name: Option<String>,
    pub email: Option<String>,
}

pub async fn api_subscribe(Json(subscriber): Json<Subscriber>) -> StatusCode {
    let Subscriber { name, email } = subscriber;

    // TODO: Do something with subscriber

    if name.is_some() && email.is_some() {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    }
}
