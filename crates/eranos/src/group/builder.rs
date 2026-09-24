//! Creating groups.

use crate::error::{Field, ValueError};
use crate::group::{Group, Language};
use crate::id::GroupId;
use crate::money::Currency;
use crate::text;

/// Builds a [`Group`]; created by [`Group::builder`].
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct GroupBuilder {
    currency: Currency,
    name: String,
    description: Option<String>,
    language: Option<Language>,
}

impl GroupBuilder {
    pub(super) const fn new(currency: Currency, name: String) -> Self {
        Self {
            currency,
            name,
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

    /// Validates the name and returns the empty group, under a newly
    /// generated identifier.
    ///
    /// # Errors
    ///
    /// [`ValueError::Empty`] when the name is blank.
    pub fn build(self) -> Result<Group, ValueError> {
        let name = text::required(self.name, Field::GroupName)?;
        Ok(Group::new(
            GroupId::generate(),
            self.currency,
            name,
            self.description.and_then(text::optional),
            self.language,
        ))
    }
}
