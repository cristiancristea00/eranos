//! A group shared between tasks and threads, behind a read-write lock.

use std::fmt;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use async_lock::{RwLock, RwLockReadGuardArc, RwLockWriteGuardArc};
#[cfg(not(target_family = "wasm"))]
use eranos_core::cooperative::block_on;

use crate::{ExpenseGroup, ExpenseGroupError};

/// An [`ExpenseGroup`] shared between tasks and threads behind a read-write
/// lock.
///
/// [`read`](Self::read) and [`write`](Self::write) wait for the lock and
/// return guards that dereference to the group, through which a caller uses
/// the group's own methods until the guard is dropped. Reads take a shared
/// lock and run alongside each other; changes take an exclusive one and run
/// alone, once the reads in progress are done. While a change waits, new
/// reads wait behind it, so a stream of reads cannot keep a change waiting.
/// A guard keeps the lock across a yield: a settlement awaited through a
/// read guard holds the shared lock while the exact solver fills its table,
/// and a currency change through a write guard holds the exclusive one while
/// it re-converts the entries.
///
/// The futures `read` and `write` return are `Send + 'static`, so an executor
/// can run them on any thread, and so are the guards, which borrow nothing.
/// A wait dropped before the lock is granted takes nothing. A synchronous
/// caller waits with [`read_blocking`](Self::read_blocking) and
/// [`write_blocking`](Self::write_blocking), which sleep meanwhile, or tries
/// the lock with [`try_read`](Self::try_read) and
/// [`try_write`](Self::try_write). Clones share one group.
/// [`Debug`](fmt::Debug) never locks, so it prints none of the group.
///
/// # Examples
///
/// ```
/// use eranos::cooperative::block_on;
/// use eranos::{CurrencyCode, ExpenseGroup, Member, SharedExpenseGroup};
///
/// let shared = SharedExpenseGroup::new(ExpenseGroup::builder("Flat", CurrencyCode::EUR).build()?);
/// let other = shared.clone();
///
/// // A change through one clone is read through the other.
/// let ana = block_on(shared.write()).add_member(Member::builder("Ana").build()?)?;
/// let group = block_on(other.read());
/// let member = group.member(ana).ok_or("Ana is a member")?;
/// assert_eq!(member.first_name(), "Ana");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone)]
pub struct SharedExpenseGroup {
    group: Arc<RwLock<ExpenseGroup>>,
}

impl SharedExpenseGroup {
    /// Shares `group`.
    #[must_use]
    pub fn new(group: ExpenseGroup) -> Self {
        Self {
            group: Arc::new(RwLock::new(group)),
        }
    }

    /// Waits for the shared lock, then reads the group through the guard
    /// until it is dropped.
    pub fn read(&self) -> impl Future<Output = ExpenseGroupReadGuard> + Send + 'static {
        let group = Arc::clone(&self.group);
        async move { ExpenseGroupReadGuard(group.read_arc().await) }
    }

    /// Waits for the exclusive lock, then changes the group through the
    /// guard until it is dropped.
    pub fn write(&self) -> impl Future<Output = ExpenseGroupWriteGuard> + Send + 'static {
        let group = Arc::clone(&self.group);
        async move { ExpenseGroupWriteGuard(group.write_arc().await) }
    }

    /// Takes the shared lock without waiting.
    ///
    /// # Errors
    ///
    /// [`ExpenseGroupError::Busy`] when a change holds the lock or waits for it.
    pub fn try_read(&self) -> Result<ExpenseGroupReadGuard, ExpenseGroupError> {
        self.group
            .try_read_arc()
            .map(ExpenseGroupReadGuard)
            .ok_or(ExpenseGroupError::Busy)
    }

    /// Takes the exclusive lock without waiting.
    ///
    /// # Errors
    ///
    /// [`ExpenseGroupError::Busy`] when another call holds the lock.
    pub fn try_write(&self) -> Result<ExpenseGroupWriteGuard, ExpenseGroupError> {
        self.group
            .try_write_arc()
            .map(ExpenseGroupWriteGuard)
            .ok_or(ExpenseGroupError::Busy)
    }

    /// Waits for the shared lock on this thread, which sleeps meanwhile.
    ///
    /// It is not compiled for wasm, where the one thread would sleep with
    /// no thread left to release the lock; [`try_read`](Self::try_read)
    /// works there.
    #[cfg(not(target_family = "wasm"))]
    pub fn read_blocking(&self) -> ExpenseGroupReadGuard {
        block_on(self.read())
    }

    /// Waits for the exclusive lock on this thread, which sleeps meanwhile.
    ///
    /// It is not compiled for wasm, where the one thread would sleep with
    /// no thread left to release the lock; [`try_write`](Self::try_write)
    /// works there.
    #[cfg(not(target_family = "wasm"))]
    pub fn write_blocking(&self) -> ExpenseGroupWriteGuard {
        block_on(self.write())
    }

    /// The group, taken back from its last handle.
    ///
    /// # Errors
    ///
    /// The shared group itself, while a clone of it, a guard, or a wait for
    /// the lock still shares the group.
    pub fn into_inner(self) -> Result<ExpenseGroup, Self> {
        Arc::try_unwrap(self.group)
            .map(RwLock::into_inner)
            .map_err(|group| Self { group })
    }
}

impl fmt::Debug for SharedExpenseGroup {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SharedExpenseGroup")
            .finish_non_exhaustive()
    }
}

/// Reads a [`SharedExpenseGroup`] under its shared lock, which it holds until it is
/// dropped; it dereferences to the [`ExpenseGroup`].
///
/// It borrows nothing, so it can be kept across an await or moved to
/// another thread.
#[derive(Debug)]
#[must_use = "the lock is released as soon as the guard is dropped"]
pub struct ExpenseGroupReadGuard(RwLockReadGuardArc<ExpenseGroup>);

impl Deref for ExpenseGroupReadGuard {
    type Target = ExpenseGroup;

    fn deref(&self) -> &ExpenseGroup {
        &self.0
    }
}

/// Changes a [`SharedExpenseGroup`] under its exclusive lock, which it holds until
/// it is dropped; it dereferences to the [`ExpenseGroup`].
///
/// It borrows nothing, so it can be kept across an await or moved to
/// another thread.
#[derive(Debug)]
#[must_use = "the lock is released as soon as the guard is dropped"]
pub struct ExpenseGroupWriteGuard(RwLockWriteGuardArc<ExpenseGroup>);

impl Deref for ExpenseGroupWriteGuard {
    type Target = ExpenseGroup;

    fn deref(&self) -> &ExpenseGroup {
        &self.0
    }
}

impl DerefMut for ExpenseGroupWriteGuard {
    fn deref_mut(&mut self) -> &mut ExpenseGroup {
        &mut self.0
    }
}

#[cfg(test)]
mod tests;
