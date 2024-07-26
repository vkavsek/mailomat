pub mod data;
mod error;
mod log;
mod midware;
mod subscriptions;

use std::{sync::Arc, time::Duration};

use axum::{
    body::Body,
    http::{HeaderName, Request, Response, StatusCode},
    middleware,
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    classify::ServerErrorsFailureClass,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::Span;

pub use error::{Error, Result};

use crate::AppState;

const REQUEST_ID_HEADER: &str = "x-request-id";

// ###################################
// ->   ROUTES
// ###################################
pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/subscribe", post(subscriptions::api_subscribe))
        .with_state(app_state.mm.clone())
        .route("/health-check", get(health_check))
}

#[tracing::instrument(name = "HEALTHCHECK")]
async fn health_check() -> StatusCode {
    StatusCode::OK
}
// ###################################
// ->   SERVE
// ###################################
/// The core async function returning a future that will serve this application.
///
/// Accepts a `TcpListener` and the `AppState` and sets up a TraceLayer that provides console logging.
///
/// Current implementation might return an IO error from `axum::serve`
// Allow unused vars otherwise the compiler complains because of the cfg macros
#[allow(unused_variables)]
pub async fn serve(listener: TcpListener, app_state: Arc<AppState>) -> Result<()> {
    let x_request_id: HeaderName = HeaderName::from_static(REQUEST_ID_HEADER);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|req: &Request<Body>| {
            let uuid = req
                .headers()
                .get(REQUEST_ID_HEADER)
                .map(|uuid| uuid.to_str().unwrap_or("").to_string());

            tracing::info_span!("req", id = uuid)
        })
        .on_response(|res: &Response<Body>, latency: Duration, _s: &Span| {
            let st_code = res.status();
            tracing::info!("END in: {:?} STATUS: {st_code}", latency)
        })
        .on_request(|req: &Request<Body>, _s: &Span| {
            tracing::info!("START: {} @ {}", req.method(), req.uri().path(),)
        })
        .on_failure(
            |err: ServerErrorsFailureClass, latency: Duration, _s: &Span| {
                tracing::error!("ERROR: {err:?} — latency: {:?}", latency)
            },
        );

    let app = Router::new().merge(routes(app_state)).layer(
        ServiceBuilder::new()
            // Set UUID per request
            .layer(SetRequestIdLayer::new(
                x_request_id.clone(),
                MakeRequestUuid,
            ))
            .layer(trace_layer)
            // This has to be in front of the Propagation layer because while the request goes through
            // middleware as listed in the ServiceBuilder, the response goes through the middleware stack from the bottom up.
            // If we want the response mapper to find the Propagated header that middleware has to run first!
            .layer(middleware::map_response(midware::response_mapper))
            // Propagate UUID to response, keep it last so it processes the response first!
            .layer(PropagateRequestIdLayer::new(x_request_id)),
    );

    axum::serve(listener, app).await?;

    Ok(())
}
