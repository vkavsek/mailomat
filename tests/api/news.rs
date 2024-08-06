use crate::helpers::TestApp;
use anyhow::Result;
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

#[tokio::test]
async fn api_news_not_delivered_to_unconfirmed_subscribers() -> Result<()> {
    let app = TestApp::spawn().await?;
    app.create_unconfirmed_subscriber().await?;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let res = app.post_api_news().await?;
    assert_eq!(res.status().as_u16(), 200);

    Ok(())
}

#[tokio::test]
async fn api_news_delivered_to_confirmed_subscriber() -> Result<()> {
    let app = TestApp::spawn().await?;
    app.create_confirmed_subscriber().await?;

    Mock::given(path("/email/batch"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        // Will fail if no requests are received
        .expect(1)
        .mount(&app.email_server)
        .await;

    let res = app.post_api_news().await?;
    assert_eq!(res.status().as_u16(), 200);

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
//
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
