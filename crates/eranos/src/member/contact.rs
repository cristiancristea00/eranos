//! Email addresses, checked against RFC 5321 and 5322, and phone numbers,
//! validated for shape only.

use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use icu_properties::CodePointMapData;
use icu_properties::props::{GeneralCategory, GeneralCategoryGroup};

use crate::error::ValueError;

/// An email address: a local part and a domain around an `@`, as RFC 5321
/// and 5322 define them, with the characters outside ASCII that RFC 6531
/// allows.
///
/// The local part is atoms of letters, digits, and the symbols RFC 5322
/// allows, such as `+` and `_`, joined by single dots, or a quoted string; it
/// has at most 64 octets. The domain is labels of letters, digits, and
/// hyphens, with no hyphen at either end and at most 63 octets each, or an IP
/// address in brackets; no dot is required, so `a@b` is an address. The whole
/// address has at most 254 octets, and invisible characters are refused. The
/// address is stored as entered; whether the mailbox exists is up to the
/// apps.
///
/// # Examples
///
/// ```
/// use eranos::Email;
///
/// let email = Email::parse(" ana@example.com ")?;
/// assert_eq!(email.as_str(), "ana@example.com");
/// assert!(Email::parse("ana.example.com").is_err());
/// assert!(Email::parse("ana..popescu@example.com").is_err()); // a doubled dot
/// # Ok::<(), eranos::error::ValueError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    /// Parses an email address, ignoring surrounding whitespace.
    ///
    /// # Errors
    ///
    /// [`ValueError::InvalidEmail`] when the text is not a local part and a
    /// domain around an `@` by the rules above, or holds whitespace or an
    /// invisible character.
    pub fn parse(text: &str) -> Result<Self, ValueError> {
        let text = text.trim();
        if is_address(text) {
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

/// The most octets an address may have: RFC 5321 § 4.5.3.1.3 limits a path,
/// the address with its two angle brackets, to 256.
const MAX_ADDRESS_OCTETS: usize = 254;
/// The most octets of a local part (RFC 5321 § 4.5.3.1.1).
const MAX_LOCAL_PART_OCTETS: usize = 64;
/// The most octets of a domain label (RFC 1035 § 2.3.4).
const MAX_LABEL_OCTETS: usize = 63;
/// The general categories refused in a local part: controls, format
/// characters such as U+200B, surrogates, private use, unassigned code
/// points, and separators.
const REFUSED_IN_LOCAL_PART: GeneralCategoryGroup =
    GeneralCategoryGroup::Other.union(GeneralCategoryGroup::Separator);
/// The general categories a domain label allows: letters, marks, and decimal
/// digits, in any script.
const ALLOWED_IN_LABEL: GeneralCategoryGroup = GeneralCategoryGroup::Letter
    .union(GeneralCategoryGroup::Mark)
    .union(GeneralCategoryGroup::DecimalNumber);

/// Whether `text` is an address: at most 254 octets, no whitespace, and a
/// local part and a domain around the last `@`. A domain holds neither `@`
/// nor `"`, so the last `@` is the separator, and an `@` inside a quoted
/// local part stays with it.
fn is_address(text: &str) -> bool {
    text.len() <= MAX_ADDRESS_OCTETS
        && !text.chars().any(char::is_whitespace)
        && text
            .rsplit_once('@')
            .is_some_and(|(local_part, domain)| is_local_part(local_part) && is_domain(domain))
}

/// Whether `local_part` has at most 64 octets and is a quoted string or a
/// dot-atom.
fn is_local_part(local_part: &str) -> bool {
    local_part.len() <= MAX_LOCAL_PART_OCTETS
        && local_part
            .strip_prefix('"')
            .and_then(|quoted| quoted.strip_suffix('"'))
            .map_or_else(|| is_dot_atom(local_part), is_quoted_content)
}

/// Whether `text` is atoms joined by single dots, with no dot at either end
/// (RFC 5322 § 3.2.3).
fn is_dot_atom(text: &str) -> bool {
    text.split('.')
        .all(|atom| !atom.is_empty() && atom.chars().all(is_atom_character))
}

/// Whether `character` may stand in an atom: RFC 5322 atext, or a printable
/// character outside ASCII (RFC 6531 § 3.3).
fn is_atom_character(character: char) -> bool {
    is_atom_text(character) || is_printable_non_ascii(character)
}

/// Whether `character` is RFC 5322 atext: an ASCII letter, a digit, or one of
/// the symbols RFC 5322 § 3.2.3 lists.
const fn is_atom_text(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || matches!(
            character,
            '!' | '#'
                | '$'
                | '%'
                | '&'
                | '\''
                | '*'
                | '+'
                | '-'
                | '/'
                | '='
                | '?'
                | '^'
                | '_'
                | '`'
                | '{'
                | '|'
                | '}'
                | '~'
        )
}

/// Whether `content`, the inside of a quoted local part, is not empty and
/// holds only qtext and quoted pairs (RFC 5322 § 3.2.4).
fn is_quoted_content(content: &str) -> bool {
    let mut characters = content.chars();
    while let Some(character) = characters.next() {
        // A backslash quotes the next character, which must be printable
        // ASCII.
        let valid = if character == '\\' {
            characters
                .next()
                .is_some_and(|quoted| quoted.is_ascii_graphic())
        } else {
            is_quoted_text(character)
        };
        if !valid {
            return false;
        }
    }
    !content.is_empty()
}

/// Whether `character` is RFC 5322 qtext, printable ASCII other than `"` and
/// the backslash, or a printable character outside ASCII (RFC 6531 § 3.3).
fn is_quoted_text(character: char) -> bool {
    (character.is_ascii_graphic() && !matches!(character, '"' | '\\'))
        || is_printable_non_ascii(character)
}

/// Whether `character` is outside ASCII and visible: not a control, a format
/// character such as U+200B, a surrogate, private use, unassigned, or a
/// separator.
fn is_printable_non_ascii(character: char) -> bool {
    !character.is_ascii() && !REFUSED_IN_LOCAL_PART.contains(general_category(character))
}

/// Whether `domain` is labels joined by dots, or an address literal in
/// brackets.
fn is_domain(domain: &str) -> bool {
    domain
        .strip_prefix('[')
        .and_then(|literal| literal.strip_suffix(']'))
        .map_or_else(|| domain.split('.').all(is_label), is_address_literal)
}

/// Whether `literal`, the inside of the brackets, is an IPv4 address, or the
/// tag `IPv6:` followed by an IPv6 address (RFC 5321 § 4.1.3); the tag ignores
/// letter case (RFC 5234 § 2.3).
fn is_address_literal(literal: &str) -> bool {
    literal.split_once(':').map_or_else(
        || literal.parse::<Ipv4Addr>().is_ok(),
        |(tag, address)| tag.eq_ignore_ascii_case("IPv6") && address.parse::<Ipv6Addr>().is_ok(),
    )
}

/// Whether `label` has 1 to 63 octets of letters, marks, digits, and
/// hyphens, with no hyphen at either end.
fn is_label(label: &str) -> bool {
    (1..=MAX_LABEL_OCTETS).contains(&label.len())
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label.chars().all(is_label_character)
}

/// Whether `character` may stand in a domain label.
fn is_label_character(character: char) -> bool {
    character == '-' || ALLOWED_IN_LABEL.contains(general_category(character))
}

/// The Unicode general category of `character`, from the tables compiled into
/// `icu_properties`.
fn general_category(character: char) -> GeneralCategory {
    CodePointMapData::<GeneralCategory>::new().get(character)
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
        let mut normalised = String::with_capacity(rest.len() + 1);
        normalised.push('+');
        for character in rest.chars() {
            match character {
                '0'..='9' => normalised.push(character),
                ' ' | '-' | '.' | '(' | ')' => {}
                _ => return Err(ValueError::InvalidPhoneNumber),
            }
        }
        let digits = normalised.len() - 1;
        if (7..=15).contains(&digits) {
            Ok(Self(normalised))
        } else {
            Err(ValueError::InvalidPhoneNumber)
        }
    }

    /// The normalised number: `+` followed by digits only.
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
        fn serialize<S: Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
            serialiser.serialize_str(self.as_str())
        }
    }

    impl<'de> Deserialize<'de> for Email {
        fn deserialize<D: Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
            crate::serde_text::deserialise(deserialiser, "an email address")
        }
    }

    impl Serialize for PhoneNumber {
        fn serialize<S: Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
            serialiser.serialize_str(self.as_str())
        }
    }

    impl<'de> Deserialize<'de> for PhoneNumber {
        fn deserialize<D: Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
            crate::serde_text::deserialise(deserialiser, "a phone number such as +40712345678")
        }
    }
}

#[cfg(test)]
mod tests;
