//! Language tags.

use std::fmt;
use std::str::FromStr;

use crate::error::ValueError;

/// A language tag with the shape of BCP 47, such as `en`, `ro-RO`, or
/// `zh-Hant-TW`.
///
/// Only the shape is checked: a first subtag of 2 to 8 letters, then subtags
/// of 1 to 8 letters or digits, separated by `-`. Whether the language exists
/// is up to the apps, which use the tag for their own display language.
///
/// # Examples
///
/// ```
/// use eranos::Language;
///
/// assert_eq!(Language::parse("ro-RO")?.as_str(), "ro-RO");
/// assert!(Language::parse("ro_RO").is_err());
/// # Ok::<(), eranos::error::ValueError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Language(String);

impl Language {
    /// Parses a language tag, ignoring surrounding whitespace.
    ///
    /// # Errors
    ///
    /// [`ValueError::InvalidLanguage`] when the text does not have the shape
    /// of a BCP 47 tag.
    pub fn parse(text: &str) -> Result<Self, ValueError> {
        let text = text.trim();
        let mut subtags = text.split('-');
        let primary_valid = subtags.next().is_some_and(|primary| {
            (2..=8).contains(&primary.len())
                && primary.bytes().all(|byte| byte.is_ascii_alphabetic())
        });
        let rest_valid = subtags.all(|subtag| {
            (1..=8).contains(&subtag.len())
                && subtag.bytes().all(|byte| byte.is_ascii_alphanumeric())
        });
        if primary_valid && rest_valid {
            Ok(Self(text.to_owned()))
        } else {
            Err(ValueError::InvalidLanguage)
        }
    }

    /// The tag as entered, without surrounding whitespace.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Language {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for Language {
    type Err = ValueError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Language {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Language {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        crate::serde_text::deserialize(deserializer, "a BCP 47 language tag")
    }
}

#[cfg(test)]
mod tests;
