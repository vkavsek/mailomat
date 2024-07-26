//!*
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::OnceLock,
    time::Duration,
};

use anyhow::Result;
use mailomat::{
    config::get_or_init_config, init_dbg_tracing, model::ModelManager, web::data::ValidEmail,
    AppState, EmailClient,
};
use tokio::net::TcpListener;
use tracing::info;

pub struct TestApp {
    pub addr: SocketAddr,
    pub mm: ModelManager,
}
impl TestApp {
    pub fn new(addr: SocketAddr, mm: ModelManager) -> Self {
        TestApp { addr, mm }
    }
}

/// Trying to bind port 0 will trigger an OS scan for an available port
/// which will then be bound to the application.
const TEST_SOCK_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);

fn _init_test_subscriber() {
    static SUBSCRIBER: OnceLock<()> = OnceLock::new();
    SUBSCRIBER.get_or_init(|| {
        init_dbg_tracing();
    });
}

/// A helper function that tries to spawn a separate thread to serve our app
/// returning the *socket address* on which it is listening.
pub async fn spawn_app() -> Result<TestApp> {
    // _init_test_subscriber();

    let addr = TEST_SOCK_ADDR;
    let config = get_or_init_config();
    let email_addr = ValidEmail::parse(config.email_config.sender_addr.as_str())
        .map_err(Into::<mailomat::web::Error>::into)?;

    let email_client = EmailClient::new(
        config.email_config.url.clone(),
        email_addr,
        config.email_config.auth_token.clone(),
        Duration::from_millis(200),
    )?;
    let mm = ModelManager::test_init().await?;
    let app_state = AppState::new(mm, email_client);

    let listener = TcpListener::bind(&addr).await?;
    let port = listener.local_addr()?.port();
    info!("Listening on {addr}");

    tokio::spawn(mailomat::serve(listener, app_state.clone()));

    let res = TestApp::new(SocketAddr::from((addr.ip(), port)), app_state.mm.clone());
    Ok(res)
}
