//! The failure of building a transfer.

/// A [`Transfer`](crate::Transfer) that cannot exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum TransferError {
    /// The sender and the recipient are the same member.
    #[error("a transfer needs two different members")]
    SameMember,
    /// The amount is zero or negative.
    #[error("the amount must be positive")]
    NonPositiveAmount,
}

/// Builds a value from a [`TransferError`], one method per failure, each
/// given what its variant holds and the error's message.
pub trait FromTransferError: Sized {
    /// The value for [`TransferError::SameMember`].
    fn same_member(message: String) -> Self;

    /// The value for [`TransferError::NonPositiveAmount`].
    fn non_positive_amount(message: String) -> Self;

    /// The value for `error`, from the method of its variant, given the error's
    /// message.
    #[must_use]
    fn from_transfer_error(error: TransferError) -> Self {
        let message = error.to_string();
        match error {
            TransferError::SameMember => Self::same_member(message),
            TransferError::NonPositiveAmount => Self::non_positive_amount(message),
        }
    }
}

#[cfg(test)]
mod tests;
