//! Integration tests
//!

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::OnceLock,
};

use anyhow::Result;
use mailomat::{init_dbg_tracing, model::ModelManager};
use reqwest::StatusCode;
use serde_json::json;
use tokio::net::TcpListener;
use tracing::info;

/// Trying to bind *port 0* will trigger an OS scan for an available port
/// which will then be bound to the application.
const TEST_SOCK_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);

fn _init_test_subscriber() {
    static SUBSCRIBER: OnceLock<()> = OnceLock::new();
    SUBSCRIBER.get_or_init(|| {
        init_dbg_tracing();
    });
}
/// A helper function that tries to spawn a separate thread to serve our app
/// returning the *socket address* on which it is listening.
async fn spawn_app() -> Result<(SocketAddr, ModelManager)> {
    // _init_test_subscriber();

    let addr = TEST_SOCK_ADDR;
    let mm = ModelManager::test_init().await?;

    let listener = TcpListener::bind(&addr).await?;
    let port = listener.local_addr()?.port();
    info!("Listening on {addr}");

    tokio::spawn(mailomat::serve(listener, mm.clone()));

    let res = (SocketAddr::from((addr.ip(), port)), mm);
    Ok(res)
}

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

    let (email, name): (String, String) = sqlx::query_as("SELECT email, name FROM subscriptions")
        .fetch_one(mm.db())
        .await?;

    assert_eq!(email, "john.doe@example.com");
    assert_eq!(name, "John Doe");

    Ok(())
}

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
            "Null name",
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

#[tokio::test]
async fn test_api_subscribe_returns_a_400_when_fields_are_present_but_invalid() -> Result<()> {
    let (addr, _mm) = spawn_app().await?;
    let addr = format!("http://{addr}/api/subscribe");

    let test_cases = vec![
        (
            json!({
                "name": "",
                "email": "jd@example.com",
            }),
            "Empty name",
        ),
        (
            json!({
                "name": "John Doe",
                "email": "",
            }),
            "Empty email",
        ),
        (
            json!({
                "name": "John Doe",
                "email": "not an email",
            }),
            "Invalid email",
        ),
    ];

    let client = reqwest::Client::new();
    for (body, description) in test_cases {
        let response = client
            .post(&addr)
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.");
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 BAD REQUEST the payload was {}.",
            description
        );
    }

    Ok(())
}
