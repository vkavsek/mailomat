use std::{net::SocketAddr, sync::OnceLock};

use anyhow::{Context, Result};
use linkify::LinkKind;
use mailomat::{config::get_or_init_config, model::ModelManager, App};
use reqwest::Client;
use serde_json::Value;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;
use wiremock::MockServer;

fn _init_test_subscriber() {
    static SUBSCRIBER: OnceLock<()> = OnceLock::new();
    SUBSCRIBER.get_or_init(|| {
        tracing_subscriber::fmt()
            .without_time()
            .with_target(false)
            .with_env_filter(EnvFilter::from_env("TEST_LOG"))
            .compact()
            .init();
    });
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub struct TestApp {
    pub http_client: Client,
    pub addr: SocketAddr,
    pub mm: ModelManager,
    pub email_server: MockServer,
}
impl TestApp {
    /// A helper function that tries to spawn a separate thread to serve our app
    /// returning the *socket address* on which it is listening.
    pub async fn spawn() -> Result<Self> {
        _init_test_subscriber();

        // A mock server to stand-in for Postmark API
        let email_server = MockServer::start().await;

        let config = {
            let mut c = get_or_init_config().to_owned();
            // A new name for each test
            c.db_config.db_name = Uuid::new_v4().to_string();
            // Trying to bind port 0 will trigger an OS scan for an available port
            // which will then be bound to the application.
            c.net_config.app_port = 0;
            c.email_config.url = email_server.uri();
            c
        };

        // Create and migrate the test DB
        ModelManager::configure_for_test(&config).await?;

        let app = App::build_from_config(&config).await?;

        let addr = app.listener.local_addr()?;
        let mm = app.app_state.model_mgr.clone();
        let http_client = Client::new();

        tokio::spawn(mailomat::serve(app));

        Ok(TestApp {
            http_client,
            addr,
            mm,
            email_server,
        })
    }

    pub async fn post_subscriptions(&self, body: &serde_json::Value) -> Result<reqwest::Response> {
        let res = self
            .http_client
            .post(format!("http://{}/api/subscribe", self.addr))
            .json(body)
            .send()
            .await?;

        Ok(res)
    }

    /// Extract confirmation links embedded in the request to the email API.
    pub fn get_confirmation_links(
        &self,
        email_req: &wiremock::Request,
    ) -> Result<ConfirmationLinks> {
        let body: Value = serde_json::from_slice(&email_req.body)?;

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| l.kind() == &LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);

            let raw_link = links[0].as_str().to_owned();
            let mut confirm_link = reqwest::Url::parse(&raw_link)?;
            // Check that we don't call random API's on the web.
            assert_eq!(confirm_link.host_str(), Some("127.0.0.1"));
            confirm_link.set_port(Some(self.addr.port())).unwrap();
            Ok::<reqwest::Url, anyhow::Error>(confirm_link)
        };

        let html = get_link(body["HtmlBody"].as_str().context("No link in HtmlBody")?)?;
        let plain_text = get_link(body["TextBody"].as_str().context("No link in TextBody")?)?;
        Ok(ConfirmationLinks { html, plain_text })
    }
}
