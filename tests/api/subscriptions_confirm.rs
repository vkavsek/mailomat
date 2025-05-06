use anyhow::Result;
use mailomat::web::types::SubscriptionToken;
use reqwest::{StatusCode, Url};
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{ConfirmationLink, TestApp};

#[tokio::test]
async fn api_subscribe_confirm_without_token_rejected_with_400() -> Result<()> {
    let app = TestApp::spawn().await?;

    let res = app
        .http_client
        .get(format!("http://{}/api/subscribe/confirm", app.addr))
        .send()
        .await?;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn api_subscribe_confirm_link_returned_by_subscribe_returns_200() -> Result<()> {
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

    app.api_subscribe_post(&body).await?;
    let email_req = &app.email_server.received_requests().await.unwrap()[0];
    let ConfirmationLink { html, .. } = app.confirmation_link_get(email_req)?;

    let resp = app.http_client.get(html).send().await?;
    assert_eq!(resp.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn api_subscribe_confirm_successful_confirmation_of_subscription() -> Result<()> {
    let app = TestApp::spawn().await?;
    let valid_sub = app.subscriber_confirmed_create().await?;

    let (email, name, status): (String, String, String) =
        sqlx::query_as("SELECT email, name, status FROM subscriptions")
            .fetch_one(app.dm.db())
            .await?;

    assert_eq!(email, valid_sub.email.as_ref());
    assert_eq!(name, valid_sub.name.as_ref());
    assert_eq!(status, "confirmed");

    Ok(())
}

#[tokio::test]
async fn api_subscribe_confirm_duplicated_confirmation_request_returns_200() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (confirm_link, _) = app.subscriber_unconfirmed_create().await?;

    for _ in 0..2 {
        let res = app
            .http_client
            .get(confirm_link.html.clone())
            .send()
            .await?
            .error_for_status()?;
        assert_eq!(res.status(), StatusCode::OK);
    }

    Ok(())
}

#[tokio::test]
async fn api_subscribe_confirm_correctly_formed_non_existent_token_returns_401() -> Result<()> {
    let app = TestApp::spawn().await?;

    let mut url = Url::parse(&format!("http://{}", app.addr))?;
    url.set_path("api/subscribe/confirm");

    for _ in 0..2 {
        let sub_token = SubscriptionToken::generate();
        url.set_query(Some(&format!("subscription_token={}", *sub_token)));

        let res = app.http_client.get(url.clone()).send().await?;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    Ok(())
}

#[tokio::test]
async fn api_subscribe_confirm_invalid_sub_token_returns_400() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (mut confirm_link, _) = app.subscriber_unconfirmed_create().await?;

    let original_query = confirm_link.html.query().unwrap().to_owned();
    let query_chars_len = original_query.chars().count();
    let ch_queries = ["-test", "{{123}}", "A|b|C", "test~", "čaša1", "test-"];
    for ch_query in ch_queries {
        let modified_query = format!(
            "{}{ch_query}",
            original_query
                .chars()
                .take(query_chars_len - 5)
                .collect::<String>()
        );

        confirm_link.html.set_query(Some(&modified_query));
        let res = app
            .http_client
            .get(confirm_link.html.clone())
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    Ok(())
}
