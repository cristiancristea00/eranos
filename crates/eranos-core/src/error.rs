//! The failure of reading an identifier.

/// Text that is not an identifier. The message never repeats the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum IdError {
    /// Empty text.
    #[error("an identifier must not be empty")]
    Empty,
    /// Text longer than 64 bytes of UTF-8.
    #[error("an identifier can be at most 64 bytes long")]
    TooLong,
    /// Text that holds a control character, such as a tab or a line break.
    #[error("an identifier cannot contain control characters")]
    ControlCharacter,
    /// Text that starts or ends with whitespace.
    #[error("an identifier cannot start or end with whitespace")]
    SurroundingWhitespace,
}

/// Builds a value from an [`IdError`], one method per failure, each
/// given what its variant holds and the error's message.
pub trait FromIdError: Sized {
    /// The value for [`IdError::Empty`].
    fn empty(message: String) -> Self;

    /// The value for [`IdError::TooLong`].
    fn too_long(message: String) -> Self;

    /// The value for [`IdError::ControlCharacter`].
    fn control_character(message: String) -> Self;

    /// The value for [`IdError::SurroundingWhitespace`].
    fn surrounding_whitespace(message: String) -> Self;

    /// The value for `error`, from the method of its variant, given the error's
    /// message.
    #[must_use]
    fn from_id_error(error: IdError) -> Self {
        let message = error.to_string();
        match error {
            IdError::Empty => Self::empty(message),
            IdError::TooLong => Self::too_long(message),
            IdError::ControlCharacter => Self::control_character(message),
            IdError::SurroundingWhitespace => Self::surrounding_whitespace(message),
        }
    }
}

#[cfg(test)]
mod tests;
