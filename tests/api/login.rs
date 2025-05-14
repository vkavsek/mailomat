use anyhow::Result;
use mailomat::web::ClientError;

use crate::helpers::{assert_resp_redir_to, TestApp};

#[tokio::test]
async fn api_login_error_messaging_works() -> Result<()> {
    let app = TestApp::spawn().await?;

    // post@login for invalid user - contains cookie that is stored in cookie_store on the
    // http_client and a redirect back to login
    let invalid_login_form = serde_json::json!({
        "username": "invalid",
        "password": "invalid"
    });
    let expected_err_str = ClientError::UsernameOrPasswordInvalid.to_string();
    let resp = app.login_post(invalid_login_form).await?;
    assert_resp_redir_to(&resp, "/login");

    // get@login with the cookie (stored in cookie_store of http_client) - contains error_msg
    let html_page = app.login_get_html().await?;
    assert!(html_page.contains(&expected_err_str));

    // get@login reload with cookie - does not contain error_msg
    let html_page = app.login_get_html().await?;
    assert!(!html_page.contains(&expected_err_str));

    Ok(())
}
