//! Creating groups.

use eranos_core::{ExpenseGroupId, text};
use eranos_money::CurrencyCode;

use crate::error::ExpenseGroupError;
use crate::{ExpenseGroup, Language};

/// Builds a [`ExpenseGroup`]; created by [`ExpenseGroup::builder`] or
/// [`ExpenseGroup::builder_with_id`].
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct ExpenseGroupBuilder {
    id: ExpenseGroupId,
    name: String,
    currency: CurrencyCode,
    note: Option<String>,
    language: Option<Language>,
}

impl ExpenseGroupBuilder {
    pub(super) const fn new(id: ExpenseGroupId, name: String, currency: CurrencyCode) -> Self {
        Self {
            id,
            name,
            currency,
            note: None,
            language: None,
        }
    }

    /// Replaces the identifier.
    pub const fn id(mut self, id: ExpenseGroupId) -> Self {
        self.id = id;
        self
    }

    /// Sets the note, trimmed; blank text leaves it absent, as
    /// [`clear_note`](Self::clear_note) does.
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Clears the note, which [`note`](Self::note)
    /// sets.
    pub fn clear_note(mut self) -> Self {
        self.note = None;
        self
    }

    /// Sets the language the apps show the group in, or clears it with
    /// `None`.
    pub fn language(mut self, language: impl Into<Option<Language>>) -> Self {
        self.language = language.into();
        self
    }

    /// Validates the name and returns the group, under the identifier the
    /// builder holds, with no entries and only the built-in Uncategorised
    /// category.
    ///
    /// # Errors
    ///
    /// [`ExpenseGroupError::EmptyName`] when the name is blank.
    pub fn build(self) -> Result<ExpenseGroup, ExpenseGroupError> {
        let name = text::required(self.name, ExpenseGroupError::EmptyName)?;
        Ok(ExpenseGroup::new(
            self.id,
            name,
            self.currency,
            self.note.and_then(text::optional),
            self.language,
        ))
    }
}
