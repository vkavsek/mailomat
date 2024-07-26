use std::{net::SocketAddr, sync::OnceLock};

use anyhow::Result;
use mailomat::{config::get_or_init_config, init_dbg_tracing, model::ModelManager};
use uuid::Uuid;

pub struct TestApp {
    pub addr: SocketAddr,
    pub mm: ModelManager,
}
impl TestApp {
    pub fn new(addr: SocketAddr, mm: ModelManager) -> Self {
        TestApp { addr, mm }
    }
}

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

    let config = {
        let mut c = get_or_init_config().to_owned();
        c.email_config.timeout_millis = 200;
        // A new name for each test
        c.db_config.db_name = Uuid::new_v4().to_string();
        // Trying to bind port 0 will trigger an OS scan for an available port
        // which will then be bound to the application.
        c.net_config.app_port = 0;
        c
    };

    // Create and migrate the test DB
    ModelManager::configure_for_test(&config).await?;

    let app = mailomat::build(&config).await?;

    let addr = app.listener.local_addr()?;
    let mm = app.app_state.mm.clone();

    tokio::spawn(mailomat::serve(app));

    let res = TestApp::new(SocketAddr::from((addr.ip(), addr.port())), mm);
    Ok(res)
}
