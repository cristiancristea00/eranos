//! International Bank Account Numbers (ISO 13616).

use std::fmt;
use std::str::FromStr;

use zeroize::Zeroizing;

use crate::error::{IbanError, ValueError};
use crate::secret::Secret;

mod registry;

/// A validated IBAN, held as a secret.
///
/// Parsing checks the country code against the SWIFT IBAN registry, the
/// length and the account-number format (the BBAN structure) the registry
/// sets for that country, and the ISO 7064 mod-97 check digits, which must
/// lie between 02 and 98 and catch every single-character typo and most
/// transpositions.
/// The number is wiped from memory when dropped; `Debug` and `Display` show
/// only a masked form, and the full number is available through
/// [`expose_secret`](Self::expose_secret).
///
/// # Examples
///
/// ```
/// use eranos::Iban;
///
/// let iban = Iban::parse("de89 3704 0044 0532 0130 00")?;
/// assert_eq!(iban.expose_secret(), "DE89370400440532013000");
/// assert_eq!(iban.country_code(), "DE");
/// assert_eq!(iban.to_string(), "DE89 •••• 3000");
///
/// // One mistyped digit fails the check digits.
/// assert!(Iban::parse("DE89 3704 0044 0532 0130 01").is_err());
/// // A letter O typed for a zero passes them, but not Germany's format.
/// assert!(Iban::parse("DE46 37O4 0044 0532 0130 00").is_err());
/// # Ok::<(), eranos::error::ValueError>(())
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct Iban(Secret);

impl Iban {
    /// Parses an IBAN in electronic (`DE89370400440532013000`) or print
    /// (`DE89 3704 0044 …`) format, in any letter case.
    ///
    /// # Errors
    ///
    /// [`ValueError::InvalidIban`] with the [`IbanError`] that failed.
    pub fn parse(text: &str) -> Result<Self, ValueError> {
        Self::validate(text).map_err(ValueError::InvalidIban)
    }

    fn validate(text: &str) -> Result<Self, IbanError> {
        // Normalised never outgrows the input, so the buffer never
        // reallocates and leaves no unwiped copy behind.
        let mut normalised = Zeroizing::new(String::with_capacity(text.len()));
        for character in text.chars() {
            match character {
                'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    normalised.push(character.to_ascii_uppercase());
                }
                character if character.is_whitespace() => {}
                _ => return Err(IbanError::InvalidCharacter),
            }
        }
        let bytes = normalised.as_bytes();
        let valid_prefix = bytes
            .get(..2)
            .is_some_and(|country| country.iter().all(u8::is_ascii_uppercase))
            && bytes
                .get(2..4)
                .is_some_and(|check| check.iter().all(u8::is_ascii_digit));
        let code = normalised
            .get(..2)
            .filter(|_| valid_prefix)
            .ok_or(IbanError::InvalidFormat)?;
        let country = registry::COUNTRIES
            .binary_search_by(|candidate| candidate.code.cmp(code))
            .ok()
            .and_then(|position| registry::COUNTRIES.get(position))
            .ok_or(IbanError::UnknownCountry)?;
        let expected = usize::from(country.length);
        if bytes.len() != expected {
            return Err(IbanError::WrongLength {
                expected,
                found: bytes.len(),
            });
        }
        if !follows_structure(bytes, country.structure) {
            return Err(IbanError::InvalidStructure);
        }
        // ISO 13616 issues check digits 02 to 98; 00, 01, and 99 pass mod 97
        // whenever 97, 98, and 02 do, so they can only be typos.
        if matches!(bytes.get(2..4), Some(b"00" | b"01" | b"99")) || mod_97(bytes) != 1 {
            return Err(IbanError::InvalidChecksum);
        }
        Ok(Self(Secret::from_zeroising(normalised)))
    }

    /// The two-letter country code, such as `"DE"`.
    #[must_use]
    pub fn country_code(&self) -> &str {
        // Parsing guaranteed at least four ASCII characters.
        self.0.expose().get(..2).unwrap_or_default()
    }

    /// The number with everything but the first and last four characters
    /// hidden, such as `DE89 •••• 3000`; safe to show and log.
    #[must_use]
    pub fn masked(&self) -> String {
        let number = self.0.expose();
        // Parsing guaranteed at least 15 ASCII characters.
        let head = number.get(..4).unwrap_or_default();
        let tail = number
            .get(number.len().saturating_sub(4)..)
            .unwrap_or_default();
        format!("{head} •••• {tail}")
    }

    /// The full number in electronic format: uppercase, without spaces.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        self.0.expose()
    }
}

/// Whether the characters after the four-character prefix follow
/// `structure`, the registry's BBAN structure: segments of a length, `!`, and
/// a class, `n` for digits, `a` for uppercase letters, and `c` for either.
///
/// The length check runs first, and every structure adds up to its country's
/// length minus four, which a unit test checks, so each segment finds its
/// characters and none is left over.
fn follows_structure(number: &[u8], structure: &str) -> bool {
    let mut characters = number.iter().skip(4);
    let mut length = 0;
    for symbol in structure.bytes() {
        let belongs: fn(&u8) -> bool = match symbol {
            digit @ b'0'..=b'9' => {
                length = length * 10 + usize::from(digit - b'0');
                continue;
            }
            b'a' => u8::is_ascii_uppercase,
            b'c' => u8::is_ascii_alphanumeric,
            b'n' => u8::is_ascii_digit,
            // The `!` between a length and its class.
            _ => continue,
        };
        if !characters.by_ref().take(length).all(belongs) {
            return false;
        }
        length = 0;
    }
    true
}

/// The ISO 7064 MOD 97-10 remainder of an uppercase alphanumeric IBAN: the
/// first four characters move to the end, and letters count as 10 (A) to 35
/// (Z).
fn mod_97(bytes: &[u8]) -> u32 {
    // Parsing checked the length first, so there are always four to move.
    let (head, tail) = bytes.split_at_checked(4).unwrap_or((bytes, &[]));
    tail.iter()
        .chain(head)
        .fold(0_u32, |remainder, &byte| match byte {
            b'0'..=b'9' => (remainder * 10 + u32::from(byte - b'0')) % 97,
            _ => (remainder * 100 + u32::from(byte - b'A') + 10) % 97,
        })
}

impl fmt::Debug for Iban {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Iban").field(&self.masked()).finish()
    }
}

impl fmt::Display for Iban {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.masked())
    }
}

impl FromStr for Iban {
    type Err = ValueError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Iban {
    fn serialize<S: serde::Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
        serialiser.serialize_str(self.expose_secret())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Iban {
    fn deserialize<D: serde::Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        crate::serde_text::deserialise(deserialiser, "an IBAN")
    }
}

#[cfg(test)]
mod tests;
