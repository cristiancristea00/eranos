//! The failure of building a transfer.

/// A [`Transfer`](crate::Transfer) that cannot exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum TransferError {
    /// The amount is zero or negative.
    #[error("the amount must be positive")]
    NonPositiveAmount,
    /// The sender and the recipient are the same member.
    #[error("a transfer needs two different members")]
    SameMember,
}
