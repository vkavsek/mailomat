use std::collections::HashMap;

use anyhow::Result;
use mailomat::{utils, web::ClientError};

use crate::helpers::TestApp;

#[tokio::test]
async fn api_login_error_messaging_works() -> Result<()> {
    let app = TestApp::spawn().await?;

    let make_error_pattern = |ce: &str| format!("<p><i>{}</i></p>", ce);

    // post@login for invalid user - error msg
    let mut invalid_login_form = HashMap::new();
    invalid_login_form.insert("username", "invalid");
    invalid_login_form.insert("password", "invalid");
    let res = app
        .http_client
        .post(format!("http://{}/login", app.addr))
        .form(&invalid_login_form)
        .send()
        .await?;
    let res_url = res.url().to_owned();
    let res_body = String::from_utf8(res.bytes().await?.into())?;

    let invalid_user_pass_pattern =
        make_error_pattern(&ClientError::UsernameOrPasswordInvalid.to_string());
    assert!(res_body.contains(&invalid_user_pass_pattern));

    // get@login with injected message - no error msg
    let Some((_, tag)) = res_url.query_pairs().find(|(k, _)| k == "tag") else {
        return Err(anyhow::anyhow!("no tag in the query!"));
    };
    let injected_error_msg = "Injected error message";
    let ie_msg_b64u = utils::b64u_encode(injected_error_msg);
    let injected_error_pattern = make_error_pattern(injected_error_msg);
    let injected_res = app
        .http_client
        .get(format!(
            "http://{}/login?error={}&tag={}",
            app.addr, ie_msg_b64u, tag
        ))
        .send()
        .await?
        .bytes()
        .await?;
    let injected_res_body = String::from_utf8(injected_res.into())?;
    assert!(!injected_res_body.contains(&injected_error_pattern));

    Ok(())
}
