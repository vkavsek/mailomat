//! Tests whether the 'health-check' route returns an appropriate status code

use anyhow::Result;
use reqwest::StatusCode;

use crate::helpers::TestApp;

#[tokio::test]
async fn healthcheck_ok() -> Result<()> {
    let TestApp {
        addr, http_client, ..
    } = TestApp::spawn().await?;

    let res = http_client
        .get(format!("http://{addr}/health-check"))
        .send()
        .await?;

    assert!(res.status() == StatusCode::OK, "Healthcheck FAILED!");

    Ok(())
}

#[tokio::test]
async fn invalid_path_404() -> Result<()> {
    let TestApp {
        addr, http_client, ..
    } = TestApp::spawn().await?;

    let res = http_client
        .get(format!("http://{addr}/invalidpath"))
        .send()
        .await?;

    assert!(
        res.status() == StatusCode::NOT_FOUND,
        "Invalid Path check FAILED!, expected: {}, got: {}",
        404,
        res.status().as_u16()
    );

    Ok(())
}
