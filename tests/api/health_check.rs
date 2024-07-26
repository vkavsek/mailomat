//! Tests whether the 'health-check' route returns an appropriate status code

use anyhow::Result;
use reqwest::StatusCode;

use crate::helpers::{spawn_app, TestApp};

#[tokio::test]
async fn test_healthcheck_ok() -> Result<()> {
    let TestApp { addr, mm: _ } = spawn_app().await?;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{addr}/health-check"))
        .send()
        .await?;

    assert!(res.status() == StatusCode::OK, "Healthcheck FAILED!");

    Ok(())
}
