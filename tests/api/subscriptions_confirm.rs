use anyhow::Result;
use reqwest::StatusCode;

use crate::helpers::spawn_test_app;

#[tokio::test]
async fn subscriptions_confirm_without_token_rejected_with_400() -> Result<()> {
    let app = spawn_test_app().await?;

    let res = app
        .http_client
        .get(&format!("http://{}/subscriptions/confirm", app.addr))
        .send()
        .await?;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn subscriptions_confirm_link_returned_by_subscribe_returns_200() -> Result<()> {
    panic!();
    Ok(())
}
