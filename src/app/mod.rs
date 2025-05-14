pub mod serve;

// re-export
pub use serve::serve;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use derive_more::Deref;
use secrecy::{ExposeSecret, SecretSlice};
use tokio::net::TcpListener;
use tracing::info;

use crate::{
    config::AppConfig, database::DbManager, templ_manager::TemplateManager, utils, EmailClient,
    Result,
};

// ###################################
// ->  Structs
// ###################################
pub struct App {
    pub app_state: AppState,
    pub listener: TcpListener,
}
impl App {
    pub fn new(app_state: AppState, listener: TcpListener) -> Self {
        App {
            app_state,
            listener,
        }
    }

    pub async fn build_from_config(config: AppConfig) -> Result<Self> {
        let email_addr = config.email_config.valid_sender()?;

        let dm = DbManager::init(&config).await?;
        let tm = TemplateManager::init();
        let email_timeout = config.email_config.timeout();
        let email_client = EmailClient::new(
            &config.email_config.url,
            email_addr,
            config.email_config.auth_token,
            email_timeout,
        )?;
        let cookie_secret = SecretSlice::from(
            utils::b64_decode(config.net_config.cookie_secret_b64enc.expose_secret())
                .context("config: failed to decode cookie secret from base64")?,
        );

        let app_state = AppState::new(
            dm,
            tm,
            email_client,
            config.net_config.base_url,
            cookie_secret,
        );

        let addr = SocketAddr::from((config.net_config.host, config.net_config.app_port));
        let listener = TcpListener::bind(addr).await?;
        let addr = listener.local_addr()?;
        info!("{:<20} - {}", "Listening on:", addr);

        let app = App::new(app_state, listener);
        Ok(app)
    }
}

pub struct InternalState {
    pub database_mgr: DbManager,
    pub templ_mgr: TemplateManager,
    pub email_client: EmailClient,
    pub base_url: String,
    pub cookie_secret: SecretSlice<u8>,
}

/// Application state containing all global data.
/// It implements `Deref` to easily access the fields on `InternalState`
/// Uses an `Arc` so it can be cloned around.
#[derive(Clone, Deref)]
pub struct AppState(Arc<InternalState>);

impl AppState {
    pub fn new(
        database_mgr: DbManager,
        templ_mgr: TemplateManager,
        email_client: EmailClient,
        base_url: String,
        cookie_secret: SecretSlice<u8>,
    ) -> Self {
        AppState(Arc::new(InternalState {
            templ_mgr,
            database_mgr,
            email_client,
            base_url,
            cookie_secret,
        }))
    }
}
