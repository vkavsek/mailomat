use anyhow::Result;
use reqwest::StatusCode;
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{ConfirmationLinks, TestApp};

#[tokio::test]
async fn subscriptions_confirm_without_token_rejected_with_400() -> Result<()> {
    let app = TestApp::spawn().await?;

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
    let app = TestApp::spawn().await?;
    let body = json!({
        "name": "John Doe",
        "email": "john.doe@example.com"
    });

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(&body).await?;
    let email_req = &app.email_server.received_requests().await.unwrap()[0];
    let ConfirmationLinks { html, .. } = app.get_confirmation_links(email_req)?;

    let resp = app.http_client.get(html).send().await?;
    assert_eq!(resp.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn subscriptions_confirm_successful_confirmation_of_subscription() -> Result<()> {
    let app = TestApp::spawn().await?;
    let body = json!({
        "name": "John Doe",
        "email": "john.doe@example.com"
    });

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(&body).await?;
    let email_req = &app.email_server.received_requests().await.unwrap()[0];
    let confirm_link = app.get_confirmation_links(email_req)?;

    app.http_client
        .get(confirm_link.html)
        .send()
        .await?
        .error_for_status()?;

    let (email, name, status): (String, String, String) =
        sqlx::query_as("SELECT email, name, status FROM subscriptions")
            .fetch_one(app.mm.db())
            .await?;

    assert_eq!(email, "john.doe@example.com");
    assert_eq!(name, "John Doe");
    assert_eq!(status, "confirmed");

    Ok(())
}
