use std::net::SocketAddr;

use mailomat::{config::get_or_init_config, model::ModelManager, AppState, EmailClient, Result};

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
    let email_addr = config.email_config.valid_sender()?;

    let email_client = EmailClient::new(
        config.email_config.url.clone(),
        email_addr,
        config.email_config.auth_token.clone(),
        config.email_config.timeout(),
    )?;
    let mm = ModelManager::init().await?;

    let app_state = AppState::new(mm, email_client);

    let addr = SocketAddr::from((config.net_config.host, config.net_config.app_port));
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on: {addr}");

    mailomat::serve(listener, app_state).await?;

    Ok(())
}

// fn main() {
//     let em = mailomat::email_client::EmailContent {
//         from: "me",
//         to: "me",
//         subject: "subjet",
//         html_body: "html",
//         text_body: "text",
//         message_stream: "outbound",
//     };
//
//     let js = serde_json::to_string_pretty(&em).unwrap();
//
//     println!("{}", js);
// }
