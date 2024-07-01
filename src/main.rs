use std::net::SocketAddr;

use mailomat::{
    config::get_or_init_config, model::ModelManager, web::data::ValidEmail, AppState, EmailClient,
    Result,
};

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

    // Blocking here probably doesn't matter since we only have the main thread.
    let config = get_or_init_config();
    let email_addr = ValidEmail::parse(config.email_config.email_addr.as_str())
        .map_err(Into::<mailomat::web::Error>::into)?;

    let email_client = EmailClient::new(config.email_config.url.clone(), email_addr);
    let mm = ModelManager::init().await?;

    let app_state = AppState::new(mm, email_client);

    let addr = SocketAddr::from((config.net_config.host, config.net_config.app_port));
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on: {addr}");

    mailomat::serve(listener, app_state.clone()).await?;

    Ok(())
}
