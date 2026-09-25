//! The failure of reading an identifier.

/// An identifier that could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum IdError {
    /// Text that is not a UUID.
    #[error("invalid identifier: expected a UUID")]
    InvalidId,
}

/// Builds a value from an [`IdError`], one method per failure, each
/// given what its variant holds and the error's message.
pub trait FromIdError: Sized {
    /// The value for [`IdError::InvalidId`].
    fn invalid_id(message: String) -> Self;

    /// The value for `error`, from the method of its variant, given the error's
    /// message.
    #[must_use]
    fn from_id_error(error: IdError) -> Self {
        let message = error.to_string();
        match error {
            IdError::InvalidId => Self::invalid_id(message),
        }
    }
}

#[cfg(test)]
mod tests;
