//! The write handle: a group's reads and changes under one exclusive lock,
//! held until the handle is released or dropped.

use std::sync::{Mutex, MutexGuard, PoisonError};

use eranos::ExpenseGroup;
use eranos::expense_group::ExpenseGroupWriteGuard;

use crate::EranosError;

/// A group's reads and changes under one exclusive lock, so that no other
/// call runs in between: every other call waits until the handle is
/// released or dropped.
///
/// A binding frees the handle when its language's collector does, which
/// can be late, so [`release`](Self::release) releases the lock at once; every
/// call after it fails with [`EranosError::Released`]. Any synchronous call
/// through the group handle on the thread that holds a write handle waits
/// forever, since the handle holds the lock that call needs and the thread
/// cannot release it meanwhile: release or drop the handle first.
#[derive(Debug)]
pub struct ExpenseGroupWriter {
    /// The lock, until the handle is released.
    guard: Mutex<Option<ExpenseGroupWriteGuard>>,
}

impl ExpenseGroupWriter {
    /// A handle that reads and changes through `guard`.
    pub(crate) const fn new(guard: ExpenseGroupWriteGuard) -> Self {
        Self {
            guard: Mutex::new(Some(guard)),
        }
    }

    /// Releases the lock; releasing a released handle does nothing.
    pub fn release(&self) {
        self.guard().take();
    }

    /// Runs `call` on the group, while the handle is open.
    pub(crate) fn writing<T>(
        &self,
        call: impl FnOnce(&mut ExpenseGroup) -> Result<T, EranosError>,
    ) -> Result<T, EranosError> {
        let mut guard = self.guard();
        let group = guard.as_mut().ok_or_else(EranosError::released)?;
        call(group)
    }

    /// The lock's slot; a panic while it was taken leaves it whole, so a
    /// poisoned slot is taken as it is.
    fn guard(&self) -> MutexGuard<'_, Option<ExpenseGroupWriteGuard>> {
        self.guard.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests;
