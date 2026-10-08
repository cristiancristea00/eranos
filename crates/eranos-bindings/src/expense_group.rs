//! The group handle: every group call in a synchronous and an asynchronous
//! form, over a group shared between tasks and threads.

use std::future::Future;
use std::sync::Arc;

use eranos::expense_group::{ExpenseGroupReadGuard, ExpenseGroupWriteGuard, SavedExpenseGroup};
use eranos::{ExpenseGroup, ExpenseGroupId, Language, SharedExpenseGroup};

use crate::cancellation::{Cancellation, cancellable};
use crate::money::{CurrencyRate, currency};
use crate::operations::CurrencyChange;
use crate::reader::ExpenseGroupReader;
use crate::settlement::{SettlementPlan, SettlementStrategy};
use crate::{EranosError, identifier};

/// What a new group starts with.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NewExpenseGroup {
    /// The identifier, which the app gives: its own, or one
    /// [`identifier_generate`](crate::identifier_generate) returns.
    pub id: String,
    /// The group's name.
    pub name: String,
    /// The ISO 4217 code of the group currency, which every total is in.
    pub currency: String,
    /// A longer description.
    pub description: Option<String>,
    /// The language the apps show the group in, as a BCP 47 tag.
    pub language: Option<String>,
}

/// A group, shared between the tasks and threads of a binding.
///
/// Every group call comes in two forms. The synchronous one locks the group for
/// the call: it waits for the lock where a thread can sleep, and on wasm, whose
/// one thread must not, it fails with
/// [`ExpenseGroupFailure::Busy`](crate::ExpenseGroupFailure::Busy) instead when another call
/// holds the lock. The asynchronous one, named with `_async`, returns a `Send +
/// 'static` future that waits for the lock, runs the call, and stops with
/// [`EranosError::Cancelled`] when its [`Cancellation`] is cancelled first.
/// Reads share the lock and changes take it alone, as [`SharedExpenseGroup`] says.
#[derive(Debug, Clone)]
pub struct ExpenseGroupHandle {
    pub(crate) shared: SharedExpenseGroup,
}

impl ExpenseGroupHandle {
    /// A new group with no members and no entries, under the identifier
    /// `group` gives.
    ///
    /// # Errors
    ///
    /// An [`EranosError::Id`] for an identifier the rules refuse,
    /// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency),
    /// [`ExpenseGroupFailure::EmptyName`](crate::ExpenseGroupFailure::EmptyName), and
    /// [`LanguageFailure::InvalidLanguage`](crate::LanguageFailure::InvalidLanguage).
    pub fn new(group: &NewExpenseGroup) -> Result<Self, EranosError> {
        let id: ExpenseGroupId = identifier(&group.id)?;
        let language = group.language.as_deref().map(Language::parse).transpose()?;
        let builder =
            ExpenseGroup::builder_with_id(id, group.name.clone(), currency(&group.currency)?)
                .language(language);
        let builder = match &group.description {
            Some(description) => builder.description(description.as_str()),
            None => builder,
        };
        Ok(Self::from(builder.build()?))
    }

    /// The group saved as `json`, loaded through the same checks as the
    /// calls that add its entries.
    ///
    /// # Errors
    ///
    /// [`EranosError::InvalidJson`] for text that is not the saved form, and
    /// the errors of the checks for entries that do not fit together.
    pub fn load(json: &str) -> Result<Self, EranosError> {
        Ok(Self::from(saved(json)?.into_expense_group()?))
    }

    /// The group saved as `json`, loaded as [`load`](Self::load) does,
    /// yielding while it replays the entries.
    ///
    /// # Errors
    ///
    /// As for [`load`](Self::load), and [`EranosError::Cancelled`].
    pub async fn load_async(
        json: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Self, EranosError> {
        let saved = saved(&json)?;
        let group = cancellable(saved.into_expense_group_async(), cancellation).await??;
        Ok(Self::from(group))
    }

    // --- Reading under one lock ----------------------------------------------

    /// A read handle, which holds the shared lock until it is released or
    /// dropped, so that its reads see no change in between.
    ///
    /// A synchronous change on the thread that holds a read handle waits for
    /// the handle, which that thread cannot release meanwhile: release or
    /// drop the handle first.
    ///
    /// # Errors
    ///
    /// [`ExpenseGroupFailure::Busy`](crate::ExpenseGroupFailure::Busy) on wasm while a change
    /// holds the lock.
    pub fn read(&self) -> Result<ExpenseGroupReader, EranosError> {
        self.read_lock().map(ExpenseGroupReader::new)
    }

    /// A read handle, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// [`EranosError::Cancelled`].
    pub fn read_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<ExpenseGroupReader, EranosError>> + Send + 'static {
        let shared = self.shared.clone();
        async move {
            Ok(ExpenseGroupReader::new(
                cancellable(shared.read(), cancellation).await?,
            ))
        }
    }

    // --- Locking -------------------------------------------------------------

    /// Runs `read` under the shared lock of a synchronous call.
    pub(crate) fn reading<T>(
        &self,
        read: impl FnOnce(&ExpenseGroup) -> Result<T, EranosError>,
    ) -> Result<T, EranosError> {
        self.read_lock().and_then(|group| read(&group))
    }

    /// Runs `write` under the exclusive lock of a synchronous call.
    pub(crate) fn writing<T>(
        &self,
        write: impl FnOnce(&mut ExpenseGroup) -> Result<T, EranosError>,
    ) -> Result<T, EranosError> {
        self.write_lock().and_then(|mut group| write(&mut group))
    }

    /// A future that runs `read` under the shared lock once it has it.
    pub(crate) fn reading_async<T: Send + 'static>(
        &self,
        cancellation: Option<Arc<Cancellation>>,
        read: impl FnOnce(&ExpenseGroup) -> Result<T, EranosError> + Send + 'static,
    ) -> impl Future<Output = Result<T, EranosError>> + Send + 'static {
        let shared = self.shared.clone();
        async move { read(&*cancellable(shared.read(), cancellation).await?) }
    }

    /// A future that runs `write` under the exclusive lock once it has it.
    pub(crate) fn writing_async<T: Send + 'static>(
        &self,
        cancellation: Option<Arc<Cancellation>>,
        write: impl FnOnce(&mut ExpenseGroup) -> Result<T, EranosError> + Send + 'static,
    ) -> impl Future<Output = Result<T, EranosError>> + Send + 'static {
        let shared = self.shared.clone();
        async move { write(&mut *cancellable(shared.write(), cancellation).await?) }
    }

    /// The shared lock of a synchronous call: waited for where a thread can
    /// sleep.
    #[cfg(not(target_family = "wasm"))]
    #[expect(
        clippy::unnecessary_wraps,
        reason = "on wasm the same call fails when the lock is taken"
    )]
    fn read_lock(&self) -> Result<ExpenseGroupReadGuard, EranosError> {
        Ok(self.shared.read_blocking())
    }

    /// The shared lock of a synchronous call: tried, since wasm's one thread
    /// must not wait.
    #[cfg(target_family = "wasm")]
    fn read_lock(&self) -> Result<ExpenseGroupReadGuard, EranosError> {
        self.try_read_lock()
    }

    /// The exclusive lock of a synchronous call: waited for where a thread
    /// can sleep.
    #[cfg(not(target_family = "wasm"))]
    #[expect(
        clippy::unnecessary_wraps,
        reason = "on wasm the same call fails when the lock is taken"
    )]
    fn write_lock(&self) -> Result<ExpenseGroupWriteGuard, EranosError> {
        Ok(self.shared.write_blocking())
    }

    /// The exclusive lock of a synchronous call: tried, since wasm's one
    /// thread must not wait.
    #[cfg(target_family = "wasm")]
    fn write_lock(&self) -> Result<ExpenseGroupWriteGuard, EranosError> {
        self.try_write_lock()
    }

    /// The shared lock, taken without waiting, as a synchronous call takes
    /// it on wasm.
    #[cfg(any(test, target_family = "wasm"))]
    pub(crate) fn try_read_lock(&self) -> Result<ExpenseGroupReadGuard, EranosError> {
        Ok(self.shared.try_read()?)
    }

    /// The exclusive lock, taken without waiting, as a synchronous call
    /// takes it on wasm.
    #[cfg(any(test, target_family = "wasm"))]
    pub(crate) fn try_write_lock(&self) -> Result<ExpenseGroupWriteGuard, EranosError> {
        Ok(self.shared.try_write()?)
    }
}

/// The asynchronous currency change of the group `shared` holds: the change
/// to `currency` at `rates` is checked before the exclusive lock is taken,
/// and yields while it holds it, so a cancellation stops it at its next
/// yield, the group unchanged.
///
/// # Errors
///
/// The library's refusal, and [`EranosError::Cancelled`].
pub(crate) async fn change_currency(
    shared: SharedExpenseGroup,
    currency: String,
    rates: Vec<CurrencyRate>,
    cancellation: Option<Arc<Cancellation>>,
) -> Result<(), EranosError> {
    let change = CurrencyChange::new(&currency, &rates)?;
    let mut group = cancellable(shared.write(), cancellation.clone()).await?;
    let changing = group.change_currency_async(change.currency, &change.rates);
    let changed = cancellable(changing, cancellation).await?;
    drop(group);
    Ok(changed?)
}

/// The asynchronous settlement of the group `shared` holds by `strategy`:
/// the cooperative computation runs under the shared lock, yielding, so a
/// cancellation stops it at its next yield.
///
/// # Errors
///
/// The library's refusal, and [`EranosError::Cancelled`].
pub(crate) async fn settlement(
    shared: SharedExpenseGroup,
    strategy: SettlementStrategy,
    cancellation: Option<Arc<Cancellation>>,
) -> Result<SettlementPlan, EranosError> {
    let group = cancellable(shared.read(), cancellation.clone()).await?;
    let kept = cancellable(group.settlement_async(strategy.to_library()), cancellation).await??;
    let settlement = SettlementPlan::from_library(kept);
    drop(group);
    Ok(settlement)
}

impl From<ExpenseGroup> for ExpenseGroupHandle {
    fn from(group: ExpenseGroup) -> Self {
        Self {
            shared: SharedExpenseGroup::new(group),
        }
    }
}

/// The saved form `json`, read but not yet loaded.
///
/// # Errors
///
/// [`EranosError::InvalidJson`] for text that is not JSON of the saved form.
fn saved(json: &str) -> Result<SavedExpenseGroup, EranosError> {
    serde_json::from_str(json).map_err(|error| EranosError::invalid_json(&error))
}

#[cfg(test)]
mod tests;
