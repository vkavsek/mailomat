use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use strum_macros::AsRefStr;

use crate::web::data::ValidEmail;

#[derive(Debug, AsRefStr)]
pub enum MessageStream {
    #[strum(serialize = "broadcast")]
    Broadcast,
    #[strum(serialize = "outbound")]
    Outbound,
}

#[derive(Debug)]
pub struct EmailClient {
    pub http_client: Client,
    pub url: reqwest::Url,
    pub sender: ValidEmail,
    auth_token: SecretString,
}

impl EmailClient {
    pub fn new<S: AsRef<str>>(
        url: S,
        sender: ValidEmail,
        auth_token: SecretString,
        timeout: std::time::Duration,
    ) -> Result<Self> {
        let url =
            reqwest::Url::parse(url.as_ref()).map_err(|e| Error::UrlParsing(e.to_string()))?;

        let http_client = Client::builder().timeout(timeout).build()?;

        Ok(EmailClient {
            http_client,
            url,
            sender,
            auth_token,
        })
    }

    pub async fn send_email<S>(
        &self,
        recepient: &ValidEmail,
        subject: S,
        html_content: S,
        text_content: S,
        message_stream: MessageStream,
    ) -> Result<()>
    where
        S: AsRef<str>,
    {
        let url = self
            .url
            .join("email")
            .map_err(|e| Error::UrlParsing(e.to_string()))?;

        let email_content = EmailContent {
            from: self.sender.as_ref(),
            to: recepient.as_ref(),
            subject: subject.as_ref(),
            html_body: html_content.as_ref(),
            text_body: text_content.as_ref(),
            message_stream: message_stream.as_ref(),
        };

        let _resp = self
            .http_client
            .post(url)
            .header("X-Postmark-Server-Token", self.auth_token.expose_secret())
            .json(&email_content)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EmailContent<'a> {
    pub from: &'a str,
    pub to: &'a str,
    pub subject: &'a str,
    pub html_body: &'a str,
    pub text_body: &'a str,
    pub message_stream: &'a str,
}

// ###################################
// ->   ERROR & RESULT
// ###################################
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, derive_more::From)]
pub enum Error {
    UrlParsing(String),
    #[from]
    Reqwest(reqwest::Error),
}
// Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

// ###################################
// ->   TESTS
// ###################################
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use anyhow::Result;
    use claims::assert_err;
    use fake::{
        faker::{internet::en::SafeEmail, lorem::en::Sentence},
        Fake, Faker,
    };
    use secrecy::Secret;
    use wiremock::{
        matchers::{any, header, header_exists, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let res: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = res {
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
                    && body.get("MessageStream").is_some()
            } else {
                false
            }
        }
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Sentence(1..10).fake()
    }

    fn email() -> Result<ValidEmail> {
        let out = ValidEmail::parse(SafeEmail().fake::<String>())?;
        Ok(out)
    }

    fn email_client(url: String) -> Result<EmailClient> {
        let out = EmailClient::new(
            url,
            email()?,
            Secret::new(Faker.fake()),
            Duration::from_millis(200),
        )?;
        Ok(out)
    }

    #[tokio::test]
    async fn send_email_send_request_success() -> Result<()> {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri())?;

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        email_client
            .send_email(
                &email()?,
                &subject(),
                &content(),
                &content(),
                MessageStream::Outbound,
            )
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn send_email_send_request_fail_if_500() -> Result<()> {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri())?;

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let out = email_client
            .send_email(
                &email()?,
                &subject(),
                &content(),
                &content(),
                MessageStream::Outbound,
            )
            .await;

        assert_err!(out);

        Ok(())
    }

    #[tokio::test]
    async fn send_email_timeout() -> Result<()> {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri())?;

        let response = ResponseTemplate::new(200).set_delay(Duration::from_secs(180));

        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let out = email_client
            .send_email(
                &email()?,
                &subject(),
                &content(),
                &content(),
                MessageStream::Outbound,
            )
            .await;

        assert_err!(out);

        Ok(())
    }
}
