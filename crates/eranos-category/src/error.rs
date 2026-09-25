//! The failure of naming a category.

/// A [`Category`](crate::Category) that could not be built or renamed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum CategoryError {
    /// A name that is blank.
    #[error("the category name must not be empty")]
    EmptyName,
}

/// Builds a value from a [`CategoryError`], one method per failure, each
/// given what its variant holds and the error's message.
pub trait FromCategoryError: Sized {
    /// The value for [`CategoryError::EmptyName`].
    fn empty_name(message: String) -> Self;

    /// The value for `error`, from the method of its variant, given the error's
    /// message.
    #[must_use]
    fn from_category_error(error: CategoryError) -> Self {
        let message = error.to_string();
        match error {
            CategoryError::EmptyName => Self::empty_name(message),
        }
    }
}

#[cfg(test)]
mod tests;
