use axum::{serve::Serve, Router};
use tokio::net::TcpListener;

use crate::{model::ModelManager, routes};
/// SERVE
/// The core function serving this application. Accepts a "TcpListener" and tries to create an App Router,
/// it returns a `Result` containing a `Serve` future. Needs to be awaited like so:
/// ```ignore
/// mailer::serve(listener).await;
/// ```
///
pub fn serve(listener: TcpListener, mm: ModelManager) -> Serve<Router, Router> {
    let app = Router::new().merge(routes::routes(mm));

    axum::serve(listener, app)
}
