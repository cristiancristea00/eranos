//! Expense categories, defined per group.

use icu_casemap::CaseMapper;
use icu_normalizer::DecomposingNormalizerBorrowed;

use crate::error::CategoryError;
use crate::id::CategoryId;
use crate::text;

/// A category a group sorts its expenses into, such as "Food" or "Fuel".
///
/// Every group defines its own categories; names are unique within a group
/// under Unicode canonical caseless matching, so `Food` and `FOOD`, `Straße`
/// and `STRASSE`, and a composed and a decomposed `Café` are one name, and so
/// are `Maße` and `Masse`. A category gets its identifier when it is
/// created, and [`renamed`](Self::renamed) keeps it. Every group also has the
/// built-in "Uncategorised" category, [`CategoryId::UNCATEGORISED`], which
/// cannot be renamed or removed.
///
/// # Examples
///
/// ```
/// use eranos::Category;
///
/// let food = Category::new("  Food ")?;
/// assert_eq!(food.name(), "Food");
/// assert!(Category::new(" ").is_err());
///
/// let groceries = food.renamed("Groceries")?;
/// assert_eq!(groceries.id(), food.id());
/// # Ok::<(), eranos::error::CategoryError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "CategoryRecord")
)]
pub struct Category {
    id: CategoryId,
    name: String,
}

impl Category {
    /// Returns a category named `name`, trimmed, under a newly generated
    /// identifier.
    ///
    /// # Errors
    ///
    /// [`CategoryError::EmptyName`] when the name is blank.
    pub fn new(name: impl Into<String>) -> Result<Self, CategoryError> {
        let name = text::required(name, CategoryError::EmptyName)?;
        Ok(Self {
            id: CategoryId::generate(),
            name,
        })
    }

    /// The built-in category, under [`CategoryId::UNCATEGORISED`].
    pub(crate) fn uncategorised() -> Self {
        Self {
            id: CategoryId::UNCATEGORISED,
            name: "Uncategorised".to_owned(),
        }
    }

    /// The same category under the name `name`, trimmed; the identifier is
    /// kept.
    ///
    /// # Errors
    ///
    /// [`CategoryError::EmptyName`] when the name is blank.
    pub fn renamed(&self, name: impl Into<String>) -> Result<Self, CategoryError> {
        Ok(Self {
            id: self.id,
            name: text::required(name, CategoryError::EmptyName)?,
        })
    }

    /// The identifier.
    #[must_use]
    pub const fn id(&self) -> CategoryId {
        self.id
    }

    /// The name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The name's canonical caseless form, on which uniqueness is checked:
    /// Unicode's D145, `NFD(toCasefold(NFD(name)))`.
    ///
    /// The folding is the default one: Turkic folding would make `Istanbul`
    /// and `istanbul` two names, and folding by the group's language would let
    /// a change of language create duplicates.
    pub(crate) fn folded_name(&self) -> String {
        let normaliser = DecomposingNormalizerBorrowed::new_nfd();
        let decomposed = normaliser.normalize(&self.name);
        let folded = CaseMapper::new().fold_string(&decomposed);
        normaliser.normalize(&folded).into_owned()
    }
}

/// The serialised form of a [`Category`], validated on the way in.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct CategoryRecord {
    id: CategoryId,
    name: String,
}

#[cfg(feature = "serde")]
impl TryFrom<CategoryRecord> for Category {
    type Error = CategoryError;

    fn try_from(record: CategoryRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            id: record.id,
            name: text::required(record.name, CategoryError::EmptyName)?,
        })
    }
}

#[cfg(test)]
mod tests;
