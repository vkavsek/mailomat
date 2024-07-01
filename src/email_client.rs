use reqwest::Client;

use crate::web::data::ValidEmail;

#[derive(Debug)]
pub struct EmailClient {
    pub http_client: Client,
    pub url: String,
    pub sender: ValidEmail,
}

impl EmailClient {
    pub fn new(url: String, sender: ValidEmail) -> Self {
        EmailClient {
            http_client: Client::new(),
            url,
            sender,
        }
    }
    pub async fn send_email<S>(
        &self,
        recepient: &ValidEmail,
        subject: S,
        html_content: S,
        text_content: S,
    ) -> Result<()>
    where
        S: AsRef<str>,
    {
        let (subject, html_content, text_content) = (
            subject.as_ref(),
            html_content.as_ref(),
            text_content.as_ref(),
        );

        todo!();
    }
}

// ###################################
// ->   ERROR & RESULT
// ###################################
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // TODO:
}
// Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
