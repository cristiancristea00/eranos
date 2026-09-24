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
