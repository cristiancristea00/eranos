//! The group handle: every group call in a synchronous and an asynchronous
//! form, over a group shared between tasks and threads.

use std::future::Future;
use std::sync::Arc;

use eranos::group::{GroupReadGuard, GroupWriteGuard, SavedGroup};
use eranos::{Group, Language, SharedGroup};

use crate::EranosError;
use crate::cancellation::{Cancellation, cancellable};
use crate::expense::Expense;
use crate::ledger::{ExpenseShares, Summary};
use crate::member::{Category, Member};
use crate::money::{CurrencyRate, Money, currency};
use crate::operations::{self, CurrencyChange};
use crate::reader::GroupReader;
use crate::settlement::{Settlement, SettlementStrategy};
use crate::transfer::Transfer;

/// What a new group starts with.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NewGroup {
    /// The ISO 4217 code of the group currency, which every total is in.
    pub currency: String,
    /// The group's name.
    pub name: String,
    /// A longer description.
    pub description: Option<String>,
    /// The language the apps show the group in, as a BCP 47 tag.
    pub language: Option<String>,
}

/// A group, shared between the tasks and threads of a binding.
///
/// Every group call comes in two forms. The synchronous one locks the group
/// for the call: it waits for the lock where a thread can sleep, and on wasm,
/// whose one thread must not, it fails with [`EranosError::Busy`] instead
/// when another call holds the lock. The asynchronous one, named with
/// `_async`, returns a `Send + 'static` future that waits for the lock, runs
/// the call, and stops with [`EranosError::Cancelled`] when its
/// [`Cancellation`] is cancelled first. Reads share the lock and changes take
/// it alone, as [`SharedGroup`] says.
#[derive(Debug, Clone)]
pub struct GroupHandle {
    shared: SharedGroup,
}

impl GroupHandle {
    /// A new group with no members and no entries.
    ///
    /// # Errors
    ///
    /// [`EranosError::UnknownCurrency`], [`EranosError::GroupEmptyName`], and
    /// [`EranosError::InvalidLanguage`].
    pub fn new(group: &NewGroup) -> Result<Self, EranosError> {
        let language = group.language.as_deref().map(Language::parse).transpose()?;
        let group = Group::builder(currency(&group.currency)?, group.name.clone())
            .description(group.description.clone().unwrap_or_default())
            .language(language)
            .build()?;
        Ok(Self::from(group))
    }

    /// The group saved as `json`, loaded through the same checks as the
    /// calls that add its entries.
    ///
    /// # Errors
    ///
    /// [`EranosError::InvalidJson`] for text that is not the saved form, and
    /// the errors of the checks for entries that do not fit together.
    pub fn load(json: &str) -> Result<Self, EranosError> {
        Ok(Self::from(saved(json)?.into_group()?))
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
        let group = cancellable(saved.into_group_async(), cancellation).await??;
        Ok(Self::from(group))
    }

    // --- Identifier ----------------------------------------------------------

    /// The group's identifier.
    ///
    /// # Errors
    ///
    /// [`EranosError::Busy`] on wasm while another call holds the lock.
    pub fn id(&self) -> Result<String, EranosError> {
        self.reading(|group| Ok(operations::id(group)))
    }

    /// The group's identifier, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// [`EranosError::Cancelled`].
    pub fn id_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<String, EranosError>> + Send + 'static {
        self.reading_async(cancellation, |group| Ok(operations::id(group)))
    }

    // --- Categories ----------------------------------------------------------

    /// Every category, the built-in one first.
    ///
    /// # Errors
    ///
    /// [`EranosError::Busy`] on wasm while another call holds the lock.
    pub fn categories(&self) -> Result<Vec<Category>, EranosError> {
        self.reading(|group| Ok(operations::categories(group)))
    }

    /// Every category, the built-in one first, as a future that waits for the
    /// lock.
    ///
    /// # Errors
    ///
    /// [`EranosError::Cancelled`].
    pub fn categories_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Vec<Category>, EranosError>> + Send + 'static {
        self.reading_async(cancellation, |group| Ok(operations::categories(group)))
    }

    /// The category with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn category(&self, id: &str) -> Result<Option<Category>, EranosError> {
        self.reading(|group| operations::category(group, id))
    }

    /// The category with the identifier `id`, as a future that waits for the
    /// lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn category_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Option<Category>, EranosError>> + Send + 'static {
        self.reading_async(cancellation, move |group| operations::category(group, &id))
    }

    /// Adds `category`, which has no identifier yet, and returns the one it
    /// gets.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn add_category(&self, category: &Category) -> Result<String, EranosError> {
        self.writing(|group| operations::add_category(group, category))
    }

    /// Adds `category`, which has no identifier yet, and returns the one it
    /// gets, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn add_category_async(
        &self,
        category: Category,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<String, EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::add_category(group, &category)
        })
    }

    /// Renames the category that `category` names by its identifier, and
    /// returns it as it was.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn replace_category(&self, category: &Category) -> Result<Category, EranosError> {
        self.writing(|group| operations::replace_category(group, category))
    }

    /// Renames the category that `category` names by its identifier, and
    /// returns it as it was, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn replace_category_async(
        &self,
        category: Category,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Category, EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::replace_category(group, &category)
        })
    }

    /// Removes the category with the identifier `id` and returns it; its
    /// expenses move to the built-in one.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn remove_category(&self, id: &str) -> Result<Category, EranosError> {
        self.writing(|group| operations::remove_category(group, id))
    }

    /// Removes the category with the identifier `id` and returns it; its
    /// expenses move to the built-in one, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn remove_category_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Category, EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::remove_category(group, &id)
        })
    }

    // --- Currency ------------------------------------------------------------

    /// The ISO 4217 code of the group currency.
    ///
    /// # Errors
    ///
    /// [`EranosError::Busy`] on wasm while another call holds the lock.
    pub fn currency(&self) -> Result<String, EranosError> {
        self.reading(|group| Ok(operations::currency_code(group)))
    }

    /// The ISO 4217 code of the group currency, as a future that waits for the
    /// lock.
    ///
    /// # Errors
    ///
    /// [`EranosError::Cancelled`].
    pub fn currency_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<String, EranosError>> + Send + 'static {
        self.reading_async(cancellation, |group| Ok(operations::currency_code(group)))
    }

    /// Moves the group to `currency`, with a rate into it for every other
    /// currency its entries use.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn change_currency(
        &self,
        currency: &str,
        rates: &[CurrencyRate],
    ) -> Result<(), EranosError> {
        self.writing(|group| operations::change_currency(group, currency, rates))
    }

    /// Moves the group to `currency`, with a rate into it for every other
    /// currency its entries use, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn change_currency_async(
        &self,
        currency: String,
        rates: Vec<CurrencyRate>,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<(), EranosError>> + Send + 'static {
        let shared = self.shared.clone();
        async move {
            let change = CurrencyChange::new(&currency, &rates)?;
            let mut group = cancellable(shared.write(), cancellation.clone()).await?;
            let changing = group.change_currency_async(change.currency, &change.rates);
            let changed = cancellable(changing, cancellation).await?;
            drop(group);
            Ok(changed?)
        }
    }

    // --- Expenses ------------------------------------------------------------

    /// Every expense, in the order they were created.
    ///
    /// # Errors
    ///
    /// [`EranosError::Busy`] on wasm while another call holds the lock.
    pub fn expenses(&self) -> Result<Vec<Expense>, EranosError> {
        self.reading(|group| Ok(operations::expenses(group)))
    }

    /// Every expense, in the order they were created, as a future that waits
    /// for the lock.
    ///
    /// # Errors
    ///
    /// [`EranosError::Cancelled`].
    pub fn expenses_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Vec<Expense>, EranosError>> + Send + 'static {
        self.reading_async(cancellation, |group| Ok(operations::expenses(group)))
    }

    /// The expense with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn expense(&self, id: &str) -> Result<Option<Expense>, EranosError> {
        self.reading(|group| operations::expense(group, id))
    }

    /// The expense with the identifier `id`, as a future that waits for the
    /// lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn expense_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Option<Expense>, EranosError>> + Send + 'static {
        self.reading_async(cancellation, move |group| operations::expense(group, &id))
    }

    /// How the expense with the identifier `id` divides, in the group currency.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn expense_shares(&self, id: &str) -> Result<Option<ExpenseShares>, EranosError> {
        self.reading(|group| operations::expense_shares(group, id))
    }

    /// How the expense with the identifier `id` divides, in the group currency,
    /// as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn expense_shares_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Option<ExpenseShares>, EranosError>> + Send + 'static {
        self.reading_async(cancellation, move |group| {
            operations::expense_shares(group, &id)
        })
    }

    /// Adds `expense`, which has no identifier yet, and returns the one it
    /// gets.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn add_expense(&self, expense: &Expense) -> Result<String, EranosError> {
        self.writing(|group| operations::add_expense(group, expense))
    }

    /// Adds `expense`, which has no identifier yet, and returns the one it
    /// gets, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn add_expense_async(
        &self,
        expense: Expense,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<String, EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::add_expense(group, &expense)
        })
    }

    /// Replaces the expense that `expense` names by its identifier, and returns
    /// it as it was.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn replace_expense(&self, expense: &Expense) -> Result<Expense, EranosError> {
        self.writing(|group| operations::replace_expense(group, expense))
    }

    /// Replaces the expense that `expense` names by its identifier, and returns
    /// it as it was, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn replace_expense_async(
        &self,
        expense: Expense,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Expense, EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::replace_expense(group, &expense)
        })
    }

    /// Removes the expense with the identifier `id` and returns it.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn remove_expense(&self, id: &str) -> Result<Expense, EranosError> {
        self.writing(|group| operations::remove_expense(group, id))
    }

    /// Removes the expense with the identifier `id` and returns it, as a future
    /// that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn remove_expense_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Expense, EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::remove_expense(group, &id)
        })
    }

    // --- Ledger --------------------------------------------------------------

    /// The group's totals.
    ///
    /// # Errors
    ///
    /// [`EranosError::Busy`] on wasm while another call holds the lock.
    pub fn summary(&self) -> Result<Summary, EranosError> {
        self.reading(|group| Ok(operations::summary(group)))
    }

    /// The group's totals, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// [`EranosError::Cancelled`].
    pub fn summary_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Summary, EranosError>> + Send + 'static {
        self.reading_async(cancellation, |group| Ok(operations::summary(group)))
    }

    // --- Members -------------------------------------------------------------

    /// Every member, archived ones included, in the order they were created.
    ///
    /// # Errors
    ///
    /// [`EranosError::Busy`] on wasm while another call holds the lock.
    pub fn members(&self) -> Result<Vec<Member>, EranosError> {
        self.reading(|group| Ok(operations::members(group)))
    }

    /// Every member, archived ones included, in the order they were created, as
    /// a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// [`EranosError::Cancelled`].
    pub fn members_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Vec<Member>, EranosError>> + Send + 'static {
        self.reading_async(cancellation, |group| Ok(operations::members(group)))
    }

    /// The member with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn member(&self, id: &str) -> Result<Option<Member>, EranosError> {
        self.reading(|group| operations::member(group, id))
    }

    /// The member with the identifier `id`, as a future that waits for the
    /// lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn member_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Option<Member>, EranosError>> + Send + 'static {
        self.reading_async(cancellation, move |group| operations::member(group, &id))
    }

    /// Whether the member with the identifier `id` is archived; `false` for an
    /// unknown member.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn is_archived(&self, id: &str) -> Result<bool, EranosError> {
        self.reading(|group| operations::is_archived(group, id))
    }

    /// Whether the member with the identifier `id` is archived; `false` for an
    /// unknown member, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn is_archived_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<bool, EranosError>> + Send + 'static {
        self.reading_async(cancellation, move |group| {
            operations::is_archived(group, &id)
        })
    }

    /// Adds `member`, who has no identifier yet, and returns the one they get.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn add_member(&self, member: &Member) -> Result<String, EranosError> {
        self.writing(|group| operations::add_member(group, member))
    }

    /// Adds `member`, who has no identifier yet, and returns the one they get,
    /// as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn add_member_async(
        &self,
        member: Member,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<String, EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::add_member(group, &member)
        })
    }

    /// Replaces the member that `member` names by their identifier, and returns
    /// them as they were.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn replace_member(&self, member: &Member) -> Result<Member, EranosError> {
        self.writing(|group| operations::replace_member(group, member))
    }

    /// Replaces the member that `member` names by their identifier, and returns
    /// them as they were, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn replace_member_async(
        &self,
        member: Member,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Member, EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::replace_member(group, &member)
        })
    }

    /// Archives the member with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn archive_member(&self, id: &str) -> Result<(), EranosError> {
        self.writing(|group| operations::archive_member(group, id))
    }

    /// Archives the member with the identifier `id`, as a future that waits for
    /// the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn archive_member_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<(), EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::archive_member(group, &id)
        })
    }

    /// Unarchives the member with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn unarchive_member(&self, id: &str) -> Result<(), EranosError> {
        self.writing(|group| operations::unarchive_member(group, id))
    }

    /// Unarchives the member with the identifier `id`, as a future that waits
    /// for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn unarchive_member_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<(), EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::unarchive_member(group, &id)
        })
    }

    /// Removes the member with the identifier `id`, whom no entry references,
    /// and returns them.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn remove_member(&self, id: &str) -> Result<Member, EranosError> {
        self.writing(|group| operations::remove_member(group, id))
    }

    /// Removes the member with the identifier `id`, whom no entry references,
    /// and returns them, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn remove_member_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Member, EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::remove_member(group, &id)
        })
    }

    // --- Name ----------------------------------------------------------------

    /// The group's name.
    ///
    /// # Errors
    ///
    /// [`EranosError::Busy`] on wasm while another call holds the lock.
    pub fn name(&self) -> Result<String, EranosError> {
        self.reading(|group| Ok(operations::name(group)))
    }

    /// The group's name, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// [`EranosError::Cancelled`].
    pub fn name_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<String, EranosError>> + Send + 'static {
        self.reading_async(cancellation, |group| Ok(operations::name(group)))
    }

    /// Renames the group.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn set_name(&self, name: &str) -> Result<(), EranosError> {
        self.writing(|group| operations::set_name(group, name))
    }

    /// Renames the group, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn set_name_async(
        &self,
        name: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<(), EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::set_name(group, &name)
        })
    }

    // --- Settlements ---------------------------------------------------------

    /// The payments that settle every balance by `strategy`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn settlement(&self, strategy: SettlementStrategy) -> Result<Settlement, EranosError> {
        self.reading(|group| operations::settlement(group, strategy))
    }

    /// The payments that settle every balance by `strategy`, as a future that
    /// waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn settlement_async(
        &self,
        strategy: SettlementStrategy,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Settlement, EranosError>> + Send + 'static {
        let shared = self.shared.clone();
        async move {
            let group = cancellable(shared.read(), cancellation.clone()).await?;
            let kept =
                cancellable(group.settlement_async(strategy.to_library()), cancellation).await??;
            let settlement = Settlement::from_library(kept);
            drop(group);
            Ok(settlement)
        }
    }

    // --- Transfers -----------------------------------------------------------

    /// Every transfer, in the order they were created.
    ///
    /// # Errors
    ///
    /// [`EranosError::Busy`] on wasm while another call holds the lock.
    pub fn transfers(&self) -> Result<Vec<Transfer>, EranosError> {
        self.reading(|group| Ok(operations::transfers(group)))
    }

    /// Every transfer, in the order they were created, as a future that waits
    /// for the lock.
    ///
    /// # Errors
    ///
    /// [`EranosError::Cancelled`].
    pub fn transfers_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Vec<Transfer>, EranosError>> + Send + 'static {
        self.reading_async(cancellation, |group| Ok(operations::transfers(group)))
    }

    /// The transfer with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn transfer(&self, id: &str) -> Result<Option<Transfer>, EranosError> {
        self.reading(|group| operations::transfer(group, id))
    }

    /// The transfer with the identifier `id`, as a future that waits for the
    /// lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn transfer_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Option<Transfer>, EranosError>> + Send + 'static {
        self.reading_async(cancellation, move |group| operations::transfer(group, &id))
    }

    /// The amount in the group currency of the transfer with the identifier
    /// `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn transfer_amount(&self, id: &str) -> Result<Option<Money>, EranosError> {
        self.reading(|group| operations::transfer_amount(group, id))
    }

    /// The amount in the group currency of the transfer with the identifier
    /// `id`, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn transfer_amount_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Option<Money>, EranosError>> + Send + 'static {
        self.reading_async(cancellation, move |group| {
            operations::transfer_amount(group, &id)
        })
    }

    /// Adds `transfer`, which has no identifier yet, and returns the one it
    /// gets.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn add_transfer(&self, transfer: &Transfer) -> Result<String, EranosError> {
        self.writing(|group| operations::add_transfer(group, transfer))
    }

    /// Adds `transfer`, which has no identifier yet, and returns the one it
    /// gets, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn add_transfer_async(
        &self,
        transfer: Transfer,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<String, EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::add_transfer(group, &transfer)
        })
    }

    /// Replaces the transfer that `transfer` names by its identifier, and
    /// returns it as it was.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn replace_transfer(&self, transfer: &Transfer) -> Result<Transfer, EranosError> {
        self.writing(|group| operations::replace_transfer(group, transfer))
    }

    /// Replaces the transfer that `transfer` names by its identifier, and
    /// returns it as it was, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn replace_transfer_async(
        &self,
        transfer: Transfer,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Transfer, EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::replace_transfer(group, &transfer)
        })
    }

    /// Removes the transfer with the identifier `id` and returns it.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn remove_transfer(&self, id: &str) -> Result<Transfer, EranosError> {
        self.writing(|group| operations::remove_transfer(group, id))
    }

    /// Removes the transfer with the identifier `id` and returns it, as a
    /// future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn remove_transfer_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Transfer, EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::remove_transfer(group, &id)
        })
    }

    // --- Description ---------------------------------------------------------

    /// The group's description.
    ///
    /// # Errors
    ///
    /// [`EranosError::Busy`] on wasm while another call holds the lock.
    pub fn description(&self) -> Result<Option<String>, EranosError> {
        self.reading(|group| Ok(operations::description(group)))
    }

    /// The group's description, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// [`EranosError::Cancelled`].
    pub fn description_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Option<String>, EranosError>> + Send + 'static {
        self.reading_async(cancellation, |group| Ok(operations::description(group)))
    }

    /// Sets the description; blank text clears it.
    ///
    /// # Errors
    ///
    /// [`EranosError::Busy`] on wasm while another call holds the lock.
    pub fn set_description(&self, description: &str) -> Result<(), EranosError> {
        self.writing(|group| {
            operations::set_description(group, description);
            Ok(())
        })
    }

    /// Sets the description; blank text clears it, as a future that waits for
    /// the lock.
    ///
    /// # Errors
    ///
    /// [`EranosError::Cancelled`].
    pub fn set_description_async(
        &self,
        description: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<(), EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::set_description(group, &description);
            Ok(())
        })
    }

    // --- Language ------------------------------------------------------------

    /// The language the apps show the group in, as a BCP 47 tag.
    ///
    /// # Errors
    ///
    /// [`EranosError::Busy`] on wasm while another call holds the lock.
    pub fn language(&self) -> Result<Option<String>, EranosError> {
        self.reading(|group| Ok(operations::language(group)))
    }

    /// The language the apps show the group in, as a BCP 47 tag, as a future
    /// that waits for the lock.
    ///
    /// # Errors
    ///
    /// [`EranosError::Cancelled`].
    pub fn language_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<Option<String>, EranosError>> + Send + 'static {
        self.reading_async(cancellation, |group| Ok(operations::language(group)))
    }

    /// Sets the language from a BCP 47 tag, or clears it with `None`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Busy`] on wasm while another
    /// call holds the lock.
    pub fn set_language(&self, language: Option<&str>) -> Result<(), EranosError> {
        self.writing(|group| operations::set_language(group, language))
    }

    /// Sets the language from a BCP 47 tag, or clears it with `None`, as a
    /// future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and [`EranosError::Cancelled`].
    pub fn set_language_async(
        &self,
        language: Option<String>,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<(), EranosError>> + Send + 'static {
        self.writing_async(cancellation, move |group| {
            operations::set_language(group, language.as_deref())
        })
    }

    // --- The saved form ------------------------------------------------------

    /// The group in its saved form, as JSON.
    ///
    /// # Errors
    ///
    /// [`EranosError::Busy`] on wasm while a change holds the lock.
    pub fn save(&self) -> Result<String, EranosError> {
        self.reading(|group| Ok(operations::save(group)))
    }

    /// The group in its saved form, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// [`EranosError::Cancelled`].
    pub fn save_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<String, EranosError>> + Send + 'static {
        self.reading_async(cancellation, |group| Ok(operations::save(group)))
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
    /// [`EranosError::Busy`] on wasm while a change holds the lock.
    pub fn read(&self) -> Result<GroupReader, EranosError> {
        self.read_lock().map(GroupReader::new)
    }

    /// A read handle, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// [`EranosError::Cancelled`].
    pub fn read_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> impl Future<Output = Result<GroupReader, EranosError>> + Send + 'static {
        let shared = self.shared.clone();
        async move {
            Ok(GroupReader::new(
                cancellable(shared.read(), cancellation).await?,
            ))
        }
    }

    // --- Locking -------------------------------------------------------------

    /// Runs `read` under the shared lock of a synchronous call.
    fn reading<T>(
        &self,
        read: impl FnOnce(&Group) -> Result<T, EranosError>,
    ) -> Result<T, EranosError> {
        self.read_lock().and_then(|group| read(&group))
    }

    /// Runs `write` under the exclusive lock of a synchronous call.
    fn writing<T>(
        &self,
        write: impl FnOnce(&mut Group) -> Result<T, EranosError>,
    ) -> Result<T, EranosError> {
        self.write_lock().and_then(|mut group| write(&mut group))
    }

    /// A future that runs `read` under the shared lock once it has it.
    fn reading_async<T: Send + 'static>(
        &self,
        cancellation: Option<Arc<Cancellation>>,
        read: impl FnOnce(&Group) -> Result<T, EranosError> + Send + 'static,
    ) -> impl Future<Output = Result<T, EranosError>> + Send + 'static {
        let shared = self.shared.clone();
        async move { read(&*cancellable(shared.read(), cancellation).await?) }
    }

    /// A future that runs `write` under the exclusive lock once it has it.
    fn writing_async<T: Send + 'static>(
        &self,
        cancellation: Option<Arc<Cancellation>>,
        write: impl FnOnce(&mut Group) -> Result<T, EranosError> + Send + 'static,
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
    fn read_lock(&self) -> Result<GroupReadGuard, EranosError> {
        Ok(self.shared.read_blocking())
    }

    /// The shared lock of a synchronous call: tried, since wasm's one thread
    /// must not wait.
    #[cfg(target_family = "wasm")]
    fn read_lock(&self) -> Result<GroupReadGuard, EranosError> {
        self.try_read_lock()
    }

    /// The exclusive lock of a synchronous call: waited for where a thread
    /// can sleep.
    #[cfg(not(target_family = "wasm"))]
    #[expect(
        clippy::unnecessary_wraps,
        reason = "on wasm the same call fails when the lock is taken"
    )]
    fn write_lock(&self) -> Result<GroupWriteGuard, EranosError> {
        Ok(self.shared.write_blocking())
    }

    /// The exclusive lock of a synchronous call: tried, since wasm's one
    /// thread must not wait.
    #[cfg(target_family = "wasm")]
    fn write_lock(&self) -> Result<GroupWriteGuard, EranosError> {
        self.try_write_lock()
    }

    /// The shared lock, taken without waiting, as a synchronous call takes
    /// it on wasm.
    #[cfg(any(test, target_family = "wasm"))]
    pub(crate) fn try_read_lock(&self) -> Result<GroupReadGuard, EranosError> {
        Ok(self.shared.try_read()?)
    }

    /// The exclusive lock, taken without waiting, as a synchronous call
    /// takes it on wasm.
    #[cfg(any(test, target_family = "wasm"))]
    pub(crate) fn try_write_lock(&self) -> Result<GroupWriteGuard, EranosError> {
        Ok(self.shared.try_write()?)
    }
}

impl From<Group> for GroupHandle {
    fn from(group: Group) -> Self {
        Self {
            shared: SharedGroup::new(group),
        }
    }
}

/// The saved form `json`, read but not yet loaded.
///
/// # Errors
///
/// [`EranosError::InvalidJson`] for text that is not JSON of the saved form.
fn saved(json: &str) -> Result<SavedGroup, EranosError> {
    serde_json::from_str(json).map_err(|error| EranosError::invalid_json(&error))
}

#[cfg(test)]
mod tests;
