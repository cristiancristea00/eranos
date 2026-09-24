//! The failure of reading an identifier.

/// An identifier that could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum IdError {
    /// Text that is not a UUID.
    #[error("invalid identifier: expected a UUID")]
    InvalidId,
}
