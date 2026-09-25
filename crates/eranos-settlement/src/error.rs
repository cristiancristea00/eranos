//! The failure of computing a settlement.

/// A settlement that cannot be computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum SettlementError {
    /// The exact solver needs 2ⁿ bytes for `n` open balances, and that table
    /// does not fit in the address space or the allocator refused it.
    #[error("not enough memory to find the minimum transfers between {balances} members")]
    OutOfMemory {
        /// The number of members with an open balance after pairing exact
        /// opposites.
        balances: usize,
    },
}

/// Builds a value from a [`SettlementError`], one method per failure, each
/// given what its variant holds and the error's message.
pub trait FromSettlementError: Sized {
    /// The value for [`SettlementError::OutOfMemory`].
    fn out_of_memory(balances: usize, message: String) -> Self;

    /// The value for `error`, from the method of its variant, given the error's
    /// message.
    #[must_use]
    fn from_settlement_error(error: &SettlementError) -> Self {
        let message = error.to_string();
        match error {
            SettlementError::OutOfMemory { balances } => Self::out_of_memory(*balances, message),
        }
    }
}

#[cfg(test)]
mod tests;
