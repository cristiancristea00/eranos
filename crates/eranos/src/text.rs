//! Normalisation shared by every text field: surrounding whitespace never
//! counts, and an optional field left blank is absent.

/// Trims `text`, and fails with `error` when nothing is left.
pub(crate) fn required<E>(text: impl Into<String>, error: E) -> Result<String, E> {
    optional(text).ok_or(error)
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
