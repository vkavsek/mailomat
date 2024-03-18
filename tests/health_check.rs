use anyhow::Result;
use reqwest::StatusCode;
use std::future::IntoFuture;
use tokio::net::TcpListener;
use tracing::info;

const TESTING_ADDR: &str = "127.0.0.1:8888";

async fn spawn_app() -> Result<()> {
    let addr = TESTING_ADDR;
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on {addr}");
    // tokio::spawn takes a Future, since IntoFuture trait didn't exist when tokio went 1.0.
    // That's why we need to call .into_future() here.
    tokio::spawn(mailer::serve(listener)?.into_future());
    Ok(())
}

#[tokio::test]
async fn test_healthcheck_ok() -> Result<()> {
    spawn_app().await?;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{TESTING_ADDR}/health-check"))
        .send()
        .await?;

    assert!(res.status() == StatusCode::OK, "Healthcheck FAILED!");

    Ok(())
}
