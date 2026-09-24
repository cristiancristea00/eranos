//! The failure of naming a category.

/// A [`Category`](crate::Category) that could not be built or renamed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum CategoryError {
    /// A name that is blank.
    #[error("the category name must not be empty")]
    EmptyName,
}
