//! The failure of building a member.

/// A [`Member`](crate::Member) that could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum MemberError {
    /// A first name that is blank.
    #[error("the first name must not be empty")]
    EmptyFirstName,
}

/// Builds a value from a [`MemberError`], one method per failure, each
/// given what its variant holds and the error's message.
pub trait FromMemberError: Sized {
    /// The value for [`MemberError::EmptyFirstName`].
    fn empty_first_name(message: String) -> Self;

    /// The value for `error`, from the method of its variant, given the error's
    /// message.
    #[must_use]
    fn from_member_error(error: MemberError) -> Self {
        let message = error.to_string();
        match error {
            MemberError::EmptyFirstName => Self::empty_first_name(message),
        }
    }
}

#[cfg(test)]
mod tests;
