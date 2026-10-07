//! The failures of building a payment account or its details.

/// A payment account, or its details, that could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum AccountError {
    /// An account whose name is blank.
    #[error("the account name must not be empty")]
    EmptyName,
    /// An IBAN that failed validation, and why.
    #[error("invalid IBAN: {0}")]
    InvalidIban(IbanError),
    /// A username that is blank.
    #[error("the username must not be empty")]
    EmptyUsername,
}

/// Builds a value from an [`AccountError`], one method per failure, each
/// given what its variant holds and the error's message.
pub trait FromAccountError: Sized {
    /// The value for [`AccountError::EmptyName`].
    fn empty_name(message: String) -> Self;

    /// The value for [`AccountError::InvalidIban`].
    fn invalid_iban(reason: IbanError, message: String) -> Self;

    /// The value for [`AccountError::EmptyUsername`].
    fn empty_username(message: String) -> Self;

    /// The value for `error`, from the method of its variant, given the error's
    /// message.
    #[must_use]
    fn from_account_error(error: &AccountError) -> Self {
        let message = error.to_string();
        match error {
            AccountError::EmptyName => Self::empty_name(message),
            AccountError::InvalidIban(reason) => Self::invalid_iban(*reason, message),
            AccountError::EmptyUsername => Self::empty_username(message),
        }
    }
}

/// Why an IBAN was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum IbanError {
    /// A character other than an ASCII letter, a digit, or whitespace (which
    /// is dropped).
    #[error("only letters, digits, and spaces are allowed")]
    InvalidCharacter,
    /// Not two letters followed by two check digits.
    #[error("expected a two-letter country code followed by two check digits")]
    InvalidFormat,
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
    /// A letter where the country's account number has a digit, or a digit
    /// where it has a letter, by the BBAN structure of the SWIFT IBAN
    /// registry.
    #[error("the account number does not match this country's format")]
    InvalidStructure,
    /// The check digits are 00, 01, or 99, which ISO 13616 never issues, or
    /// the ISO 7064 mod-97 check failed: the number contains a typo.
    #[error("the check digits do not match")]
    InvalidChecksum,
}

/// Builds a value from an [`IbanError`], the reason inside
/// [`AccountError::InvalidIban`], one method per reason.
pub trait FromIbanError: Sized {
    /// The value for [`IbanError::InvalidCharacter`].
    fn invalid_character() -> Self;

    /// The value for [`IbanError::InvalidFormat`].
    fn invalid_format() -> Self;

    /// The value for [`IbanError::UnknownCountry`].
    fn unknown_country() -> Self;

    /// The value for [`IbanError::WrongLength`].
    fn wrong_length(expected: usize, found: usize) -> Self;

    /// The value for [`IbanError::InvalidStructure`].
    fn invalid_structure() -> Self;

    /// The value for [`IbanError::InvalidChecksum`].
    fn invalid_checksum() -> Self;

    /// The value for `error`, from the method of its variant.
    #[must_use]
    fn from_iban_error(error: &IbanError) -> Self {
        match error {
            IbanError::InvalidCharacter => Self::invalid_character(),
            IbanError::InvalidFormat => Self::invalid_format(),
            IbanError::UnknownCountry => Self::unknown_country(),
            IbanError::WrongLength { expected, found } => Self::wrong_length(*expected, *found),
            IbanError::InvalidStructure => Self::invalid_structure(),
            IbanError::InvalidChecksum => Self::invalid_checksum(),
        }
    }
}

#[cfg(test)]
mod tests;
