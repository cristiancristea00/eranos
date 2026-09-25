//! The objects the bindings hand out: the group, its read handle, and the
//! cancellation of an asynchronous call, each wrapping the binding core's
//! type and forwarding every method to it.

use std::sync::Arc;

use eranos_bindings::{
    Category, CurrencyRate, EranosError, Expense, ExpenseShares, GroupHandle, Member, Money,
    NewGroup, Settlement, SettlementStrategy, Summary, Transfer,
};

/// A group, shared between the tasks and threads of a binding.
///
/// Every group call comes in two forms. The synchronous one waits for the
/// group's lock where a thread can sleep, and on wasm, whose one thread must
/// not, fails with `GroupFailure::Busy` while another call holds it. The
/// asynchronous one, named with `_async`, waits for the lock, runs the call,
/// and stops with `EranosError::Cancelled` when its `Cancellation` is
/// cancelled first. Reads share the lock and changes take it alone.
#[derive(Debug, uniffi::Object)]
pub(crate) struct Group {
    handle: GroupHandle,
}

#[uniffi::export]
impl Group {
    /// A new group with no members and no entries.
    ///
    /// # Errors
    ///
    /// `MoneyFailure::UnknownCurrency`,
    /// `GroupFailure::EmptyName`, and
    /// `LanguageFailure::InvalidLanguage`.
    #[uniffi::constructor]
    pub(crate) fn new(group: &NewGroup) -> Result<Arc<Self>, EranosError> {
        GroupHandle::new(group).map(Self::shared)
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
        GroupHandle::load(json).map(Self::shared)
    }

    /// The group's identifier.
    ///
    /// # Errors
    ///
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn id(&self) -> Result<String, EranosError> {
        self.handle.id()
    }

    /// The group's identifier, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// `EranosError::Cancelled`.
    pub(crate) async fn id_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<String, EranosError> {
        self.handle.id_async(shared(cancellation)).await
    }

    /// Every category, the built-in one first.
    ///
    /// # Errors
    ///
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn categories(&self) -> Result<Vec<Category>, EranosError> {
        self.handle.categories()
    }

    /// Every category, the built-in one first, as a future that waits for the
    /// lock.
    ///
    /// # Errors
    ///
    /// `EranosError::Cancelled`.
    pub(crate) async fn categories_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Vec<Category>, EranosError> {
        self.handle.categories_async(shared(cancellation)).await
    }

    /// The category with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn category(&self, id: &str) -> Result<Option<Category>, EranosError> {
        self.handle.category(id)
    }

    /// The category with the identifier `id`, as a future that waits for the
    /// lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn category_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Option<Category>, EranosError> {
        self.handle.category_async(id, shared(cancellation)).await
    }

    /// Adds `category`, which has no identifier yet, and returns the one it
    /// gets.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn add_category(&self, category: &Category) -> Result<String, EranosError> {
        self.handle.add_category(category)
    }

    /// Adds `category`, which has no identifier yet, and returns the one it
    /// gets, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn add_category_async(
        &self,
        category: Category,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<String, EranosError> {
        self.handle
            .add_category_async(category, shared(cancellation))
            .await
    }

    /// Renames the category that `category` names by its identifier, and
    /// returns it as it was.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn replace_category(&self, category: &Category) -> Result<Category, EranosError> {
        self.handle.replace_category(category)
    }

    /// Renames the category that `category` names by its identifier, and
    /// returns it as it was, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn replace_category_async(
        &self,
        category: Category,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Category, EranosError> {
        self.handle
            .replace_category_async(category, shared(cancellation))
            .await
    }

    /// Removes the category with the identifier `id` and returns it; its
    /// expenses move to the built-in one.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn remove_category(&self, id: &str) -> Result<Category, EranosError> {
        self.handle.remove_category(id)
    }

    /// Removes the category with the identifier `id` and returns it; its
    /// expenses move to the built-in one, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn remove_category_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Category, EranosError> {
        self.handle
            .remove_category_async(id, shared(cancellation))
            .await
    }

    /// The ISO 4217 code of the group currency.
    ///
    /// # Errors
    ///
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn currency(&self) -> Result<String, EranosError> {
        self.handle.currency()
    }

    /// The ISO 4217 code of the group currency, as a future that waits for the
    /// lock.
    ///
    /// # Errors
    ///
    /// `EranosError::Cancelled`.
    pub(crate) async fn currency_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<String, EranosError> {
        self.handle.currency_async(shared(cancellation)).await
    }

    /// Moves the group to `currency`, with a rate into it for every other
    /// currency its entries use.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn change_currency(
        &self,
        currency: &str,
        rates: &[CurrencyRate],
    ) -> Result<(), EranosError> {
        self.handle.change_currency(currency, rates)
    }

    /// Moves the group to `currency`, with a rate into it for every other
    /// currency its entries use, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn change_currency_async(
        &self,
        currency: String,
        rates: Vec<CurrencyRate>,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<(), EranosError> {
        self.handle
            .change_currency_async(currency, rates, shared(cancellation))
            .await
    }

    /// Every expense, in the order they were created.
    ///
    /// # Errors
    ///
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn expenses(&self) -> Result<Vec<Expense>, EranosError> {
        self.handle.expenses()
    }

    /// Every expense, in the order they were created, as a future that waits
    /// for the lock.
    ///
    /// # Errors
    ///
    /// `EranosError::Cancelled`.
    pub(crate) async fn expenses_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Vec<Expense>, EranosError> {
        self.handle.expenses_async(shared(cancellation)).await
    }

    /// The expense with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn expense(&self, id: &str) -> Result<Option<Expense>, EranosError> {
        self.handle.expense(id)
    }

    /// The expense with the identifier `id`, as a future that waits for the
    /// lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn expense_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Option<Expense>, EranosError> {
        self.handle.expense_async(id, shared(cancellation)).await
    }

    /// How the expense with the identifier `id` divides, in the group currency.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn expense_shares(&self, id: &str) -> Result<Option<ExpenseShares>, EranosError> {
        self.handle.expense_shares(id)
    }

    /// How the expense with the identifier `id` divides, in the group currency,
    /// as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn expense_shares_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Option<ExpenseShares>, EranosError> {
        self.handle
            .expense_shares_async(id, shared(cancellation))
            .await
    }

    /// Adds `expense`, which has no identifier yet, and returns the one it
    /// gets.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn add_expense(&self, expense: &Expense) -> Result<String, EranosError> {
        self.handle.add_expense(expense)
    }

    /// Adds `expense`, which has no identifier yet, and returns the one it
    /// gets, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn add_expense_async(
        &self,
        expense: Expense,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<String, EranosError> {
        self.handle
            .add_expense_async(expense, shared(cancellation))
            .await
    }

    /// Replaces the expense that `expense` names by its identifier, and returns
    /// it as it was.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn replace_expense(&self, expense: &Expense) -> Result<Expense, EranosError> {
        self.handle.replace_expense(expense)
    }

    /// Replaces the expense that `expense` names by its identifier, and returns
    /// it as it was, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn replace_expense_async(
        &self,
        expense: Expense,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Expense, EranosError> {
        self.handle
            .replace_expense_async(expense, shared(cancellation))
            .await
    }

    /// Removes the expense with the identifier `id` and returns it.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn remove_expense(&self, id: &str) -> Result<Expense, EranosError> {
        self.handle.remove_expense(id)
    }

    /// Removes the expense with the identifier `id` and returns it, as a future
    /// that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn remove_expense_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Expense, EranosError> {
        self.handle
            .remove_expense_async(id, shared(cancellation))
            .await
    }

    /// The group's totals.
    ///
    /// # Errors
    ///
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn summary(&self) -> Result<Summary, EranosError> {
        self.handle.summary()
    }

    /// The group's totals, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// `EranosError::Cancelled`.
    pub(crate) async fn summary_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Summary, EranosError> {
        self.handle.summary_async(shared(cancellation)).await
    }

    /// Every member, archived ones included, in the order they were created.
    ///
    /// # Errors
    ///
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn members(&self) -> Result<Vec<Member>, EranosError> {
        self.handle.members()
    }

    /// Every member, archived ones included, in the order they were created, as
    /// a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// `EranosError::Cancelled`.
    pub(crate) async fn members_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Vec<Member>, EranosError> {
        self.handle.members_async(shared(cancellation)).await
    }

    /// The member with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn member(&self, id: &str) -> Result<Option<Member>, EranosError> {
        self.handle.member(id)
    }

    /// The member with the identifier `id`, as a future that waits for the
    /// lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn member_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Option<Member>, EranosError> {
        self.handle.member_async(id, shared(cancellation)).await
    }

    /// Whether the member with the identifier `id` is archived; `false` for an
    /// unknown member.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn is_archived(&self, id: &str) -> Result<bool, EranosError> {
        self.handle.is_archived(id)
    }

    /// Whether the member with the identifier `id` is archived; `false` for an
    /// unknown member, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn is_archived_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<bool, EranosError> {
        self.handle
            .is_archived_async(id, shared(cancellation))
            .await
    }

    /// Adds `member`, who has no identifier yet, and returns the one they get.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn add_member(&self, member: &Member) -> Result<String, EranosError> {
        self.handle.add_member(member)
    }

    /// Adds `member`, who has no identifier yet, and returns the one they get,
    /// as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn add_member_async(
        &self,
        member: Member,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<String, EranosError> {
        self.handle
            .add_member_async(member, shared(cancellation))
            .await
    }

    /// Replaces the member that `member` names by their identifier, and returns
    /// them as they were.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn replace_member(&self, member: &Member) -> Result<Member, EranosError> {
        self.handle.replace_member(member)
    }

    /// Replaces the member that `member` names by their identifier, and returns
    /// them as they were, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn replace_member_async(
        &self,
        member: Member,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Member, EranosError> {
        self.handle
            .replace_member_async(member, shared(cancellation))
            .await
    }

    /// Archives the member with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn archive_member(&self, id: &str) -> Result<(), EranosError> {
        self.handle.archive_member(id)
    }

    /// Archives the member with the identifier `id`, as a future that waits for
    /// the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn archive_member_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<(), EranosError> {
        self.handle
            .archive_member_async(id, shared(cancellation))
            .await
    }

    /// Unarchives the member with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn unarchive_member(&self, id: &str) -> Result<(), EranosError> {
        self.handle.unarchive_member(id)
    }

    /// Unarchives the member with the identifier `id`, as a future that waits
    /// for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn unarchive_member_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<(), EranosError> {
        self.handle
            .unarchive_member_async(id, shared(cancellation))
            .await
    }

    /// Removes the member with the identifier `id`, whom no entry references,
    /// and returns them.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn remove_member(&self, id: &str) -> Result<Member, EranosError> {
        self.handle.remove_member(id)
    }

    /// Removes the member with the identifier `id`, whom no entry references,
    /// and returns them, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn remove_member_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Member, EranosError> {
        self.handle
            .remove_member_async(id, shared(cancellation))
            .await
    }

    /// The group's name.
    ///
    /// # Errors
    ///
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn name(&self) -> Result<String, EranosError> {
        self.handle.name()
    }

    /// The group's name, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// `EranosError::Cancelled`.
    pub(crate) async fn name_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<String, EranosError> {
        self.handle.name_async(shared(cancellation)).await
    }

    /// Renames the group.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn set_name(&self, name: &str) -> Result<(), EranosError> {
        self.handle.set_name(name)
    }

    /// Renames the group, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn set_name_async(
        &self,
        name: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<(), EranosError> {
        self.handle.set_name_async(name, shared(cancellation)).await
    }

    /// The payments that settle every balance by `strategy`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn settlement(
        &self,
        strategy: SettlementStrategy,
    ) -> Result<Settlement, EranosError> {
        self.handle.settlement(strategy)
    }

    /// The payments that settle every balance by `strategy`, as a future that
    /// waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn settlement_async(
        &self,
        strategy: SettlementStrategy,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Settlement, EranosError> {
        self.handle
            .settlement_async(strategy, shared(cancellation))
            .await
    }

    /// Every transfer, in the order they were created.
    ///
    /// # Errors
    ///
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn transfers(&self) -> Result<Vec<Transfer>, EranosError> {
        self.handle.transfers()
    }

    /// Every transfer, in the order they were created, as a future that waits
    /// for the lock.
    ///
    /// # Errors
    ///
    /// `EranosError::Cancelled`.
    pub(crate) async fn transfers_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Vec<Transfer>, EranosError> {
        self.handle.transfers_async(shared(cancellation)).await
    }

    /// The transfer with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn transfer(&self, id: &str) -> Result<Option<Transfer>, EranosError> {
        self.handle.transfer(id)
    }

    /// The transfer with the identifier `id`, as a future that waits for the
    /// lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn transfer_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Option<Transfer>, EranosError> {
        self.handle.transfer_async(id, shared(cancellation)).await
    }

    /// The amount in the group currency of the transfer with the identifier
    /// `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn transfer_amount(&self, id: &str) -> Result<Option<Money>, EranosError> {
        self.handle.transfer_amount(id)
    }

    /// The amount in the group currency of the transfer with the identifier
    /// `id`, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn transfer_amount_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Option<Money>, EranosError> {
        self.handle
            .transfer_amount_async(id, shared(cancellation))
            .await
    }

    /// Adds `transfer`, which has no identifier yet, and returns the one it
    /// gets.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn add_transfer(&self, transfer: &Transfer) -> Result<String, EranosError> {
        self.handle.add_transfer(transfer)
    }

    /// Adds `transfer`, which has no identifier yet, and returns the one it
    /// gets, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn add_transfer_async(
        &self,
        transfer: Transfer,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<String, EranosError> {
        self.handle
            .add_transfer_async(transfer, shared(cancellation))
            .await
    }

    /// Replaces the transfer that `transfer` names by its identifier, and
    /// returns it as it was.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn replace_transfer(&self, transfer: &Transfer) -> Result<Transfer, EranosError> {
        self.handle.replace_transfer(transfer)
    }

    /// Replaces the transfer that `transfer` names by its identifier, and
    /// returns it as it was, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn replace_transfer_async(
        &self,
        transfer: Transfer,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Transfer, EranosError> {
        self.handle
            .replace_transfer_async(transfer, shared(cancellation))
            .await
    }

    /// Removes the transfer with the identifier `id` and returns it.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn remove_transfer(&self, id: &str) -> Result<Transfer, EranosError> {
        self.handle.remove_transfer(id)
    }

    /// Removes the transfer with the identifier `id` and returns it, as a
    /// future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn remove_transfer_async(
        &self,
        id: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Transfer, EranosError> {
        self.handle
            .remove_transfer_async(id, shared(cancellation))
            .await
    }

    /// The group's description.
    ///
    /// # Errors
    ///
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn description(&self) -> Result<Option<String>, EranosError> {
        self.handle.description()
    }

    /// The group's description, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// `EranosError::Cancelled`.
    pub(crate) async fn description_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Option<String>, EranosError> {
        self.handle.description_async(shared(cancellation)).await
    }

    /// Sets the description; blank text clears it.
    ///
    /// # Errors
    ///
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn set_description(&self, description: &str) -> Result<(), EranosError> {
        self.handle.set_description(description)
    }

    /// Sets the description; blank text clears it, as a future that waits for
    /// the lock.
    ///
    /// # Errors
    ///
    /// `EranosError::Cancelled`.
    pub(crate) async fn set_description_async(
        &self,
        description: String,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<(), EranosError> {
        self.handle
            .set_description_async(description, shared(cancellation))
            .await
    }

    /// The language the apps show the group in, as a BCP 47 tag.
    ///
    /// # Errors
    ///
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    pub(crate) fn language(&self) -> Result<Option<String>, EranosError> {
        self.handle.language()
    }

    /// The language the apps show the group in, as a BCP 47 tag, as a future
    /// that waits for the lock.
    ///
    /// # Errors
    ///
    /// `EranosError::Cancelled`.
    pub(crate) async fn language_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Option<String>, EranosError> {
        self.handle.language_async(shared(cancellation)).await
    }

    /// Sets the language from a BCP 47 tag, or clears it with `None`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and
    /// `GroupFailure::Busy` on wasm while another
    /// call holds the lock.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "UniFFI lifts an optional text only as an owned value"
    )]
    pub(crate) fn set_language(&self, language: Option<String>) -> Result<(), EranosError> {
        self.handle.set_language(language.as_deref())
    }

    /// Sets the language from a BCP 47 tag, or clears it with `None`, as a
    /// future that waits for the lock.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Cancelled`.
    pub(crate) async fn set_language_async(
        &self,
        language: Option<String>,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<(), EranosError> {
        self.handle
            .set_language_async(language, shared(cancellation))
            .await
    }

    /// The group in its saved form, as JSON.
    ///
    /// # Errors
    ///
    /// `GroupFailure::Busy` on wasm while a change
    /// holds the lock.
    pub(crate) fn save(&self) -> Result<String, EranosError> {
        self.handle.save()
    }

    /// The group in its saved form, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// `EranosError::Cancelled`.
    pub(crate) async fn save_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<String, EranosError> {
        self.handle.save_async(shared(cancellation)).await
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
    /// `GroupFailure::Busy` on wasm while a change
    /// holds the lock.
    pub(crate) fn read(&self) -> Result<Arc<GroupReader>, EranosError> {
        self.handle.read().map(GroupReader::shared)
    }

    /// A read handle, as a future that waits for the lock.
    ///
    /// # Errors
    ///
    /// `EranosError::Cancelled`.
    pub(crate) async fn read_async(
        &self,
        cancellation: Option<Arc<Cancellation>>,
    ) -> Result<Arc<GroupReader>, EranosError> {
        self.handle
            .read_async(shared(cancellation))
            .await
            .map(GroupReader::shared)
    }
}

impl Group {
    /// The object of `handle`.
    fn shared(handle: GroupHandle) -> Arc<Self> {
        Arc::new(Self { handle })
    }
}

/// The group saved as `json`, loaded as `Group::load` does,
/// yielding while it replays the entries.
///
/// # Errors
///
/// As for `Group::load`, and `EranosError::Cancelled`.
#[uniffi::export]
pub(crate) async fn load_group_async(
    json: String,
    cancellation: Option<Arc<Cancellation>>,
) -> Result<Arc<Group>, EranosError> {
    GroupHandle::load_async(json, shared(cancellation))
        .await
        .map(Group::shared)
}

/// A group's reads under one shared lock, so that they see no change in
/// between: changes wait until the handle is released or freed.
#[derive(Debug, uniffi::Object)]
pub(crate) struct GroupReader {
    reader: eranos_bindings::GroupReader,
}

#[uniffi::export]
impl GroupReader {
    /// Releases the lock; releasing a released handle does nothing.
    pub(crate) fn release(&self) {
        self.reader.release();
    }

    /// The group's identifier.
    ///
    /// # Errors
    ///
    /// `EranosError::Released` after `release`.
    pub(crate) fn id(&self) -> Result<String, EranosError> {
        self.reader.id()
    }

    /// Every category, the built-in one first.
    ///
    /// # Errors
    ///
    /// `EranosError::Released` after `release`.
    pub(crate) fn categories(&self) -> Result<Vec<Category>, EranosError> {
        self.reader.categories()
    }

    /// The category with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Released` after
    /// `release`.
    pub(crate) fn category(&self, id: &str) -> Result<Option<Category>, EranosError> {
        self.reader.category(id)
    }

    /// The ISO 4217 code of the group currency.
    ///
    /// # Errors
    ///
    /// `EranosError::Released` after `release`.
    pub(crate) fn currency(&self) -> Result<String, EranosError> {
        self.reader.currency()
    }

    /// Every expense, in the order they were created.
    ///
    /// # Errors
    ///
    /// `EranosError::Released` after `release`.
    pub(crate) fn expenses(&self) -> Result<Vec<Expense>, EranosError> {
        self.reader.expenses()
    }

    /// The expense with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Released` after
    /// `release`.
    pub(crate) fn expense(&self, id: &str) -> Result<Option<Expense>, EranosError> {
        self.reader.expense(id)
    }

    /// How the expense with the identifier `id` divides, in the group currency.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Released` after
    /// `release`.
    pub(crate) fn expense_shares(&self, id: &str) -> Result<Option<ExpenseShares>, EranosError> {
        self.reader.expense_shares(id)
    }

    /// The group's totals.
    ///
    /// # Errors
    ///
    /// `EranosError::Released` after `release`.
    pub(crate) fn summary(&self) -> Result<Summary, EranosError> {
        self.reader.summary()
    }

    /// Every member, archived ones included, in the order they were created.
    ///
    /// # Errors
    ///
    /// `EranosError::Released` after `release`.
    pub(crate) fn members(&self) -> Result<Vec<Member>, EranosError> {
        self.reader.members()
    }

    /// The member with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Released` after
    /// `release`.
    pub(crate) fn member(&self, id: &str) -> Result<Option<Member>, EranosError> {
        self.reader.member(id)
    }

    /// Whether the member with the identifier `id` is archived; `false` for an
    /// unknown member.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Released` after
    /// `release`.
    pub(crate) fn is_archived(&self, id: &str) -> Result<bool, EranosError> {
        self.reader.is_archived(id)
    }

    /// The group's name.
    ///
    /// # Errors
    ///
    /// `EranosError::Released` after `release`.
    pub(crate) fn name(&self) -> Result<String, EranosError> {
        self.reader.name()
    }

    /// The payments that settle every balance by `strategy`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Released` after
    /// `release`.
    pub(crate) fn settlement(
        &self,
        strategy: SettlementStrategy,
    ) -> Result<Settlement, EranosError> {
        self.reader.settlement(strategy)
    }

    /// Every transfer, in the order they were created.
    ///
    /// # Errors
    ///
    /// `EranosError::Released` after `release`.
    pub(crate) fn transfers(&self) -> Result<Vec<Transfer>, EranosError> {
        self.reader.transfers()
    }

    /// The transfer with the identifier `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Released` after
    /// `release`.
    pub(crate) fn transfer(&self, id: &str) -> Result<Option<Transfer>, EranosError> {
        self.reader.transfer(id)
    }

    /// The amount in the group currency of the transfer with the identifier
    /// `id`.
    ///
    /// # Errors
    ///
    /// The library's refusal, and `EranosError::Released` after
    /// `release`.
    pub(crate) fn transfer_amount(&self, id: &str) -> Result<Option<Money>, EranosError> {
        self.reader.transfer_amount(id)
    }

    /// The group's description.
    ///
    /// # Errors
    ///
    /// `EranosError::Released` after `release`.
    pub(crate) fn description(&self) -> Result<Option<String>, EranosError> {
        self.reader.description()
    }

    /// The language the apps show the group in, as a BCP 47 tag.
    ///
    /// # Errors
    ///
    /// `EranosError::Released` after `release`.
    pub(crate) fn language(&self) -> Result<Option<String>, EranosError> {
        self.reader.language()
    }

    /// The group in its saved form, as JSON.
    ///
    /// # Errors
    ///
    /// `EranosError::Released` after `release`.
    pub(crate) fn save(&self) -> Result<String, EranosError> {
        self.reader.save()
    }
}

impl GroupReader {
    /// The object of `reader`.
    fn shared(reader: eranos_bindings::GroupReader) -> Arc<Self> {
        Arc::new(Self { reader })
    }
}

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

    /// Whether `cancel` was called.
    #[must_use]
    pub(crate) fn is_cancelled(&self) -> bool {
        self.shared.is_cancelled()
    }
}

/// The binding core's cancellation that `cancellation` holds.
fn shared(cancellation: Option<Arc<Cancellation>>) -> Option<Arc<eranos_bindings::Cancellation>> {
    Some(Arc::clone(&cancellation?.shared))
}
