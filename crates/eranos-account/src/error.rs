//! The failures of building a payment account or its details.

/// A payment account, or its details, that could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum AccountError {
    /// An account whose name is blank.
    #[error("the account name must not be empty")]
    EmptyName,
    /// A username that is blank.
    #[error("the username must not be empty")]
    EmptyUsername,
    /// An IBAN that failed validation, and why.
    #[error("invalid IBAN: {0}")]
    InvalidIban(IbanError),
}

/// Why an IBAN was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum IbanError {
    /// A character other than an ASCII letter, a digit, or whitespace (which
    /// is dropped).
    #[error("only letters, digits, and spaces are allowed")]
    InvalidCharacter,
    /// The check digits are 00, 01, or 99, which ISO 13616 never issues, or
    /// the ISO 7064 mod-97 check failed: the number contains a typo.
    #[error("the check digits do not match")]
    InvalidChecksum,
    /// Not two letters followed by two check digits.
    #[error("expected a two-letter country code followed by two check digits")]
    InvalidFormat,
    /// A letter where the country's account number has a digit, or a digit
    /// where it has a letter, by the BBAN structure of the SWIFT IBAN
    /// registry.
    #[error("the account number does not match this country's format")]
    InvalidStructure,
    /// A country code without an entry in the SWIFT IBAN registry.
    #[error("the country code is not in the IBAN registry")]
    UnknownCountry,
    /// The wrong number of characters for the country.
    #[error("expected {expected} characters for this country, found {found}")]
    WrongLength {
        /// The registry length for the country.
        expected: usize,
        /// The number of characters after removing spaces.
        found: usize,
    },
}
