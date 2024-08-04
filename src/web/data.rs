use derive_more::{Deref, Display};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;
use validator::ValidateEmail;

use crate::utils;

// ###################################
// ->   STRUCTS
// ###################################
/// Deserializable Subscriber
/// A Subscriber that can be Deserialized but can have invalid fields.
#[derive(Deserialize, Debug)]
pub struct DeserSubscriber {
    pub name: String,
    pub email: String,
}

/// Validated Subscriber
/// A Subscriber with all the fields validated.
#[derive(Debug, Clone)]
pub struct ValidSubscriber {
    pub email: ValidEmail,
    pub name: ValidName,
}

/// Validated Subscriber Email
#[derive(Debug, Clone)]
pub struct ValidEmail(String);

/// Validated Subscriber Name
#[derive(Debug, Clone)]
pub struct ValidName(String);

/// A random 86 character-long case-sensitive Base64-URL encoded subscription token.
#[derive(Deref)]
pub struct SubscriptionToken(String);

// ###################################
// ->   IMPLS
// ###################################
impl SubscriptionToken {
    /// Generates an array of 64 random bytes and encodes it to Base64-URL without padding.
    pub fn generate() -> Self {
        let mut rand_bytes = [0u8; 64];
        thread_rng().fill_bytes(&mut rand_bytes);
        let token = utils::b64u_encode(rand_bytes);

        Self(token)
    }

    pub fn parse<S>(value: S) -> Result<Self, DataParsingError>
    where
        S: AsRef<str>,
    {
        let value = value.as_ref();

        if value.chars().count() != 86 || utils::b64u_decode(value).is_err() {
            return Err(DataParsingError::SubscriberTokenInvalid(value.to_string()));
        }

        Ok(Self(value.to_string()))
    }
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

// ###################################
// ->   ERROR
// ###################################
#[derive(Debug, Serialize, Display)]
pub enum DataParsingError {
    #[display(fmt = "MISSING SUBSCRIBER NAME")]
    SubscriberNameEmpty,
    #[display(fmt = "SUBSCRIBER NAME TOO LONG")]
    SubscriberNameTooLong,
    #[display(fmt = "SUBSCRIBER NAME FORBIDDEN")]
    SubscriberNameForbiddenChars,

    #[display(fmt = "EMAIL INVALID")]
    EmailInvalid,
    #[display(fmt = "EMAIL TOO LONG")]
    EmailTooLong,

    #[display(fmt = "INVALID SUBSCRIBER TOKEN: {}", "_0")]
    SubscriberTokenInvalid(String),
}

impl std::error::Error for DataParsingError {}

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
