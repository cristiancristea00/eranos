//! Creating groups.

use eranos_core::{GroupId, text};
use eranos_money::Currency;

use crate::error::GroupError;
use crate::{Group, Language};

/// Builds a [`Group`]; created by [`Group::builder`].
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct GroupBuilder {
    name: String,
    currency: Currency,
    description: Option<String>,
    language: Option<Language>,
}

impl GroupBuilder {
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
    /// [`GroupError::EmptyName`] when the name is blank.
    pub fn build(self) -> Result<Group, GroupError> {
        let name = text::required(self.name, GroupError::EmptyName)?;
        Ok(Group::new(
            GroupId::generate(),
            name,
            self.currency,
            self.description.and_then(text::optional),
            self.language,
        ))
    }
}
