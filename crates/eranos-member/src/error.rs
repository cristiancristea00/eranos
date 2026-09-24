//! The failure of building a member.

/// A [`Member`](crate::Member) that could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum MemberError {
    /// A first name that is blank.
    #[error("the first name must not be empty")]
    EmptyFirstName,
}
