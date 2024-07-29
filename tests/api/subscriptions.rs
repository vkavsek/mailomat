use anyhow::Result;
use reqwest::StatusCode;
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_test_app;

#[tokio::test]
async fn api_subscribe_ok() -> Result<()> {
    let app = spawn_test_app().await?;

    let json_request = json!({
        "name": "John Doe",
        "email": "john.doe@example.com"
    });

    // Setup the mock server
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let res = app.post_subscriptions(&json_request).await?;

    assert_eq!(
        res.status(),
        StatusCode::OK,
        "Wrong response StatusCode: {}",
        res.status()
    );

    let (email, name): (String, String) = sqlx::query_as("SELECT email, name FROM subscriptions")
        .fetch_one(app.mm.db())
        .await?;

    assert_eq!(email, "john.doe@example.com");
    assert_eq!(name, "John Doe");

    Ok(())
}

#[tokio::test]
async fn api_subscribe_unprocessable_entity() -> Result<()> {
    let app = spawn_test_app().await?;

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

    for (json_request, params) in tests {
        let res = app.post_subscriptions(&json_request).await?;
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
async fn api_subscribe_returns_a_400_when_fields_are_present_but_invalid() -> Result<()> {
    let app = spawn_test_app().await?;

    let cases = vec![
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

    for (body, description) in cases {
        let response = app.post_subscriptions(&body).await?;
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 BAD REQUEST the payload was {}.",
            description
        );
    }

    Ok(())
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() -> Result<()> {
    let app = spawn_test_app().await?;
    let body = json!({
        "name": "Ursula",
        "email": "le_guin@gmail.com",
    });

    // Setup the mock server
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let res = app.post_subscriptions(&body).await?;
    assert_eq!(res.status(), StatusCode::OK);

    Ok(())
}
