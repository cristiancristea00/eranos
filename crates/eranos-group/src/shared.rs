//! A group shared between tasks and threads, behind a read-write lock.

use std::fmt;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use async_lock::{RwLock, RwLockReadGuardArc, RwLockWriteGuardArc};
use eranos_category::Category;
#[cfg(not(target_family = "wasm"))]
use eranos_core::cooperative::block_on;
use eranos_core::{CategoryId, ExpenseId, GroupId, MemberId, TransferId};
use eranos_expense::Expense;
use eranos_ledger::{ExpenseShares, Summary};
use eranos_member::Member;
use eranos_money::{Currency, CurrencyRate, Money};
use eranos_settlement::{Settlement, SettlementError, SettlementStrategy};
use eranos_transfer::Transfer;

use crate::{Group, GroupError, Language};

/// A [`Group`] shared between tasks and threads: each call locks the group
/// while it runs and returns, owned, what the group's method of the same
/// name returns.
///
/// Reads take a shared lock and run alongside each other; changes take an
/// exclusive one and run alone, once the reads in progress are done. While a
/// change waits, new reads wait behind it, so a stream of reads cannot keep
/// a change waiting. Two calls yield while they hold the lock:
/// [`settlement`](Self::settlement), under the shared lock, while the exact
/// solver fills its table, and [`change_currency`](Self::change_currency),
/// under the exclusive one, while it re-converts the entries. Every other
/// call holds the lock only for one poll.
///
/// Every call returns a future that is `Send + 'static`, so an executor can
/// run it on any thread. A call dropped before it completes changes nothing
/// and releases the lock it holds or waits for. A synchronous caller drives
/// a call with [`block_on`](eranos_core::cooperative::block_on), which sleeps
/// while the call waits for the lock.
///
/// A caller that needs several reads or changes to see no other call in
/// between takes a guard, with [`read`](Self::read) or
/// [`write`](Self::write), and calls the group's own methods on it. Clones
/// share one group. [`Debug`](fmt::Debug) never locks, so it prints none of
/// the group.
///
/// # Examples
///
/// ```
/// use eranos::cooperative::block_on;
/// use eranos::{Currency, Group, Member, SharedGroup};
///
/// let shared = SharedGroup::new(Group::builder(Currency::EUR, "Flat").build()?);
/// let other = shared.clone();
///
/// // A change through one clone is read through the other.
/// let ana = block_on(shared.add_member(Member::builder("Ana").build()?))?;
/// let member = block_on(other.member(ana)).ok_or("Ana is a member")?;
/// assert_eq!(member.first_name(), "Ana");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone)]
pub struct SharedGroup {
    group: Arc<RwLock<Group>>,
}

impl SharedGroup {
    /// Shares `group`.
    #[must_use]
    pub fn new(group: Group) -> Self {
        Self {
            group: Arc::new(RwLock::new(group)),
        }
    }

    // --- Identifier ----------------------------------------------------------

    /// The identifier, as [`Group::id`] returns it.
    pub fn id(&self) -> impl Future<Output = GroupId> + Send + 'static {
        self.read_with(Group::id)
    }

    // --- Categories ----------------------------------------------------------

    /// Every category, as [`Group::categories`] lists them.
    pub fn categories(&self) -> impl Future<Output = Vec<Category>> + Send + 'static {
        self.read_with(|group| group.categories().cloned().collect())
    }

    /// The category with this identifier, as [`Group::category`] finds it.
    pub fn category(
        &self,
        id: CategoryId,
    ) -> impl Future<Output = Option<Category>> + Send + 'static {
        self.read_with(move |group| group.category(id).cloned())
    }

    /// Adds a category, as [`Group::add_category`] does.
    ///
    /// # Errors
    ///
    /// As for [`Group::add_category`].
    pub fn add_category(
        &self,
        category: Category,
    ) -> impl Future<Output = Result<CategoryId, GroupError>> + Send + 'static {
        self.write_with(move |group| group.add_category(category))
    }

    /// Replaces a category, as [`Group::replace_category`] does.
    ///
    /// # Errors
    ///
    /// As for [`Group::replace_category`].
    pub fn replace_category(
        &self,
        category: Category,
    ) -> impl Future<Output = Result<Category, GroupError>> + Send + 'static {
        self.write_with(move |group| group.replace_category(category))
    }

    /// Removes a category, as [`Group::remove_category`] does.
    ///
    /// # Errors
    ///
    /// As for [`Group::remove_category`].
    pub fn remove_category(
        &self,
        id: CategoryId,
    ) -> impl Future<Output = Result<Category, GroupError>> + Send + 'static {
        self.write_with(move |group| group.remove_category(id))
    }

    // --- Currency ------------------------------------------------------------

    /// The currency of every total, balance, and settlement, as
    /// [`Group::currency`] returns it.
    pub fn currency(&self) -> impl Future<Output = Currency> + Send + 'static {
        self.read_with(Group::currency)
    }

    /// Moves the group to another currency, as [`Group::change_currency`]
    /// does, with a rate for every other currency its entries use.
    ///
    /// The call holds the exclusive lock while it re-converts the entries
    /// and yields as [`Group::change_currency_async`] does; dropped at a
    /// yield, it leaves the group as it was.
    ///
    /// # Errors
    ///
    /// As for [`Group::change_currency`].
    pub fn change_currency(
        &self,
        currency: Currency,
        rates: impl Into<Vec<CurrencyRate>>,
    ) -> impl Future<Output = Result<(), GroupError>> + Send + 'static {
        let group = Arc::clone(&self.group);
        let rates = rates.into();
        async move {
            let mut group = group.write().await;
            group.change_currency_async(currency, &rates).await
        }
    }

    // --- Expenses ------------------------------------------------------------

    /// Every expense, as [`Group::expenses`] lists them.
    pub fn expenses(&self) -> impl Future<Output = Vec<Expense>> + Send + 'static {
        self.read_with(|group| group.expenses().cloned().collect())
    }

    /// The expense with this identifier, as [`Group::expense`] finds it.
    pub fn expense(&self, id: ExpenseId) -> impl Future<Output = Option<Expense>> + Send + 'static {
        self.read_with(move |group| group.expense(id).cloned())
    }

    /// How an expense divides in the group currency, as
    /// [`Group::expense_shares`] returns it.
    pub fn expense_shares(
        &self,
        id: ExpenseId,
    ) -> impl Future<Output = Option<ExpenseShares>> + Send + 'static {
        self.read_with(move |group| group.expense_shares(id).cloned())
    }

    /// Adds an expense, as [`Group::add_expense`] does.
    ///
    /// # Errors
    ///
    /// As for [`Group::add_expense`].
    pub fn add_expense(
        &self,
        expense: Expense,
    ) -> impl Future<Output = Result<ExpenseId, GroupError>> + Send + 'static {
        self.write_with(move |group| group.add_expense(expense))
    }

    /// Replaces an expense, as [`Group::replace_expense`] does.
    ///
    /// # Errors
    ///
    /// As for [`Group::replace_expense`].
    pub fn replace_expense(
        &self,
        expense: Expense,
    ) -> impl Future<Output = Result<Expense, GroupError>> + Send + 'static {
        self.write_with(move |group| group.replace_expense(expense))
    }

    /// Removes an expense, as [`Group::remove_expense`] does.
    ///
    /// # Errors
    ///
    /// As for [`Group::remove_expense`].
    pub fn remove_expense(
        &self,
        id: ExpenseId,
    ) -> impl Future<Output = Result<Expense, GroupError>> + Send + 'static {
        self.write_with(move |group| group.remove_expense(id))
    }

    // --- Ledger --------------------------------------------------------------

    /// The group's totals, as [`Group::summary`] returns them.
    pub fn summary(&self) -> impl Future<Output = Summary> + Send + 'static {
        self.read_with(|group| group.summary().clone())
    }

    // --- Members -------------------------------------------------------------

    /// Every member, as [`Group::members`] lists them.
    pub fn members(&self) -> impl Future<Output = Vec<Member>> + Send + 'static {
        self.read_with(|group| group.members().cloned().collect())
    }

    /// The member with this identifier, as [`Group::member`] finds them.
    pub fn member(&self, id: MemberId) -> impl Future<Output = Option<Member>> + Send + 'static {
        self.read_with(move |group| group.member(id).cloned())
    }

    /// Whether the member is archived, as [`Group::is_archived`] says.
    pub fn is_archived(&self, id: MemberId) -> impl Future<Output = bool> + Send + 'static {
        self.read_with(move |group| group.is_archived(id))
    }

    /// Adds a member, as [`Group::add_member`] does.
    ///
    /// # Errors
    ///
    /// As for [`Group::add_member`].
    pub fn add_member(
        &self,
        member: Member,
    ) -> impl Future<Output = Result<MemberId, GroupError>> + Send + 'static {
        self.write_with(move |group| group.add_member(member))
    }

    /// Replaces a member, as [`Group::replace_member`] does.
    ///
    /// # Errors
    ///
    /// As for [`Group::replace_member`].
    pub fn replace_member(
        &self,
        member: Member,
    ) -> impl Future<Output = Result<Member, GroupError>> + Send + 'static {
        self.write_with(move |group| group.replace_member(member))
    }

    /// Archives a member, as [`Group::archive_member`] does.
    ///
    /// # Errors
    ///
    /// As for [`Group::archive_member`].
    pub fn archive_member(
        &self,
        id: MemberId,
    ) -> impl Future<Output = Result<(), GroupError>> + Send + 'static {
        self.write_with(move |group| group.archive_member(id))
    }

    /// Unarchives a member, as [`Group::unarchive_member`] does.
    ///
    /// # Errors
    ///
    /// As for [`Group::unarchive_member`].
    pub fn unarchive_member(
        &self,
        id: MemberId,
    ) -> impl Future<Output = Result<(), GroupError>> + Send + 'static {
        self.write_with(move |group| group.unarchive_member(id))
    }

    /// Removes a member, as [`Group::remove_member`] does.
    ///
    /// # Errors
    ///
    /// As for [`Group::remove_member`].
    pub fn remove_member(
        &self,
        id: MemberId,
    ) -> impl Future<Output = Result<Member, GroupError>> + Send + 'static {
        self.write_with(move |group| group.remove_member(id))
    }

    // --- Name ----------------------------------------------------------------

    /// The name, as [`Group::name`] returns it.
    pub fn name(&self) -> impl Future<Output = String> + Send + 'static {
        self.read_with(|group| group.name().to_owned())
    }

    /// Renames the group, as [`Group::set_name`] does.
    ///
    /// # Errors
    ///
    /// As for [`Group::set_name`].
    pub fn set_name(
        &self,
        name: impl Into<String>,
    ) -> impl Future<Output = Result<(), GroupError>> + Send + 'static {
        let name = name.into();
        self.write_with(move |group| group.set_name(name))
    }

    // --- Settlements ---------------------------------------------------------

    /// The payments that settle every balance, as [`Group::settlement`]
    /// computes and caches them.
    ///
    /// The call holds the shared lock while the exact solver fills its
    /// table and yields as [`Group::settlement_async`] does; dropped at a
    /// yield, it caches nothing.
    ///
    /// # Errors
    ///
    /// As for [`Group::settlement`].
    pub fn settlement(
        &self,
        strategy: SettlementStrategy,
    ) -> impl Future<Output = Result<Settlement, SettlementError>> + Send + 'static {
        let group = Arc::clone(&self.group);
        async move {
            let group = group.read().await;
            group.settlement_async(strategy).await.cloned()
        }
    }

    // --- Transfers -----------------------------------------------------------

    /// Every transfer, as [`Group::transfers`] lists them.
    pub fn transfers(&self) -> impl Future<Output = Vec<Transfer>> + Send + 'static {
        self.read_with(|group| group.transfers().cloned().collect())
    }

    /// The transfer with this identifier, as [`Group::transfer`] finds it.
    pub fn transfer(
        &self,
        id: TransferId,
    ) -> impl Future<Output = Option<Transfer>> + Send + 'static {
        self.read_with(move |group| group.transfer(id).cloned())
    }

    /// A transfer's amount in the group currency, as
    /// [`Group::transfer_amount`] returns it.
    pub fn transfer_amount(
        &self,
        id: TransferId,
    ) -> impl Future<Output = Option<Money>> + Send + 'static {
        self.read_with(move |group| group.transfer_amount(id))
    }

    /// Adds a transfer, as [`Group::add_transfer`] does.
    ///
    /// # Errors
    ///
    /// As for [`Group::add_transfer`].
    pub fn add_transfer(
        &self,
        transfer: Transfer,
    ) -> impl Future<Output = Result<TransferId, GroupError>> + Send + 'static {
        self.write_with(move |group| group.add_transfer(transfer))
    }

    /// Replaces a transfer, as [`Group::replace_transfer`] does.
    ///
    /// # Errors
    ///
    /// As for [`Group::replace_transfer`].
    pub fn replace_transfer(
        &self,
        transfer: Transfer,
    ) -> impl Future<Output = Result<Transfer, GroupError>> + Send + 'static {
        self.write_with(move |group| group.replace_transfer(transfer))
    }

    /// Removes a transfer, as [`Group::remove_transfer`] does.
    ///
    /// # Errors
    ///
    /// As for [`Group::remove_transfer`].
    pub fn remove_transfer(
        &self,
        id: TransferId,
    ) -> impl Future<Output = Result<Transfer, GroupError>> + Send + 'static {
        self.write_with(move |group| group.remove_transfer(id))
    }

    // --- Description ---------------------------------------------------------

    /// The description, if any, as [`Group::description`] returns it.
    pub fn description(&self) -> impl Future<Output = Option<String>> + Send + 'static {
        self.read_with(|group| group.description().map(str::to_owned))
    }

    /// Sets the description, as [`Group::set_description`] does.
    pub fn set_description(
        &self,
        description: impl Into<String>,
    ) -> impl Future<Output = ()> + Send + 'static {
        let description = description.into();
        self.write_with(move |group| group.set_description(description))
    }

    // --- Language ------------------------------------------------------------

    /// The language, if set, as [`Group::language`] returns it.
    pub fn language(&self) -> impl Future<Output = Option<Language>> + Send + 'static {
        self.read_with(|group| group.language().cloned())
    }

    /// Sets or clears the language, as [`Group::set_language`] does.
    pub fn set_language(
        &self,
        language: impl Into<Option<Language>>,
    ) -> impl Future<Output = ()> + Send + 'static {
        let language = language.into();
        self.write_with(move |group| group.set_language(language))
    }

    // --- Locking -------------------------------------------------------------

    /// Waits for the shared lock, then reads the group through the guard
    /// until it is dropped.
    pub fn read(&self) -> impl Future<Output = GroupReadGuard> + Send + 'static {
        let group = Arc::clone(&self.group);
        async move { GroupReadGuard(group.read_arc().await) }
    }

    /// Waits for the exclusive lock, then changes the group through the
    /// guard until it is dropped.
    pub fn write(&self) -> impl Future<Output = GroupWriteGuard> + Send + 'static {
        let group = Arc::clone(&self.group);
        async move { GroupWriteGuard(group.write_arc().await) }
    }

    /// Takes the shared lock without waiting.
    ///
    /// # Errors
    ///
    /// [`GroupError::Busy`] when a change holds the lock or waits for it.
    pub fn try_read(&self) -> Result<GroupReadGuard, GroupError> {
        self.group
            .try_read_arc()
            .map(GroupReadGuard)
            .ok_or(GroupError::Busy)
    }

    /// Takes the exclusive lock without waiting.
    ///
    /// # Errors
    ///
    /// [`GroupError::Busy`] when another call holds the lock.
    pub fn try_write(&self) -> Result<GroupWriteGuard, GroupError> {
        self.group
            .try_write_arc()
            .map(GroupWriteGuard)
            .ok_or(GroupError::Busy)
    }

    /// Waits for the shared lock on this thread, which sleeps meanwhile.
    ///
    /// It is not compiled for wasm, where the one thread would sleep with
    /// no thread left to release the lock; [`try_read`](Self::try_read)
    /// works there.
    #[cfg(not(target_family = "wasm"))]
    pub fn read_blocking(&self) -> GroupReadGuard {
        block_on(self.read())
    }

    /// Waits for the exclusive lock on this thread, which sleeps meanwhile.
    ///
    /// It is not compiled for wasm, where the one thread would sleep with
    /// no thread left to release the lock; [`try_write`](Self::try_write)
    /// works there.
    #[cfg(not(target_family = "wasm"))]
    pub fn write_blocking(&self) -> GroupWriteGuard {
        block_on(self.write())
    }

    /// Whether a call holds the lock, shared or exclusive.
    #[must_use]
    pub fn is_locked(&self) -> bool {
        self.group.try_write().is_none()
    }

    /// The group, taken back from its last handle.
    ///
    /// # Errors
    ///
    /// The shared group itself, while a clone of it or a call still in
    /// progress shares the group.
    pub fn into_group(self) -> Result<Group, Self> {
        Arc::try_unwrap(self.group)
            .map(RwLock::into_inner)
            .map_err(|group| Self { group })
    }

    /// A call that reads the group under the shared lock, which it holds
    /// for one poll.
    fn read_with<T: Send + 'static>(
        &self,
        read: impl FnOnce(&Group) -> T + Send + 'static,
    ) -> impl Future<Output = T> + Send + 'static {
        let group = Arc::clone(&self.group);
        async move { read(&*group.read().await) }
    }

    /// A call that changes the group under the exclusive lock, which it
    /// holds for one poll.
    fn write_with<T: Send + 'static>(
        &self,
        write: impl FnOnce(&mut Group) -> T + Send + 'static,
    ) -> impl Future<Output = T> + Send + 'static {
        let group = Arc::clone(&self.group);
        async move { write(&mut *group.write().await) }
    }
}

impl fmt::Debug for SharedGroup {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SharedGroup")
            .finish_non_exhaustive()
    }
}

/// Reads a [`SharedGroup`] under its shared lock, which it holds until it is
/// dropped; it dereferences to the [`Group`].
///
/// It borrows nothing, so it can be kept across an await or moved to
/// another thread.
#[derive(Debug)]
#[must_use = "the lock is released as soon as the guard is dropped"]
pub struct GroupReadGuard(RwLockReadGuardArc<Group>);

impl Deref for GroupReadGuard {
    type Target = Group;

    fn deref(&self) -> &Group {
        &self.0
    }
}

/// Changes a [`SharedGroup`] under its exclusive lock, which it holds until
/// it is dropped; it dereferences to the [`Group`].
///
/// It borrows nothing, so it can be kept across an await or moved to
/// another thread.
#[derive(Debug)]
#[must_use = "the lock is released as soon as the guard is dropped"]
pub struct GroupWriteGuard(RwLockWriteGuardArc<Group>);

impl Deref for GroupWriteGuard {
    type Target = Group;

    fn deref(&self) -> &Group {
        &self.0
    }
}

impl DerefMut for GroupWriteGuard {
    fn deref_mut(&mut self) -> &mut Group {
        &mut self.0
    }
}

#[cfg(test)]
mod tests;
