use axum::{http::StatusCode, routing::get, serve::Serve, Router};
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
    let app = Router::new().route("/health-check", get(health_check));

    Ok(axum::serve(listener, app))
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
