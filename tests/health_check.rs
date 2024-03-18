//! Integration tests
//!

use std::{
    future::IntoFuture,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use anyhow::Result;
use reqwest::StatusCode;
use serial_test::serial;
use tokio::net::TcpListener;
use tracing::info;

/// Trying to bind *port 0* will trigger an OS scan for an available port
/// which will then be bound to the application.
const TEST_SOCK_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);

/// Tries to spawn a separate thread to serve our app
/// returning the *socket address* on which it is listening.
async fn spawn_app() -> Result<SocketAddr> {
    let addr = TEST_SOCK_ADDR;

    let listener = TcpListener::bind(&addr).await?;
    let port = listener.local_addr()?.port();
    info!("Listening on {addr}");

    // tokio::spawn takes a Future, since IntoFuture trait didn't exist when tokio went 1.0.
    // That's why we need to call .into_future() here.
    tokio::spawn(mailer::serve(listener)?.into_future());

    Ok(SocketAddr::from((TEST_SOCK_ADDR.ip(), port)))
}

#[serial]
#[tokio::test]
async fn test_healthcheck_ok() -> Result<()> {
    let addr = spawn_app().await?;

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
async fn test_healthcheck_ok_duplicate() -> Result<()> {
    let addr = spawn_app().await?;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{addr}/health-check"))
        .send()
        .await?;

    assert!(res.status() == StatusCode::OK, "Healthcheck FAILED!");

    Ok(())
}
