//! Normalization shared by every text field: surrounding whitespace never
//! counts, and an optional field left blank is absent.

use crate::error::{Field, ValueError};

/// Trims `text` and rejects it when nothing is left.
pub(crate) fn required(text: impl Into<String>, field: Field) -> Result<String, ValueError> {
    optional(text).ok_or(ValueError::Empty { field })
}

/// Trims `text`; blank text is `None`.
pub(crate) fn optional(text: impl Into<String>) -> Option<String> {
    let text = text.into();
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else if trimmed.len() == text.len() {
        Some(text)
    } else {
        Some(trimmed.to_owned())
    }
}

#[cfg(test)]
mod tests;
