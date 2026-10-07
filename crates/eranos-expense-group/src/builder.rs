//! Creating groups.

use eranos_core::{ExpenseGroupId, text};
use eranos_money::CurrencyCode;

use crate::error::ExpenseGroupError;
use crate::{ExpenseGroup, Language};

/// Builds a [`ExpenseGroup`]; created by [`ExpenseGroup::builder`].
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct ExpenseGroupBuilder {
    id: Option<ExpenseGroupId>,
    name: String,
    currency: CurrencyCode,
    description: Option<String>,
    language: Option<Language>,
}

impl ExpenseGroupBuilder {
    pub(super) const fn new(name: String, currency: CurrencyCode) -> Self {
        Self {
            id: None,
            name,
            currency,
            description: None,
            language: None,
        }
    }

    /// Sets the identifier, such as the key an app's own store gives the
    /// group, or leaves it to [`build`](Self::build) to generate one with
    /// `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{CurrencyCode, ExpenseGroup, ExpenseGroupId};
    ///
    /// let group = ExpenseGroup::builder("Lisbon", CurrencyCode::EUR)
    ///     .id(ExpenseGroupId::new("lisbon-2026")?)
    ///     .build()?;
    /// assert_eq!(group.id().as_str(), "lisbon-2026");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn id(mut self, id: impl Into<Option<ExpenseGroupId>>) -> Self {
        self.id = id.into();
        self
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

    /// Validates the name and returns the group, under the identifier given
    /// to [`id`](Self::id) or a newly generated one, with no entries and only
    /// the built-in Uncategorised category.
    ///
    /// # Errors
    ///
    /// [`ExpenseGroupError::EmptyName`] when the name is blank.
    pub fn build(self) -> Result<ExpenseGroup, ExpenseGroupError> {
        let name = text::required(self.name, ExpenseGroupError::EmptyName)?;
        Ok(ExpenseGroup::new(
            self.id.unwrap_or_else(ExpenseGroupId::generate),
            name,
            self.currency,
            self.description.and_then(text::optional),
            self.language,
        ))
    }
}
