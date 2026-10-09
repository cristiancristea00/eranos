//! Expense categories, defined per group.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use. The facade does not re-export [`FromCategoryError`],
//! through which a crate that tells every failure apart converts one without
//! the wildcard arm a `match` outside this crate would need.
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

mod error;

use eranos_core::{CategoryId, text};
use icu_casemap::CaseMapper;
use icu_normalizer::DecomposingNormalizerBorrowed;

pub use error::{CategoryError, FromCategoryError};

/// A category a group sorts its expenses into, such as "Food" or "Fuel".
///
/// Every group defines its own categories; names are unique within a group
/// under Unicode canonical caseless matching, so `Food` and `FOOD`, `Straße`
/// and `STRASSE`, and a composed and a decomposed `Café` are one name, and so
/// are `Maße` and `Masse`. A category has the identifier given to
/// [`with_id`](Self::with_id), or one that [`new`](Self::new) generates,
/// and [`renamed`](Self::renamed) keeps it. Every group also has the
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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Category {
    id: CategoryId,
    name: String,
}

impl Category {
    /// Returns a category named `name`, trimmed, under an identifier
    /// generated when it is called; [`with_id`](Self::with_id) takes one
    /// instead.
    ///
    /// # Errors
    ///
    /// [`CategoryError::EmptyName`] when the name is blank.
    pub fn new(name: impl Into<String>) -> Result<Self, CategoryError> {
        Self::with_id(CategoryId::generate(), name)
    }

    /// Returns a category named `name`, trimmed, under the identifier `id`,
    /// such as the key an app's own store gives it; [`new`](Self::new)
    /// generates one instead.
    ///
    /// A group refuses to add a category under an identifier it already
    /// holds, [`CategoryId::UNCATEGORISED`] included.
    ///
    /// # Errors
    ///
    /// [`CategoryError::EmptyName`] when the name is blank.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{Category, CategoryId};
    ///
    /// let lodging = Category::with_id(CategoryId::new("lodging")?, " Lodging ")?;
    /// assert_eq!(lodging.id().as_str(), "lodging");
    /// assert_eq!(lodging.name(), "Lodging");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_id(id: CategoryId, name: impl Into<String>) -> Result<Self, CategoryError> {
        Ok(Self {
            id,
            name: text::required(name, CategoryError::EmptyName)?,
        })
    }

    /// Returns the built-in category every group has, "Uncategorised", under
    /// [`CategoryId::UNCATEGORISED`].
    ///
    /// The apps show its name in the user's language, and a group refuses to
    /// rename or remove it.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{Category, CategoryId};
    ///
    /// let built_in = Category::uncategorised();
    /// assert_eq!(built_in.id(), CategoryId::UNCATEGORISED);
    /// assert_eq!(built_in.name(), "Uncategorised");
    /// ```
    #[must_use]
    pub fn uncategorised() -> Self {
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
        Self::with_id(self.id, name)
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

    /// The name's canonical caseless form, on which a group checks that
    /// names are unique: Unicode's D145, `NFD(toCasefold(NFD(name)))`.
    ///
    /// The folding is the default one: Turkic folding would make `Istanbul`
    /// and `istanbul` two names, and folding by the group's language would let
    /// a change of language create duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::Category;
    ///
    /// let street = Category::new("Straße")?;
    /// assert_eq!(street.folded_name(), Category::new("STRASSE")?.folded_name());
    /// # Ok::<(), eranos::error::CategoryError>(())
    /// ```
    #[must_use]
    pub fn folded_name(&self) -> String {
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

eranos_core::deserialise_through!(Category, CategoryRecord, |record: CategoryRecord| {
    Self::with_id(record.id, record.name)
});

#[cfg(test)]
mod tests;
