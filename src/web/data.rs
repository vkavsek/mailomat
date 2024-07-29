use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;
use validator::ValidateEmail;

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

// ###################################
// ->   IMPLS
// ###################################
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
            return Err(DataParsingError::SubscriberNameForbidden);
        }

        Ok(ValidName(value.to_owned()))
    }
}

// ###################################
// ->   ERROR
// ###################################
#[derive(Debug, Serialize)]
pub enum DataParsingError {
    SubscriberNameEmpty,
    SubscriberNameTooLong,
    SubscriberNameForbidden,

    EmailInvalid,
    EmailTooLong,
}
// Error Boilerplate
impl core::fmt::Display for DataParsingError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
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
