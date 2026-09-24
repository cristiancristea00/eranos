//! Expense categories, defined per group.

use crate::error::{Field, ValueError};
use crate::text;

/// A category a group sorts its expenses into, such as "Food" or "Fuel".
///
/// Every group defines its own categories; names are unique within a group,
/// compared case-insensitively.
///
/// # Examples
///
/// ```
/// use eranos::Category;
///
/// let food = Category::new("  Food ")?;
/// assert_eq!(food.name(), "Food");
/// assert!(Category::new(" ").is_err());
/// # Ok::<(), eranos::error::ValueError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "CategoryRecord")
)]
pub struct Category {
    name: String,
}

impl Category {
    /// Returns a category named `name`, trimmed.
    ///
    /// # Errors
    ///
    /// [`ValueError::Empty`] when the name is blank.
    pub fn new(name: impl Into<String>) -> Result<Self, ValueError> {
        Ok(Self {
            name: text::required(name, Field::CategoryName)?,
        })
    }

    /// The name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The name folded for case-insensitive comparison.
    pub(crate) fn folded_name(&self) -> String {
        self.name.to_lowercase()
    }
}

/// The serialised form of a [`Category`], validated on the way in.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct CategoryRecord {
    name: String,
}

#[cfg(feature = "serde")]
impl TryFrom<CategoryRecord> for Category {
    type Error = ValueError;

    fn try_from(record: CategoryRecord) -> Result<Self, Self::Error> {
        Self::new(record.name)
    }
}

#[cfg(test)]
mod tests;
