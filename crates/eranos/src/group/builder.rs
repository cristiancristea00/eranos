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
    id: Option<GroupId>,
    name: String,
    currency: Currency,
    description: Option<String>,
    language: Option<Language>,
}

impl GroupBuilder {
    pub(super) const fn new(name: String, currency: Currency) -> Self {
        Self {
            id: None,
            name,
            currency,
            description: None,
            language: None,
        }
    }

    /// Uses an existing identifier instead of generating one, for a group
    /// that already exists elsewhere.
    pub const fn id(mut self, id: GroupId) -> Self {
        self.id = Some(id);
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

    /// Validates the name and returns the empty group.
    ///
    /// # Errors
    ///
    /// [`ValueError::Empty`] when the name is blank.
    pub fn build(self) -> Result<Group, ValueError> {
        let name = text::required(self.name, Field::GroupName)?;
        Ok(Group::new(
            self.id.unwrap_or_else(GroupId::generate),
            name,
            self.currency,
            self.description.and_then(text::optional),
            self.language,
        ))
    }
}
