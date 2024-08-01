pub mod config;
pub mod email_client;
mod error;
pub mod model;
pub mod web;

use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::info;

use config::AppConfig;
use model::ModelManager;

pub use email_client::EmailClient;
pub use error::{Error, Result};
pub use web::serve;

use tracing_subscriber::EnvFilter;

// Initialize tracing for DEV
pub fn init_dbg_tracing() {
    tracing_subscriber::fmt()
        .with_target(false)
        .without_time()
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();
}

// Initialize tracing for PRODUCTION
pub fn init_production_tracing() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

// ###################################
// ->  Structs
// ###################################
pub struct App {
    pub app_state: Arc<AppState>,
    pub listener: TcpListener,
}
impl App {
    pub fn new(app_state: Arc<AppState>, listener: TcpListener) -> Self {
        App {
            app_state,
            listener,
        }
    }

    pub async fn build_from_config(config: &AppConfig) -> Result<Self> {
        let email_addr = config.email_config.valid_sender()?;

        let email_client = EmailClient::new(
            config.email_config.url.clone(),
            email_addr,
            config.email_config.auth_token.clone(),
            config.email_config.timeout(),
        )?;
        let mm = ModelManager::init(config).await?;
        let base_url = config.net_config.base_url.clone();
        let app_state = AppState::new(mm, email_client, base_url);

        let addr = SocketAddr::from((config.net_config.host, config.net_config.app_port));
        let listener = TcpListener::bind(addr).await?;
        info!("Listening on: {addr}");

        let app = App::new(app_state, listener);
        Ok(app)
    }
}

pub struct AppState {
    pub mm: ModelManager,
    pub email_client: EmailClient,
    pub base_url: String,
}
impl AppState {
    pub fn new(mm: ModelManager, email_client: EmailClient, base_url: String) -> Arc<Self> {
        Arc::new(AppState {
            mm,
            email_client,
            base_url,
        })
    }
}
