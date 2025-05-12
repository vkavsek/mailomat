//! Most of the structs in `web` module and their implementations live here.
//! Includes structs that need to be validated, their parsing implementations and tests for those

use anyhow::Context;
use derive_more::Deref;
use hmac::{Hmac, Mac};
use rand::{rng, RngCore};
use secrecy::ExposeSecret;
use serde::Deserialize;
use sha2::Sha256;
use unicode_segmentation::UnicodeSegmentation;
use validator::ValidateEmail;

use crate::{utils, AppState};

// ###################################
// ->   STRUCTS
// ###################################
/// A deserializable struct that contains the data of the newsletter to be sent to the subscribers
#[derive(Debug, Deserialize)]
pub struct News {
    pub title: String,
    pub content: NewsContent,
}
/// A deserializable struct that contains the content of the newsletter to be sent to the subscribers
#[derive(Debug, Deserialize)]
pub struct NewsContent {
    pub text: String,
    pub html: String,
}

/// Deserializable Subscriber
/// A Subscriber that can be Deserialized but can have invalid fields
#[derive(Debug, Deserialize)]
pub struct DeserSubscriber {
    pub name: String,
    pub email: String,
}

impl DeserSubscriber {
    pub fn new(name: String, email: String) -> Self {
        Self { name, email }
    }
}

/// Validated Subscriber
/// A Subscriber with all the fields validated
#[derive(Debug, Clone)]
pub struct ValidSubscriber {
    pub email: ValidEmail,
    pub name: ValidName,
}

impl TryFrom<DeserSubscriber> for ValidSubscriber {
    type Error = DataParsingError;

    fn try_from(deser_sub: DeserSubscriber) -> Result<Self, Self::Error> {
        Ok(ValidSubscriber {
            email: ValidEmail::parse(deser_sub.email)?,
            name: ValidName::parse(deser_sub.name)?,
        })
    }
}

/// Validated Subscriber Email
#[derive(Debug, Clone)]
pub struct ValidEmail(String);

impl AsRef<str> for ValidEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ValidEmail {
    pub fn parse<S>(value: S) -> Result<Self, DataParsingError>
    where
        S: AsRef<str>,
    {
        let value = value.as_ref();

        if value.graphemes(true).count() > 256 {
            return Err(DataParsingError::EmailTooLong);
        }

        if value.validate_email() {
            Ok(ValidEmail(value.to_owned()))
        } else {
            Err(DataParsingError::EmailInvalid)
        }
    }
}

/// Validated Subscriber Name
#[derive(Debug, Clone)]
pub struct ValidName(String);

impl AsRef<str> for ValidName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ValidName {
    pub fn parse<S>(value: S) -> Result<Self, DataParsingError>
    where
        S: AsRef<str>,
    {
        let value = value.as_ref();
        if value.graphemes(true).count() > 256 {
            return Err(DataParsingError::SubscriberNameTooLong);
        }

        if value.trim().is_empty() {
            return Err(DataParsingError::SubscriberNameEmpty);
        }

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        if value.chars().any(|c| forbidden_characters.contains(&c)) {
            return Err(DataParsingError::SubscriberNameForbiddenChars);
        }

        Ok(ValidName(value.to_owned()))
    }
}

/// A random 86 character-long case-sensitive Base64-URL encoded subscription token
#[derive(Debug, Deref)]
pub struct SubscriptionToken(String);

impl SubscriptionToken {
    /// Generates an array of 64 random bytes and encodes it to Base64-URL without padding
    pub fn generate() -> Self {
        let mut rand_bytes = [0u8; 64];
        rng().fill_bytes(&mut rand_bytes);
        let token = utils::b64u_encode(rand_bytes);

        Self(token)
    }

    pub fn parse<S>(value: S) -> Result<Self, DataParsingError>
    where
        S: AsRef<str>,
    {
        let value = value.as_ref();

        let decoded = utils::b64u_decode(value);
        if decoded.is_err() || decoded.is_ok_and(|v| v.len() != 64) {
            return Err(DataParsingError::SubscriberTokenInvalid(value.to_string()));
        }

        Ok(Self(value.to_string()))
    }
}

/// A deserializable struct that contains the `subscription_token` to be deserialized from the query
#[derive(Debug, Deserialize, Deref)]
pub struct SubscribeConfirmQuery {
    pub subscription_token: String,
}

/// A deserializable struct that contains the BASE64-url encoded string representation of `ClientError`
/// and an HMAC tag to authenticate the error message to be deserialized from the query on the login page
/// in zero-padded hexadecimal representation
#[derive(Debug, Deserialize)]
pub struct LoginQueryParams {
    /// BASE64-url encoded error
    pub error: String,
    /// Zero-padded lowercase Hexadecimal encoded HMAC tag
    pub tag: String,
}

impl LoginQueryParams {
    /// Try to verify that the received HMAC tag matches the HMAC tag computed from the received user error string.
    /// If the tags don't match the method returns a `DataParsingError`, otherwise it returns the received user error string.
    pub fn verify(self, app_state: &AppState) -> Result<String, DataParsingError> {
        let received_hmac_tag = utils::hex_decode(self.tag).map_err(DataParsingError::Utils)?;
        let received_error =
            utils::b64u_decode_to_string(&self.error).map_err(DataParsingError::Utils)?;

        let mut mac = Hmac::<Sha256>::new_from_slice(app_state.hmac_secret.expose_secret())
            .context("deserializing login query: failed to create HMAC from hmac_secret")?;
        mac.update(self.error.as_bytes());
        mac.verify_slice(&received_hmac_tag)
            .map_err(|_| DataParsingError::LoginHmacTagInvalid(received_error.clone()))?;

        Ok(received_error)
    }
}

// ###################################
// ->   ERROR
// ###################################
#[derive(Debug, thiserror::Error)]
pub enum DataParsingError {
    #[error("missing subscriber name")]
    SubscriberNameEmpty,
    #[error("subscriber name too long")]
    SubscriberNameTooLong,
    #[error("subscriber name forbidden")]
    SubscriberNameForbiddenChars,

    #[error("email invalid")]
    EmailInvalid,
    #[error("email too long")]
    EmailTooLong,

    #[error("invalid subscriber token: {0}")]
    SubscriberTokenInvalid(String),

    #[error("received hmac tag does not match the hmac tag computed from the received error: {0}")]
    LoginHmacTagInvalid(String),

    #[error("utils error: {0}")]
    Utils(#[from] utils::UtilsError),

    #[error("unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

// ###################################
// ->   TESTS
// ###################################
#[cfg(test)]
mod test {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn subscription_token_is_86_chars_long() {
        for _ in 0..100 {
            let st = SubscriptionToken::generate();
            assert_eq!(st.len(), 86)
        }
    }

    #[test]
    fn name_a_256_grapheme_long_name_is_valid() {
        let name = "ё".repeat(256);
        assert_ok!(ValidName::parse(name));
    }
    #[test]
    fn name_longer_than_256_rejected() {
        let name = "a".repeat(257);
        assert_err!(ValidName::parse(name));
    }
    #[test]
    fn name_whitespace_only_rejected() {
        let name = " ".to_string();
        assert_err!(ValidName::parse(name));
    }
    #[test]
    fn name_empty_string_rejected() {
        let name = "".to_string();
        assert_err!(ValidName::parse(name));
    }
    #[test]
    fn name_containing_invalid_character_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(ValidName::parse(name));
        }
    }
    #[test]
    fn name_a_valid_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert_ok!(ValidName::parse(name));
    }

    #[test]
    fn email_empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(ValidEmail::parse(email));
    }
    #[test]
    fn email_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(ValidName::parse(name));
    }
    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(ValidEmail::parse(email));
    }
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(ValidEmail::parse(email));
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email: String = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    /// A quickcheck test that generates random valid emails and tests them.
    /// Random generation is based on `Arbitrary` implementation above
    #[quickcheck_macros::quickcheck]
    fn email_valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        dbg!(&valid_email.0);
        ValidEmail::parse(valid_email.0).is_ok()
    }
}
