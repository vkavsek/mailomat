mod error;
mod log;
mod midware;
mod routes;

use axum::{middleware, serve::Serve, Router};
use tokio::net::TcpListener;

use crate::model::ModelManager;

pub use error::{Error, Result};

/// SERVE
/// The core sync function returning a future that will serve this application.
///
/// Accepts a "TcpListener" and the state(ModelManager) and creates an App Router.
/// It returns a `Result` containing a `Serve` future. Needs to be awaited like so:
/// ```ignore
/// mailer::serve(listener).await;
/// ```
pub fn serve(listener: TcpListener, mm: ModelManager) -> Serve<Router, Router> {
    let app = Router::new()
        .merge(routes::routes(mm))
        .layer(middleware::map_response(midware::response_mapper));

    axum::serve(listener, app)
}
