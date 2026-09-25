//! The failure of reading a contact detail.

/// A contact detail that could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum ContactError {
    /// An email address that breaks the RFC 5321 and 5322 rules: no local
    /// part or domain around an `@`, a malformed one, one too long, or
    /// whitespace or an invisible character.
    #[error("invalid email address")]
    InvalidEmail,
    /// A phone number that does not start with `+`, holds a letter or an
    /// extension, or is not valid in its country's numbering plan.
    #[error("invalid phone number: expected `+` and a number valid in its country")]
    InvalidPhoneNumber,
}

/// Builds a value from a [`ContactError`], one method per failure, each
/// given what its variant holds and the error's message.
pub trait FromContactError: Sized {
    /// The value for [`ContactError::InvalidEmail`].
    fn invalid_email(message: String) -> Self;

    /// The value for [`ContactError::InvalidPhoneNumber`].
    fn invalid_phone_number(message: String) -> Self;

    /// The value for `error`, from the method of its variant, given the error's
    /// message.
    #[must_use]
    fn from_contact_error(error: ContactError) -> Self {
        let message = error.to_string();
        match error {
            ContactError::InvalidEmail => Self::invalid_email(message),
            ContactError::InvalidPhoneNumber => Self::invalid_phone_number(message),
        }
    }
}

#[cfg(test)]
mod tests;
