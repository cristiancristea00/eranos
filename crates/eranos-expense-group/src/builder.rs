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
    description: Option<String>,
    language: Option<Language>,
}

impl ExpenseGroupBuilder {
    pub(super) const fn new(id: ExpenseGroupId, name: String, currency: CurrencyCode) -> Self {
        Self {
            id,
            name,
            currency,
            description: None,
            language: None,
        }
    }

    /// Replaces the identifier.
    pub const fn id(mut self, id: ExpenseGroupId) -> Self {
        self.id = id;
        self
    }

    /// Sets the description, trimmed; blank text leaves it absent, as
    /// [`clear_description`](Self::clear_description) does.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Clears the description, which [`description`](Self::description)
    /// sets.
    pub fn clear_description(mut self) -> Self {
        self.description = None;
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
            self.description.and_then(text::optional),
            self.language,
        ))
    }
}
