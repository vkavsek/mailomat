use axum::{
    routing::{get, post},
    serve::Serve,
    Router,
};
use tokio::net::TcpListener;

use crate::{
    routes::{api_subscribe, health_check},
    Result,
};
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
