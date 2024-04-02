use std::net::SocketAddr;

use mailer::{config::get_or_init_config, model::ModelManager, Result};

use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        // .without_time() // For early dev
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();

    let net_config = &get_or_init_config().net_config;
    let mm = ModelManager::init()?;

    let addr = SocketAddr::from((net_config.host, net_config.app_port));
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on: {addr}");

    mailer::serve(listener, mm).await?;

    Ok(())
}
