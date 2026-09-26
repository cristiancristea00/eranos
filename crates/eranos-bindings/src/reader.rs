//! The read handle: a group's reads under one shared lock, held until the
//! handle is released or dropped.

use std::sync::{Mutex, MutexGuard, PoisonError};

use eranos::Group;
use eranos::group::GroupReadGuard;

use crate::EranosError;

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

    /// Runs `read` on the group, while the handle is open.
    pub(crate) fn reading<T>(
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
