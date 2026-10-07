//! The objects the bindings hand out: the group, its read handle, and the
//! cancellation of an asynchronous call, each wrapping the binding core's
//! type and forwarding every method to it.

use std::sync::Arc;

use eranos_bindings::{
    Category, CurrencyRate, EranosError, Expense, ExpenseGroupHandle, ExpenseShares, Member, Money,
    NewExpenseGroup, Settlement, SettlementStrategy, Summary, Transfer,
};

/// A group, shared between the tasks and threads of a binding.
///
/// Every group call comes in two forms. The synchronous one waits for the
/// group's lock where a thread can sleep, and on wasm, whose one thread must
/// not, fails with `ExpenseGroupFailure::Busy` while another call holds it. The
/// asynchronous one, named with `_async`, waits for the lock, runs the call,
/// and stops with `EranosError::Cancelled` when its `Cancellation` is
/// cancelled first. Reads share the lock and changes take it alone.
#[derive(Debug, uniffi::Object)]
pub(crate) struct ExpenseGroup {
    handle: ExpenseGroupHandle,
}

#[uniffi::export]
impl ExpenseGroup {
    /// A new group with no members and no entries.
    ///
    /// # Errors
    ///
    /// `MoneyFailure::UnknownCurrency`,
    /// `ExpenseGroupFailure::EmptyName`, and
    /// `LanguageFailure::InvalidLanguage`.
    #[uniffi::constructor]
    pub(crate) fn new(group: &NewExpenseGroup) -> Result<Arc<Self>, EranosError> {
        ExpenseGroupHandle::new(group).map(Self::shared)
    }

    /// The group saved as `json`, loaded through the same checks as the
    /// calls that add its entries.
    ///
    /// # Errors
    ///
    /// `EranosError::InvalidJson` for text that is not the saved form, and
    /// the errors of the checks for entries that do not fit together.
    #[uniffi::constructor]
    pub(crate) fn load(json: &str) -> Result<Arc<Self>, EranosError> {
        ExpenseGroupHandle::load(json).map(Self::shared)
    }

    /// A read handle, which holds the shared lock until it is released or
    /// dropped, so that its reads see no change in between.
    ///
    /// A synchronous change on the thread that holds a read handle waits for
    /// the handle, which that thread cannot release meanwhile: release or
    /// drop the handle first.
    ///
    /// # Errors
    ///
    /// `ExpenseGroupFailure::Busy` on wasm while a change
    /// holds the lock.
    pub(crate) fn read(&self) -> Result<Arc<ExpenseGroupReader>, EranosError> {
        self.handle.read().map(ExpenseGroupReader::shared)
    }

    /// A read handle, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// `EranosError::Cancelled`.
    pub(crate) async fn read_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Arc<ExpenseGroupReader>, EranosError> {
        self.handle
            .read_async(shared(cancellation))
            .await
            .map(ExpenseGroupReader::shared)
    }
}

impl ExpenseGroup {
    /// The object of `handle`.
    fn shared(handle: ExpenseGroupHandle) -> Arc<Self> {
        Arc::new(Self { handle })
    }
}

/// The group saved as `json`, loaded as `ExpenseGroup::load` does,
/// yielding while it replays the entries.
///
/// # Errors
///
/// As for `ExpenseGroup::load`, and `EranosError::Cancelled`.
#[uniffi::export]
pub(crate) async fn load_expense_group_async(
    json: String,
    cancellation: Option<Arc<Cancellation>>,
) -> Result<Arc<ExpenseGroup>, EranosError> {
    ExpenseGroupHandle::load_async(json, shared(cancellation))
        .await
        .map(ExpenseGroup::shared)
}

/// A group's reads under one shared lock, so that they see no change in
/// between: changes wait until the handle is released or freed.
#[derive(Debug, uniffi::Object)]
pub(crate) struct ExpenseGroupReader {
    reader: eranos_bindings::ExpenseGroupReader,
}

#[uniffi::export]
impl ExpenseGroupReader {
    /// Releases the lock; releasing a released handle does nothing.
    pub(crate) fn release(&self) {
        self.reader.release();
    }
}

impl ExpenseGroupReader {
    /// The object of `reader`.
    fn shared(reader: eranos_bindings::ExpenseGroupReader) -> Arc<Self> {
        Arc::new(Self { reader })
    }
}

// Every group call's synchronous and asynchronous forms on `ExpenseGroup`, and every
// read's form on `ExpenseGroupReader`, stamped from the core's list of calls.
eranos_bindings::expense_group_calls!(::eranos_glue_macros::uniffi_objects);

/// Cancels the asynchronous calls it is given to: a call stops at its next
/// yield, or at once while it waits for the group's lock, and fails with
/// `EranosError::Cancelled`, leaving the group as it was.
#[derive(Debug, Default, uniffi::Object)]
pub(crate) struct Cancellation {
    shared: Arc<eranos_bindings::Cancellation>,
}

#[uniffi::export]
impl Cancellation {
    /// A cancellation not yet cancelled.
    #[uniffi::constructor]
    #[must_use]
    pub(crate) fn new() -> Arc<Self> {
        Arc::default()
    }

    /// Cancels every call that holds the cancellation.
    pub(crate) fn cancel(&self) {
        self.shared.cancel();
    }
}

/// The binding core's cancellation that `cancellation` holds.
fn shared(cancellation: Option<Arc<Cancellation>>) -> Option<Arc<eranos_bindings::Cancellation>> {
    Some(Arc::clone(&cancellation?.shared))
}
