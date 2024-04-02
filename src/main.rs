use std::net::SocketAddr;

use mailer::{config::get_or_init_config, model::ModelManager, Result};

use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // We have a different logging mechanism for production
    #[cfg(not(debug_assertions))]
    {
        mailer::init_production_tracing()
    }
    #[cfg(debug_assertions)]
    {
        mailer::init_dbg_tracing();
    }

    let net_config = &get_or_init_config().net_config;
    let mm = ModelManager::init()?;

    let addr = SocketAddr::from((net_config.host, net_config.app_port));
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on: {addr}");

    mailer::serve(listener, mm).await?;

    Ok(())
}
