mod error;
mod log;
mod midware;
mod routes;

use axum::{middleware, serve::Serve, Router};
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

use crate::model::ModelManager;

pub use error::{Error, Result};

#[derive(Deserialize, Debug)]
pub struct Subscriber {
    pub name: String,
    pub email: String,
}

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
        .layer(
            TraceLayer::new_for_http()
                .on_request(trace::DefaultOnRequest::new().level(Level::INFO))
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(middleware::map_response(midware::response_mapper));

    axum::serve(listener, app)
}
