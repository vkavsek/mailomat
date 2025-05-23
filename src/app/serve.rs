use std::time::Duration;

use axum::{
    body::Body,
    http::{HeaderName, Request, Response},
    middleware, Router,
};
use secrecy::ExposeSecret;
use tower::ServiceBuilder;
use tower_cookies::{cookie, CookieManagerLayer, Key};
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{MakeSpan, OnRequest, OnResponse, TraceLayer},
};
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::RedisStore;
use tracing::Span;

use crate::{config::get_or_init_config, App};

use crate::web::{midware, routes::routes, REQUEST_ID_HEADER};

// ###################################
// ->   ERROR
// ###################################
pub type Result<T> = core::result::Result<T, ServeError>;

#[derive(Debug, thiserror::Error)]
pub enum ServeError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// The core async function returning a future that will serve this application.
///
/// Accepts a `TcpListener` and the `AppState` and sets up a TraceLayer that provides console logging.
///
/// Current implementation might return an IO error from `axum::serve`
// Allow unused vars otherwise the compiler complains because of the cfg macros
#[allow(unused_variables)]
pub async fn serve(app: App) -> Result<()> {
    let App {
        app_state,
        listener,
    } = app;
    let x_request_id: HeaderName = HeaderName::from_static(REQUEST_ID_HEADER);

    let trace_layer = build_trace_layer();

    // TODO: check session settings for security, read all the `with_` methods
    let session_config = &get_or_init_config().session_config;
    let session_store = RedisStore::new(app_state.redis_manager.get_pool());
    let session_man_layer = SessionManagerLayer::new(session_store)
        .with_signed(Key::from(app_state.cookie_secret.expose_secret()))
        .with_secure(session_config.secure)
        .with_expiry(Expiry::OnInactivity(cookie::time::Duration::seconds(
            session_config.expiry_secs,
        )));

    let app = Router::new().merge(routes(app_state.clone())).layer(
        ServiceBuilder::new()
            // Set UUID per request
            .layer(SetRequestIdLayer::new(
                x_request_id.clone(),
                MakeRequestUuid,
            ))
            .layer(trace_layer)
            // cookie manager
            .layer(CookieManagerLayer::new())
            // session manager
            .layer(session_man_layer)
            // This has to be in front of the Propagation layer because while the request goes through
            // middleware as listed in the ServiceBuilder, the response goes through the middleware stack from the bottom up.
            // If we want the response mapper to find the Propagated header that middleware has to run first!
            .layer(middleware::map_response_with_state(
                app_state,
                midware::error_handle_response_mapper,
            ))
            // Propagate UUID to response, keep it last so it processes the response first!
            .layer(PropagateRequestIdLayer::new(x_request_id)),
    );

    axum::serve(listener, app).await?;

    Ok(())
}

/// A helper function that sets up the `tower_http::TraceLayer` - tracing configuration.
fn build_trace_layer() -> TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    impl MakeSpan<Body> + Clone,
    impl OnRequest<Body> + Clone,
    impl OnResponse<Body> + Clone,
> {
    TraceLayer::new_for_http()
        .make_span_with(|req: &Request<Body>| {
            let uuid = req
                .headers()
                .get(REQUEST_ID_HEADER)
                .map(|uuid| uuid.to_str().unwrap_or("").to_string());

            tracing::error_span!(
                "serve",
                id = uuid,
                method = req.method().to_string(),
                path = req.uri().path()
            )
        })
        .on_request(|req: &Request<Body>, _s: &Span| tracing::info!("START @ {}", req.uri()))
        .on_response(|res: &Response<Body>, latency: Duration, _s: &Span| {
            let st_code = res.status().as_u16();

            if (400..=599).contains(&st_code) {
                tracing::error!("END in: {:?} — STATUS: {st_code}", latency)
            } else {
                tracing::info!("END in: {:?} — STATUS: {st_code}", latency)
            }
        })
}
