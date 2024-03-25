//! Integration tests
//!

use std::{
    future::IntoFuture,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use anyhow::Result;
use mailer::model::ModelManager;
use reqwest::StatusCode;
use serde_json::json;
use serial_test::serial;
use tokio::net::TcpListener;
use tracing::info;

/// Trying to bind *port 0* will trigger an OS scan for an available port
/// which will then be bound to the application.
const TEST_SOCK_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);

/// A helper function that tries to spawn a separate thread to serve our app
/// returning the *socket address* on which it is listening.
async fn spawn_app() -> Result<(SocketAddr, ModelManager)> {
    let addr = TEST_SOCK_ADDR;
    let mm = ModelManager::test_init().await?;

    let listener = TcpListener::bind(&addr).await?;
    let port = listener.local_addr()?.port();
    info!("Listening on {addr}");

    // tokio::spawn takes a Future, since IntoFuture trait didn't exist when tokio went 1.0
    // we need to call .into_future() here.
    // We could technically await the future that serve() returns inside of on async block, but it's
    // easier to get error handling this way.
    tokio::spawn(mailer::serve(listener, mm.clone()).into_future());

    let res = (SocketAddr::from((TEST_SOCK_ADDR.ip(), port)), mm);
    Ok(res)
}

#[serial]
#[tokio::test]
async fn test_healthcheck_ok() -> Result<()> {
    let (addr, _mm) = spawn_app().await?;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{addr}/health-check"))
        .send()
        .await?;

    assert!(res.status() == StatusCode::OK, "Healthcheck FAILED!");

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_api_subscribe_ok() -> Result<()> {
    let (addr, mm) = spawn_app().await?;

    let client = reqwest::Client::new();

    let json_request = json!({
        "name": "John Doe",
        "email": "john.doe@example.com"
    });

    let res = client
        .post(format!("http://{addr}/api/subscribe"))
        .json(&json_request)
        .send()
        .await?;

    assert_eq!(
        res.status(),
        StatusCode::OK,
        "Wrong response StatusCode: {}",
        res.status()
    );

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(mm.db())
        .await?;

    assert_eq!(saved.email, "john.doe@example.com");
    assert_eq!(saved.name, "John Doe");

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_api_subscribe_unprocessable_entity() -> Result<()> {
    let (addr, _mm) = spawn_app().await?;
    let addr = format!("http://{addr}/api/subscribe");

    let tests = [
        (
            json!({
                "name": "John Doe",
            }),
            "Missing email",
        ),
        (
            json!({
                "name": null,
                "email": "jd@example.com",
            }),
            "Missing name",
        ),
        (json!({}), "Empty json"),
    ];

    let client = reqwest::Client::new();

    for (json_request, params) in tests {
        let res = client.post(&addr).json(&json_request).send().await?;
        assert_eq!(
            res.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "Wrong response: ({}), Expected: ({}); for request with: {params}",
            res.status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    Ok(())
}
