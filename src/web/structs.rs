use unicode_segmentation::UnicodeSegmentation;

use serde::Deserialize;

// ###################################
// ->   ERROR
// ###################################
#[derive(Debug)]
pub enum WebStructParsingError {
    SubscriberNameEmpty,
    SubscriberNameTooLong,
    SubscriberNameForbidden,

    EmailInvalid,
}
// Error Boilerplate
impl core::fmt::Display for WebStructParsingError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for WebStructParsingError {}

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
pub struct ValidSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

pub struct SubscriberEmail(String);

pub struct SubscriberName(String);

// ###################################
// ->   IMPLS
// ###################################
impl TryFrom<DeserSubscriber> for ValidSubscriber {
    type Error = WebStructParsingError;

    fn try_from(deser_sub: DeserSubscriber) -> Result<Self, Self::Error> {
        Ok(ValidSubscriber {
            email: SubscriberEmail::parse(deser_sub.email)?,
            name: SubscriberName::parse(deser_sub.name)?,
        })
    }
}

impl SubscriberEmail {
    pub fn get(&self) -> &str {
        self.0.as_str()
    }
    pub fn parse<S>(value: S) -> Result<Self, WebStructParsingError>
    where
        S: AsRef<str>,
    {
        let valid_email = lazy_regex::regex_is_match!(
            r#"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"#,
            value.as_ref()
        );

        if valid_email {
            Ok(SubscriberEmail(value.as_ref().to_owned()))
        } else {
            Err(WebStructParsingError::EmailInvalid)
        }
    }
}

impl SubscriberName {
    pub fn get(&self) -> &str {
        self.0.as_str()
    }
    pub fn parse<S>(value: S) -> Result<Self, WebStructParsingError>
    where
        S: AsRef<str>,
    {
        let value = value.as_ref();
        if value.trim().is_empty() {
            return Err(WebStructParsingError::SubscriberNameEmpty);
        }
        if value.graphemes(true).count() > 256 {
            return Err(WebStructParsingError::SubscriberNameTooLong);
        }

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        if value.chars().any(|c| forbidden_characters.contains(&c)) {
            return Err(WebStructParsingError::SubscriberNameForbidden);
        }

        Ok(SubscriberName(value.to_owned()))
    }
}
