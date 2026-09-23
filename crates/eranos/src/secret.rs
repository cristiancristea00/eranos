//! Text that is wiped from memory when dropped and never printed.
//!
//! Copies made before the text reached the library (the caller's buffers, or
//! a deserializer's scratch space) are outside its reach.

use std::fmt;

use zeroize::Zeroizing;

/// A string whose buffer is zeroed on drop and whose `Debug` is redacted.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Secret(Zeroizing<String>);

impl Secret {
    /// Takes ownership of `text` without copying it.
    pub(crate) fn new(text: String) -> Self {
        Self(Zeroizing::new(text))
    }

    /// Takes ownership of an already zeroizing buffer.
    pub(crate) const fn from_zeroizing(text: Zeroizing<String>) -> Self {
        Self(text)
    }

    /// The secret text.
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

#[cfg(test)]
mod tests;
