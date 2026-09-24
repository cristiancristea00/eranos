//! Language tags.

use std::fmt;
use std::str::FromStr;

use icu_locale_core::Locale;

use crate::error::ValueError;

/// A language tag, such as `en`, `ro-RO`, or `zh-Hant-TW`: a Unicode BCP 47
/// locale identifier (UTS #35), the rule the locale APIs of iOS, Android, and
/// the web apply.
///
/// A tag is a language of 2 or 3 letters, then an optional script, region,
/// and variants, then extensions such as `-u-nu-thai`. It is stored in
/// canonical form: each subtag in its canonical case (`en-GB`, `zh-Hant`),
/// with variants and `-u-` keywords sorted; aliases such as `iw` stay as
/// typed. Whether the language exists is up to the apps, which use the tag
/// for their own display language.
///
/// # Examples
///
/// ```
/// use eranos::Language;
///
/// assert_eq!(Language::parse("ro-RO")?.as_str(), "ro-RO");
/// assert_eq!(Language::parse("EN-gb")?.as_str(), "en-GB");
/// assert!(Language::parse("ro_RO").is_err());
/// # Ok::<(), eranos::error::ValueError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Language(String);

impl Language {
    /// Parses a language tag, ignoring surrounding whitespace, and keeps its
    /// canonical form.
    ///
    /// # Errors
    ///
    /// [`ValueError::InvalidLanguage`] when the text is not a Unicode BCP 47
    /// locale identifier.
    pub fn parse(text: &str) -> Result<Self, ValueError> {
        Locale::normalize(text.trim())
            .map(|canonical| Self(canonical.into_owned()))
            .map_err(|_| ValueError::InvalidLanguage)
    }

    /// The tag in canonical form, such as `en-GB`.
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
    fn serialize<S: serde::Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
        serialiser.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Language {
    fn deserialize<D: serde::Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        crate::serde_text::deserialise(deserialiser, "a BCP 47 language tag")
    }
}

#[cfg(test)]
mod tests;
