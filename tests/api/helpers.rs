use std::{net::SocketAddr, sync::OnceLock, time::Duration};

use anyhow::{Context, Result};
use fake::Fake;
use linkify::LinkKind;
use mailomat::{
    config::{get_or_init_config, AppConfig},
    database::DbManager,
    web::{
        auth::password,
        data::{DeserSubscriber, ValidSubscriber},
    },
    App,
};
use reqwest::Client;
use secrecy::SecretString;
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, Connection, PgConnection};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

pub struct ConfirmationLink {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub struct TestApp {
    pub http_client: Client,
    pub addr: SocketAddr,
    pub dm: DbManager,
    pub email_server: MockServer,
    pub test_user: TestUser,
}

#[derive(Clone)]
pub struct TestUser {
    pub username: String,
    pub password: String,
}

impl TestApp {
    /// A helper function that tries to spawn a separate thread to serve our app
    /// returning the *socket address* on which it is listening.
    pub async fn spawn() -> Result<Self> {
        init_test_subscriber();

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

        test_database_create_migrate(&config).await?;

        let app = App::build_from_config(&config).await?;
        let username = Uuid::new_v4().to_string();
        let password = Uuid::new_v4().to_string();
        let password_hash =
            password::hash_new_to_string_async(SecretString::new(password.clone())).await?;

        // Add a test user
        sqlx::query(
            r#"INSERT INTO users (user_id, username, password_hash)
        VALUES ($1, $2, $3)"#,
        )
        .bind(Uuid::new_v4())
        .bind(&username)
        .bind(password_hash)
        .execute(app.app_state.database_mgr.db())
        .await?;

        // Build a TestApp
        let addr = app.listener.local_addr()?;
        let dm = app.app_state.database_mgr.clone();
        let http_client = Client::new();
        let test_user = TestUser { username, password };
        let test_app = TestApp {
            http_client,
            addr,
            dm,
            email_server,
            test_user,
        };

        tokio::spawn(mailomat::serve(app));

        Ok(test_app)
    }

    pub async fn api_subscribe_post(&self, body: &serde_json::Value) -> Result<reqwest::Response> {
        let res = self
            .http_client
            .post(format!("http://{}/api/subscribe", self.addr))
            .json(body)
            .send()
            .await?;

        Ok(res)
    }

    pub async fn api_news_post_unauthorized(&self) -> Result<reqwest::Response> {
        // A sketch of the current newsletter payload structure.
        let newsletter_req_body = json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>",
            }
        });

        let res = self
            .http_client
            .post(&format!("http://{}/api/news", &self.addr))
            .json(&newsletter_req_body)
            .send()
            .await?;

        Ok(res)
    }

    pub async fn api_news_post(&self) -> Result<reqwest::Response> {
        // A sketch of the current newsletter payload structure.
        let newsletter_req_body = json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>",
            }
        });

        let test_user = &self.test_user;
        let res = self
            .http_client
            .post(&format!("http://{}/api/news", &self.addr))
            .basic_auth(test_user.username.clone(), Some(test_user.password.clone()))
            .json(&newsletter_req_body)
            .send()
            .await?;

        Ok(res)
    }

    /// Extract confirmation links embedded in the request to the email API.
    pub fn confirmation_link_get(&self, email_req: &wiremock::Request) -> Result<ConfirmationLink> {
        let body: Value = serde_json::from_slice(&email_req.body)?;

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| l.kind() == &LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);

            let raw_link = links[0].as_str().to_owned();
            let mut confirm_link = reqwest::Url::parse(&raw_link)?;
            // Check that we don''s on the web.
            assert_eq!(confirm_link.host_str(), Some("127.0.0.1"));
            confirm_link.set_port(Some(self.addr.port())).unwrap();
            Ok::<reqwest::Url, anyhow::Error>(confirm_link)
        };

        let html = get_link(body["HtmlBody"].as_str().context("No link in HtmlBody")?)?;
        let plain_text = get_link(body["TextBody"].as_str().context("No link in TextBody")?)?;
        Ok(ConfirmationLink { html, plain_text })
    }

    /// Create new subscriber with: NAME - *John Doe*, EMAIL - *john.doe@example.com*
    /// Returns confirmation links required to confirm this subscriber and the subscriber's info.
    pub async fn subscriber_unconfirmed_create(
        &self,
    ) -> Result<(ConfirmationLink, ValidSubscriber)> {
        let name: String = fake::faker::name::en::Name().fake();
        let email_provider: String = fake::faker::internet::en::FreeEmailProvider().fake();
        let email = name.to_lowercase().replace(" ", "_") + "@" + &email_provider;

        let body = json!({
            "name": name,
            "email": email
        });
        let valid_sub = ValidSubscriber::try_from(DeserSubscriber::new(name, email))?;

        let _mock_guard = Mock::given(path("/email"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .named("Create unconfirmed subscriber")
            .expect(1)
            .mount_as_scoped(&self.email_server)
            .await;

        self.api_subscribe_post(&body).await?.error_for_status()?;
        let email_req = &self
            .email_server
            .received_requests()
            .await
            .expect("Requests should be received")
            .pop()
            .expect("1 request is expected");
        let links = self.confirmation_link_get(email_req)?;

        Ok((links, valid_sub))
    }

    /// Create new subscriber with: NAME - *John Doe*, EMAIL - *john.doe@example.com*
    /// and confirm it. Returns the info of the subscriber that was just added and confirmed.
    pub async fn subscriber_confirmed_create(&self) -> Result<ValidSubscriber> {
        let (links, subscriber) = self.subscriber_unconfirmed_create().await?;
        self.http_client
            .get(links.html)
            .send()
            .await?
            .error_for_status()?;

        Ok(subscriber)
    }
}

fn init_test_subscriber() {
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

/// Create a test database based on AppConfig and migrate it
async fn test_database_create_migrate(config: &AppConfig) -> Result<()> {
    let db_config = &config.db_config;
    let mut connection =
        PgConnection::connect_with(&db_config.connection_options_without_db()).await?;

    let sql = format!(r#"CREATE DATABASE "{}";"#, db_config.db_name.clone());
    sqlx::query(&sql).execute(&mut connection).await?;

    // Create pool only used to migrate the DB
    let db_pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(1000))
        .connect_with(db_config.connection_options())
        .await?;
    // Migrate DB
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    Ok(())
}

pub async fn api_news_post_appless(
    test_user: &TestUser,
    addr: &SocketAddr,
    http_client: Client,
) -> Result<reqwest::Response> {
    // A sketch of the current newsletter payload structure.
    let newsletter_req_body = json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    });

    let res = http_client
        .post(&format!("http://{}/api/news", &addr))
        .basic_auth(test_user.username.clone(), Some(test_user.password.clone()))
        .json(&newsletter_req_body)
        .send()
        .await?;

    Ok(res)
}
