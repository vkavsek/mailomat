use axum::{serve::Serve, Router};
use tokio::net::TcpListener;

use crate::routes;
/// SERVE
/// The core function serving this application. Accepts a "TcpListener" and tries to create an App Router,
/// it returns a `Result` containing a `Serve` future. Needs to be awaited like so:
/// ```ignore
/// mailer::serve(listener).await;
/// ```
///
pub fn serve(listener: TcpListener) -> Serve<Router, Router> {
    let app = Router::new().merge(routes::routes());

    axum::serve(listener, app)
}
