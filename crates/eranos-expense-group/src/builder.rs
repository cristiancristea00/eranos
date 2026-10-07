//! Creating groups.

use eranos_core::{ExpenseGroupId, text};
use eranos_money::Currency;

use crate::error::ExpenseGroupError;
use crate::{ExpenseGroup, Language};

/// Builds a [`ExpenseGroup`]; created by [`ExpenseGroup::builder`].
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct ExpenseGroupBuilder {
    name: String,
    currency: Currency,
    description: Option<String>,
    language: Option<Language>,
}

impl ExpenseGroupBuilder {
    pub(super) const fn new(name: String, currency: Currency) -> Self {
        Self {
            name,
            currency,
            description: None,
            language: None,
        }
    }

    /// Sets the description; blank text clears it.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the language the apps show the group in, or clears it with
    /// `None`.
    pub fn language(mut self, language: impl Into<Option<Language>>) -> Self {
        self.language = language.into();
        self
    }

    /// Validates the name and returns the group, under a newly generated
    /// identifier, with no entries and only the built-in Uncategorised
    /// category.
    ///
    /// # Errors
    ///
    /// [`ExpenseGroupError::EmptyName`] when the name is blank.
    pub fn build(self) -> Result<ExpenseGroup, ExpenseGroupError> {
        let name = text::required(self.name, ExpenseGroupError::EmptyName)?;
        Ok(ExpenseGroup::new(
            ExpenseGroupId::generate(),
            name,
            self.currency,
            self.description.and_then(text::optional),
            self.language,
        ))
    }
}
