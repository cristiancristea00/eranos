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
