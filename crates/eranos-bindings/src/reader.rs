//! The read handle: a group's reads under one shared lock, held until the
//! handle is released or dropped.

use std::sync::{Mutex, MutexGuard, PoisonError};

use eranos::Group;
use eranos::group::GroupReadGuard;

use crate::EranosError;
use crate::expense::Expense;
use crate::ledger::{ExpenseShares, Summary};
use crate::member::{Category, Member};
use crate::money::Money;
use crate::operations;
use crate::settlement::{Settlement, SettlementStrategy};
use crate::transfer::Transfer;

/// A group's reads under one shared lock, so that they see no change in
/// between: changes wait until the handle is released or dropped.
///
/// A binding frees the handle when its language's collector does, which
/// can be late, so [`release`](Self::release) releases the lock at once; every
/// read after it fails with [`EranosError::Released`].
#[derive(Debug)]
pub struct GroupReader {
    /// The lock, until the handle is released.
    guard: Mutex<Option<GroupReadGuard>>,
}

impl GroupReader {
    /// A handle that reads through `guard`.
    pub(crate) const fn new(guard: GroupReadGuard) -> Self {
        Self {
            guard: Mutex::new(Some(guard)),
        }
    }

    /// Releases the lock; releasing a released handle does nothing.
    pub fn release(&self) {
        self.guard().take();
    }

    /// The group's identifier.
    ///
    /// # Errors
    ///
    /// [`EranosError::Released`] after [`release`](Self::release).
    pub fn id(&self) -> Result<String, EranosError> {
        self.reading(|group| Ok(operations::id(group)))
    }

    /// Every category, the built-in one first.
    ///
    /// # Errors
    ///
    /// [`EranosError::Released`] after [`release`](Self::release).
    pub fn categories(&self) -> Result<Vec<Category>, EranosError> {
        self.reading(|group| Ok(operations::categories(group)))
    }

    /// The category with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Released`] after
    /// [`release`](Self::release).
    pub fn category(&self, id: &str) -> Result<Option<Category>, EranosError> {
        self.reading(|group| operations::category(group, id))
    }

    /// The ISO 4217 code of the group currency.
    ///
    /// # Errors
    ///
    /// [`EranosError::Released`] after [`release`](Self::release).
    pub fn currency(&self) -> Result<String, EranosError> {
        self.reading(|group| Ok(operations::currency_code(group)))
    }

    /// Every expense, in the order they were created.
    ///
    /// # Errors
    ///
    /// [`EranosError::Released`] after [`release`](Self::release).
    pub fn expenses(&self) -> Result<Vec<Expense>, EranosError> {
        self.reading(|group| Ok(operations::expenses(group)))
    }

    /// The expense with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Released`] after
    /// [`release`](Self::release).
    pub fn expense(&self, id: &str) -> Result<Option<Expense>, EranosError> {
        self.reading(|group| operations::expense(group, id))
    }

    /// How the expense with the identifier `id` divides, in the group currency.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Released`] after
    /// [`release`](Self::release).
    pub fn expense_shares(&self, id: &str) -> Result<Option<ExpenseShares>, EranosError> {
        self.reading(|group| operations::expense_shares(group, id))
    }

    /// The group's totals.
    ///
    /// # Errors
    ///
    /// [`EranosError::Released`] after [`release`](Self::release).
    pub fn summary(&self) -> Result<Summary, EranosError> {
        self.reading(|group| Ok(operations::summary(group)))
    }

    /// Every member, archived ones included, in the order they were created.
    ///
    /// # Errors
    ///
    /// [`EranosError::Released`] after [`release`](Self::release).
    pub fn members(&self) -> Result<Vec<Member>, EranosError> {
        self.reading(|group| Ok(operations::members(group)))
    }

    /// The member with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Released`] after
    /// [`release`](Self::release).
    pub fn member(&self, id: &str) -> Result<Option<Member>, EranosError> {
        self.reading(|group| operations::member(group, id))
    }

    /// Whether the member with the identifier `id` is archived; `false` for an
    /// unknown member.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Released`] after
    /// [`release`](Self::release).
    pub fn is_archived(&self, id: &str) -> Result<bool, EranosError> {
        self.reading(|group| operations::is_archived(group, id))
    }

    /// The group's name.
    ///
    /// # Errors
    ///
    /// [`EranosError::Released`] after [`release`](Self::release).
    pub fn name(&self) -> Result<String, EranosError> {
        self.reading(|group| Ok(operations::name(group)))
    }

    /// The payments that settle every balance by `strategy`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Released`] after
    /// [`release`](Self::release).
    pub fn settlement(&self, strategy: SettlementStrategy) -> Result<Settlement, EranosError> {
        self.reading(|group| operations::settlement(group, strategy))
    }

    /// Every transfer, in the order they were created.
    ///
    /// # Errors
    ///
    /// [`EranosError::Released`] after [`release`](Self::release).
    pub fn transfers(&self) -> Result<Vec<Transfer>, EranosError> {
        self.reading(|group| Ok(operations::transfers(group)))
    }

    /// The transfer with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Released`] after
    /// [`release`](Self::release).
    pub fn transfer(&self, id: &str) -> Result<Option<Transfer>, EranosError> {
        self.reading(|group| operations::transfer(group, id))
    }

    /// The amount in the group currency of the transfer with the identifier
    /// `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Released`] after
    /// [`release`](Self::release).
    pub fn transfer_amount(&self, id: &str) -> Result<Option<Money>, EranosError> {
        self.reading(|group| operations::transfer_amount(group, id))
    }

    /// The group's description.
    ///
    /// # Errors
    ///
    /// [`EranosError::Released`] after [`release`](Self::release).
    pub fn description(&self) -> Result<Option<String>, EranosError> {
        self.reading(|group| Ok(operations::description(group)))
    }

    /// The language the apps show the group in, as a BCP 47 tag.
    ///
    /// # Errors
    ///
    /// [`EranosError::Released`] after [`release`](Self::release).
    pub fn language(&self) -> Result<Option<String>, EranosError> {
        self.reading(|group| Ok(operations::language(group)))
    }

    /// The group in its saved form, as JSON.
    ///
    /// # Errors
    ///
    /// [`EranosError::Released`] after [`release`](Self::release).
    pub fn save(&self) -> Result<String, EranosError> {
        self.reading(|group| Ok(operations::save(group)))
    }

    /// Runs `read` on the group, while the handle is open.
    fn reading<T>(
        &self,
        read: impl FnOnce(&Group) -> Result<T, EranosError>,
    ) -> Result<T, EranosError> {
        let guard = self.guard();
        let group = guard.as_ref().ok_or_else(EranosError::released)?;
        read(group)
    }

    /// The lock's slot; a panic while it was taken leaves it whole, so a
    /// poisoned slot is taken as it is.
    fn guard(&self) -> MutexGuard<'_, Option<GroupReadGuard>> {
        self.guard.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests;
