//! Email addresses and phone numbers, validated for shape only.

use std::fmt;
use std::str::FromStr;

use crate::error::ValueError;

/// An email address: exactly one `@` with text on both sides and no
/// whitespace.
///
/// Only the shape is checked; whether the mailbox exists is up to the apps.
///
/// # Examples
///
/// ```
/// use eranos::Email;
///
/// let email = Email::parse(" ana@example.com ")?;
/// assert_eq!(email.as_str(), "ana@example.com");
/// assert!(Email::parse("ana.example.com").is_err());
/// # Ok::<(), eranos::error::ValueError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    /// Parses an email address, ignoring surrounding whitespace.
    ///
    /// # Errors
    ///
    /// [`ValueError::InvalidEmail`] when the text does not have exactly one
    /// `@` with text on both sides, or contains whitespace.
    pub fn parse(text: &str) -> Result<Self, ValueError> {
        let text = text.trim();
        let valid = match text.split_once('@') {
            Some((local, domain)) => {
                !local.is_empty()
                    && !domain.is_empty()
                    && !domain.contains('@')
                    && !text.chars().any(char::is_whitespace)
            }
            None => false,
        };
        if valid {
            Ok(Self(text.to_owned()))
        } else {
            Err(ValueError::InvalidEmail)
        }
    }

    /// The address as entered, without surrounding whitespace.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for Email {
    type Err = ValueError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

/// A phone number in international (E.164) shape: `+` followed by 7 to 15
/// digits.
///
/// Spaces, dashes, dots, and parentheses between the digits are accepted and
/// dropped, so the stored form is `+` and digits only.
///
/// # Examples
///
/// ```
/// use eranos::PhoneNumber;
///
/// let phone = PhoneNumber::parse("+40 (712) 345-678")?;
/// assert_eq!(phone.as_str(), "+40712345678");
/// assert!(PhoneNumber::parse("0712 345 678").is_err()); // no country code
/// # Ok::<(), eranos::error::ValueError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    /// Parses a phone number, dropping separators.
    ///
    /// # Errors
    ///
    /// [`ValueError::InvalidPhoneNumber`] when the text does not start with
    /// `+`, contains characters other than digits and separators, or has
    /// fewer than 7 or more than 15 digits.
    pub fn parse(text: &str) -> Result<Self, ValueError> {
        let rest = text
            .trim()
            .strip_prefix('+')
            .ok_or(ValueError::InvalidPhoneNumber)?;
        let mut normalized = String::with_capacity(rest.len() + 1);
        normalized.push('+');
        for character in rest.chars() {
            match character {
                '0'..='9' => normalized.push(character),
                ' ' | '-' | '.' | '(' | ')' => {}
                _ => return Err(ValueError::InvalidPhoneNumber),
            }
        }
        let digits = normalized.len() - 1;
        if (7..=15).contains(&digits) {
            Ok(Self(normalized))
        } else {
            Err(ValueError::InvalidPhoneNumber)
        }
    }

    /// The normalized number: `+` followed by digits only.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for PhoneNumber {
    type Err = ValueError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{Email, PhoneNumber};

    impl Serialize for Email {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(self.as_str())
        }
    }

    impl<'de> Deserialize<'de> for Email {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            crate::serde_text::deserialize(deserializer, "an email address")
        }
    }

    impl Serialize for PhoneNumber {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(self.as_str())
        }
    }

    impl<'de> Deserialize<'de> for PhoneNumber {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            crate::serde_text::deserialize(deserializer, "a phone number such as +40712345678")
        }
    }
}

#[cfg(test)]
mod tests;
