//! Email addresses, checked against RFC 5321 and 5322, and phone numbers,
//! checked against libphonenumber's numbering plans.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use. The facade does not re-export [`FromContactError`],
//! through which a crate that tells every failure apart converts one without
//! the wildcard arm a `match` outside this crate would need.
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

mod error;

use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use icu_properties::CodePointMapData;
use icu_properties::props::{GeneralCategory, GeneralCategoryGroup};
use rlibphonenumber::{PHONE_NUMBER_UTIL, PhoneNumberFormat};

pub use error::{ContactError, FromContactError};

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
/// # Ok::<(), eranos::error::ContactError>(())
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

/// Prints the type and not the address, which a log must not hold.
impl fmt::Debug for Email {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Email").finish_non_exhaustive()
    }
}

impl Email {
    /// Parses an email address, ignoring surrounding whitespace.
    ///
    /// # Errors
    ///
    /// [`ContactError::InvalidEmail`] when the text is not a local part and a
    /// domain around an `@` by the rules above, or holds whitespace or an
    /// invisible character.
    pub fn parse(text: &str) -> Result<Self, ContactError> {
        let text = text.trim();
        if is_address(text) {
            Ok(Self(text.to_owned()))
        } else {
            Err(ContactError::InvalidEmail)
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
    type Err = ContactError;

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
    MAX_ADDRESS_OCTETS >= text.len()
        && !text.chars().any(char::is_whitespace)
        && text
            .rsplit_once('@')
            .is_some_and(|(local_part, domain)| is_local_part(local_part) && is_domain(domain))
}

/// Whether `local_part` has at most 64 octets and is a quoted string or a
/// dot-atom.
fn is_local_part(local_part: &str) -> bool {
    MAX_LOCAL_PART_OCTETS >= local_part.len()
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
        let valid = if '\\' == character {
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
    '-' == character || ALLOWED_IN_LABEL.contains(general_category(character))
}

/// The Unicode general category of `character`, from the tables compiled into
/// `icu_properties`.
fn general_category(character: char) -> GeneralCategory {
    CodePointMapData::<GeneralCategory>::new().get(character)
}

/// A phone number that is valid in its country's numbering plan, stored in
/// E.164 form: `+`, the country code, and the national number, digits only.
///
/// The number starts with `+`. Separators such as spaces, dashes, dots,
/// slashes, and parentheses are dropped, and so is a national trunk 0 after
/// the country code (`+44 (0)20 …`); letters, extensions, and `tel:` URIs are
/// refused. The plans are libphonenumber's, embedded through the exactly
/// pinned `rlibphonenumber`.
///
/// # Examples
///
/// ```
/// use eranos::PhoneNumber;
///
/// let phone = PhoneNumber::parse("+40 (712) 345-678")?;
/// assert_eq!(phone.as_str(), "+40712345678");
/// assert!(PhoneNumber::parse("0712 345 678").is_err()); // no country code
/// assert!(PhoneNumber::parse("+40 612 345 678").is_err()); // no such range in Romania
/// # Ok::<(), eranos::error::ContactError>(())
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PhoneNumber(String);

/// Prints the type and not the number, which a log must not hold.
impl fmt::Debug for PhoneNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("PhoneNumber").finish_non_exhaustive()
    }
}

impl PhoneNumber {
    /// Parses a phone number, ignoring surrounding whitespace, and stores its
    /// E.164 form.
    ///
    /// # Errors
    ///
    /// [`ContactError::InvalidPhoneNumber`] when the text does not start with
    /// `+`, holds a letter or an extension, or is not a valid number in its
    /// country's numbering plan.
    pub fn parse(text: &str) -> Result<Self, ContactError> {
        let text = text.trim();
        // Letters are refused before libphonenumber sees them: it reads three
        // or more as a vanity number, and would store `+1 800 FLOWERS` as
        // `+18003569377`.
        if !text.starts_with('+') || text.chars().any(char::is_alphabetic) {
            return Err(ContactError::InvalidPhoneNumber);
        }
        // `PHONE_NUMBER_UTIL` decodes the metadata of the pinned
        // rlibphonenumber on first use, and a country's patterns compile the
        // first time one of its numbers is checked; either panics only if that
        // fixed data is corrupt or a pattern does not compile under `lite`.
        // Every test run decodes it, and the test
        // `every_regions_example_numbers_are_accepted` checks every region's
        // patterns, so a failure fails the tests, never an app.
        let plans = &*PHONE_NUMBER_UTIL;
        let number = plans
            .parse(text, None)
            .map_err(|_| ContactError::InvalidPhoneNumber)?;
        // E.164 has no place for an extension, and dropping it would dial
        // another line.
        let has_extension = number
            .extension
            .as_deref()
            .is_some_and(|extension| !extension.is_empty());
        if has_extension || !plans.is_valid_number(&number) {
            return Err(ContactError::InvalidPhoneNumber);
        }
        Ok(Self(
            plans.format(&number, PhoneNumberFormat::E164).into_owned(),
        ))
    }

    /// The number in E.164 form: `+` followed by digits only.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The number grouped the way its country writes it for callers abroad,
    /// such as `+40 712 345 678`, for showing; [`as_str`](Self::as_str) is the
    /// form to dial and store.
    ///
    /// # Panics
    ///
    /// Never: [`parse`](Self::parse) accepted the number, directly or on
    /// load, with the same embedded metadata, so it parses again, and
    /// formatting compiles the country's patterns, which a unit test compiles
    /// for every region's example numbers.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::PhoneNumber;
    ///
    /// let phone = PhoneNumber::parse("+40712345678")?;
    /// assert_eq!(phone.international(), "+40 712 345 678");
    /// # Ok::<(), eranos::error::ContactError>(())
    /// ```
    #[must_use]
    pub fn international(&self) -> String {
        let plans = &*PHONE_NUMBER_UTIL;
        #[expect(
            clippy::expect_used,
            reason = "the stored number came from `parse` with the same pinned metadata, so it parses again"
        )]
        let number = plans.parse(&self.0, None).expect("a stored number parses");
        plans
            .format(&number, PhoneNumberFormat::International)
            .into_owned()
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for PhoneNumber {
    type Err = ContactError;

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
            eranos_core::serde_text::deserialise(deserialiser, "an email address")
        }
    }

    impl Serialize for PhoneNumber {
        fn serialize<S: Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
            serialiser.serialize_str(self.as_str())
        }
    }

    impl<'de> Deserialize<'de> for PhoneNumber {
        fn deserialize<D: Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
            eranos_core::serde_text::deserialise(
                deserialiser,
                "a phone number such as +40712345678",
            )
        }
    }
}

#[cfg(test)]
mod tests;
