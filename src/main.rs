use std::net::SocketAddr;

use mailomat::{config::get_or_init_config, model::ModelManager, Result};

use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // We have a different logging mechanism for production
    #[cfg(not(debug_assertions))]
    {
        mailomat::init_production_tracing()
    }
    #[cfg(debug_assertions)]
    {
        mailomat::init_dbg_tracing();
    }

    // NOTE: Does this spawn_blocking even make sense? Probably not.
    let net_config = tokio::task::spawn_blocking(move || &get_or_init_config().net_config).await?;
    let mm = ModelManager::init().await?;

    let addr = SocketAddr::from((net_config.host, net_config.app_port));
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on: {addr}");

    mailomat::serve(listener, mm).await?;

    Ok(())
}
