use std::net::SocketAddr;

use mailer::{config::get_config, model::ModelManager, Result};

use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .without_time() // For early dev
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = get_config()?;
    let mm = ModelManager::init().await?;

    let addr = SocketAddr::from(([127, 0, 0, 1], config.app_port));
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on: {addr}");

    mailer::serve(listener, mm).await?;

    Ok(())
}
