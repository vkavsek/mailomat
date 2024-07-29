//! Tests whether the 'health-check' route returns an appropriate status code

use anyhow::Result;
use reqwest::StatusCode;

use crate::helpers::{spawn_app, TestApp};

#[tokio::test]
async fn test_healthcheck_ok() -> Result<()> {
    let TestApp {
        addr, http_client, ..
    } = spawn_app().await?;

    let res = http_client
        .get(format!("http://{addr}/health-check"))
        .send()
        .await?;

    assert!(res.status() == StatusCode::OK, "Healthcheck FAILED!");

    Ok(())
}
