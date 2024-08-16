use crate::helpers::{api_news_post_appless, TestApp};
use anyhow::Result;
use uuid::Uuid;
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

#[tokio::test]
async fn api_news_not_delivered_to_unconfirmed_subscribers() -> Result<()> {
    let app = TestApp::spawn().await?;
    app.subscriber_unconfirmed_create().await?;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let res = app.api_news_post().await?;
    assert_eq!(res.status().as_u16(), 200);

    Ok(())
}

#[tokio::test]
async fn api_news_subscribers_with_invalid_emails_dont_get_news() -> Result<()> {
    use chrono::Utc;
    use uuid::Uuid;

    let app = TestApp::spawn().await?;
    let subscriber_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
    "#,
    )
    .bind(subscriber_id)
    .bind("invalid_email")
    .bind("Invalid Email")
    .bind(Utc::now())
    .execute(app.dm.db())
    .await?;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let resp = app.api_news_post().await?.error_for_status()?;
    assert_eq!(resp.status().as_u16(), 200);

    Ok(())
}

#[tokio::test]
async fn api_news_delivered_to_confirmed_subscriber() -> Result<()> {
    let app = TestApp::spawn().await?;
    app.subscriber_confirmed_create().await?;

    Mock::given(path("/email/batch"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        // Will fail if no requests are received
        .expect(1)
        .mount(&app.email_server)
        .await;

    let res = app.api_news_post().await?;
    assert_eq!(res.status().as_u16(), 200);

    Ok(())
}

#[tokio::test]
async fn api_news_delivered_to_confirmed_subscriber_without_blocking() -> Result<()> {
    let app = TestApp::spawn().await?;
    app.subscriber_confirmed_create().await?;

    Mock::given(path("/email/batch"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        // Will fail if no requests are received
        .expect(100)
        .mount(&app.email_server)
        .await;

    let mut set = tokio::task::JoinSet::new();
    for _ in 0..100 {
        let test_user = app.test_user.clone();
        let addr = app.addr;

        let http_client = app.http_client.clone();
        set.spawn(async move { api_news_post_appless(&test_user, &addr, http_client).await });
    }

    while let Some(res) = set.join_next().await {
        let res = res??;
        assert_eq!(res.status().as_u16(), 200);
    }

    Ok(())
}

#[tokio::test]
async fn api_news_invalid_data_422() -> Result<()> {
    let app = TestApp::spawn().await?;
    let test_cases = [
        (
            serde_json::json!({
            "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
            }
            }),
            "missing title",
        ),
        (
            serde_json::json!({"title": "Newsletter!"}),
            "missing content",
        ),
    ];

    for (inv_body, err_msg) in test_cases {
        let resp = app
            .http_client
            .post(&format!("http://{}/api/news", &app.addr))
            .json(&inv_body)
            .send()
            .await?;
        assert_eq!(
            resp.status().as_u16(),
            422,
            "api didn't return status code 422 - unprocessable entity, when invalid body was: {}",
            err_msg
        )
    }

    Ok(())
}

#[tokio::test]
async fn api_news_requests_missing_authorization_are_rejected() -> Result<()> {
    let app = TestApp::spawn().await?;
    let resp = app.api_news_post_unauthorized().await?;
    assert_eq!(resp.status().as_u16(), 401);
    assert_eq!(
        resp.headers()["WWW-Authenticate"],
        r#"Basic realm="publish""#
    );
    Ok(())
}

#[tokio::test]
async fn api_news_non_existing_user_rejected() -> Result<()> {
    let app = TestApp::spawn().await?;
    let username = Uuid::new_v4();
    let password = Uuid::new_v4();

    let response = app
        .http_client
        .post(&format!("http://{}/api/news", &app.addr))
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({
            "title": "Title",
            "content": {
                "text": "Hello",
                "html": "<p>hello</p>",
            }
        }))
        .send()
        .await?;

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        response.headers()["WWW-Authenticate"],
        r#"Basic realm="publish""#
    );

    Ok(())
}

#[tokio::test]
async fn api_news_invalid_password_rejected() -> Result<()> {
    let app = TestApp::spawn().await?;
    let password = Uuid::new_v4();

    let response = app
        .http_client
        .post(&format!("http://{}/api/news", &app.addr))
        .basic_auth(app.test_user.username, Some(password))
        .json(&serde_json::json!({
            "title": "Title",
            "content": {
                "text": "Hello",
                "html": "<p>hello</p>",
            }
        }))
        .send()
        .await?;

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        response.headers()["WWW-Authenticate"],
        r#"Basic realm="publish""#
    );

    Ok(())
}

// #[tokio::test]
// async fn api_news_delivered_to_all_confirmed_subscribers() -> Result<()> {
//     let app = TestApp::spawn().await?;
//     app.create_confirmed_subscriber().await?;
//
//     Mock::given(path("/email/batch"))
//         .and(method("POST"))
//         .respond_with(ResponseTemplate::new(200))
//         // Will fail if no requests are received
//         .expect(1)
//         .mount(&app.email_server)
//         .await;
//
//     let res = app.post_api_news().await?;
//     assert_eq!(res.status().as_u16(), 200);
//     todo!();
//
//     Ok(())
// }
