//! Every group call, once. The trait below lists each call's name,
//! parameters, answer, lock, and documentation, and
//! `eranos_glue_macros::calls` writes from it the group handle's synchronous
//! and asynchronous form of every call, the read handle's form of every
//! read, and the exported macro `expense_group_calls!`, which hands the list to the
//! glue crates. Each call runs the operation of its name in `operations`
//! under the lock it names; a call marked `#[infallible]` runs one that
//! cannot fail, and `#[async_body]` names the hand-written asynchronous form
//! of a call that works before or while it holds the lock.

use crate::EranosError;
use crate::cancellation::Cancellation;
use crate::expense::Expense;
use crate::expense_group::{self, ExpenseGroupHandle};
use crate::ledger::{ExpenseShares, Summary};
use crate::member::{Category, Member};
use crate::money::{CurrencyRate, Money};
use crate::operations;
use crate::reader::ExpenseGroupReader;
use crate::settlement::{SettlementPlan, SettlementStrategy};
use crate::transfer::Transfer;

#[eranos_glue_macros::calls]
trait ExpenseGroupCalls {
    // --- Identifier ----------------------------------------------------------

    /// The group's identifier.
    #[read]
    #[infallible]
    fn id(&self) -> Result<String, EranosError>;

    // --- Categories ----------------------------------------------------------

    /// Every category: the built-in one first, then the others in identifier
    /// order, which is creation order for generated identifiers.
    #[read]
    #[infallible]
    fn categories(&self) -> Result<Vec<Category>, EranosError>;

    /// The category with the identifier `id`.
    #[read]
    fn category(&self, id: &str) -> Result<Option<Category>, EranosError>;

    /// Adds `category` under its identifier, or one the library generates
    /// when it has none, and returns the identifier.
    #[write]
    fn add_category(&self, category: &Category) -> Result<String, EranosError>;

    /// Renames the category that `category` names by its identifier, and
    /// returns it as it was.
    #[write]
    fn replace_category(&self, category: &Category) -> Result<Category, EranosError>;

    /// Removes the category with the identifier `id` and returns it; its
    /// expenses move to the built-in one.
    #[write]
    fn remove_category(&self, id: &str) -> Result<Category, EranosError>;

    // --- Currency ------------------------------------------------------------

    /// The ISO 4217 code of the group currency.
    #[read]
    #[infallible]
    fn currency(&self) -> Result<String, EranosError>;

    /// Moves the group to `currency`, with a rate into it for every other
    /// currency its entries use.
    #[write]
    #[async_body = expense_group::change_currency]
    fn change_currency(&self, currency: &str, rates: &[CurrencyRate]) -> Result<(), EranosError>;

    // --- Expenses ------------------------------------------------------------

    /// Every expense, in identifier order, which is creation order for
    /// generated identifiers.
    #[read]
    #[infallible]
    fn expenses(&self) -> Result<Vec<Expense>, EranosError>;

    /// The expense with the identifier `id`.
    #[read]
    fn expense(&self, id: &str) -> Result<Option<Expense>, EranosError>;

    /// How the expense with the identifier `id` divides, in the group currency.
    #[read]
    fn expense_shares(&self, id: &str) -> Result<Option<ExpenseShares>, EranosError>;

    /// Adds `expense` under its identifier, or one the library generates
    /// when it has none, and returns the identifier.
    #[write]
    fn add_expense(&self, expense: &Expense) -> Result<String, EranosError>;

    /// Replaces the expense that `expense` names by its identifier, and returns
    /// it as it was.
    #[write]
    fn replace_expense(&self, expense: &Expense) -> Result<Expense, EranosError>;

    /// Removes the expense with the identifier `id` and returns it.
    #[write]
    fn remove_expense(&self, id: &str) -> Result<Expense, EranosError>;

    // --- Ledger --------------------------------------------------------------

    /// The group's totals.
    #[read]
    #[infallible]
    fn summary(&self) -> Result<Summary, EranosError>;

    // --- Members -------------------------------------------------------------

    /// Every member, archived ones included, in identifier order, which is
    /// creation order for generated identifiers.
    #[read]
    #[infallible]
    fn members(&self) -> Result<Vec<Member>, EranosError>;

    /// The member with the identifier `id`.
    #[read]
    fn member(&self, id: &str) -> Result<Option<Member>, EranosError>;

    /// Whether the member with the identifier `id` is archived; `false` for an
    /// unknown member.
    #[read]
    fn is_archived(&self, id: &str) -> Result<bool, EranosError>;

    /// Adds `member` under their identifier, or one the library generates
    /// when they have none, and returns the identifier.
    #[write]
    fn add_member(&self, member: &Member) -> Result<String, EranosError>;

    /// Replaces the member that `member` names by their identifier, and returns
    /// them as they were.
    #[write]
    fn replace_member(&self, member: &Member) -> Result<Member, EranosError>;

    /// Archives the member with the identifier `id`.
    #[write]
    fn archive_member(&self, id: &str) -> Result<(), EranosError>;

    /// Unarchives the member with the identifier `id`.
    #[write]
    fn unarchive_member(&self, id: &str) -> Result<(), EranosError>;

    /// Removes the member with the identifier `id`, whom no entry references,
    /// and returns them.
    #[write]
    fn remove_member(&self, id: &str) -> Result<Member, EranosError>;

    // --- Name ----------------------------------------------------------------

    /// The group's name.
    #[read]
    #[infallible]
    fn name(&self) -> Result<String, EranosError>;

    /// Renames the group.
    #[write]
    fn set_name(&self, name: &str) -> Result<(), EranosError>;

    // --- Settlements ---------------------------------------------------------

    /// The payments that settle every balance by `strategy`.
    #[read]
    #[async_body = expense_group::settlement]
    fn settlement(&self, strategy: SettlementStrategy) -> Result<SettlementPlan, EranosError>;

    // --- Transfers -----------------------------------------------------------

    /// Every transfer, in identifier order, which is creation order for
    /// generated identifiers.
    #[read]
    #[infallible]
    fn transfers(&self) -> Result<Vec<Transfer>, EranosError>;

    /// The transfer with the identifier `id`.
    #[read]
    fn transfer(&self, id: &str) -> Result<Option<Transfer>, EranosError>;

    /// The amount in the group currency of the transfer with the identifier
    /// `id`.
    #[read]
    fn transfer_amount(&self, id: &str) -> Result<Option<Money>, EranosError>;

    /// Adds `transfer` under its identifier, or one the library generates
    /// when it has none, and returns the identifier.
    #[write]
    fn add_transfer(&self, transfer: &Transfer) -> Result<String, EranosError>;

    /// Replaces the transfer that `transfer` names by its identifier, and
    /// returns it as it was.
    #[write]
    fn replace_transfer(&self, transfer: &Transfer) -> Result<Transfer, EranosError>;

    /// Removes the transfer with the identifier `id` and returns it.
    #[write]
    fn remove_transfer(&self, id: &str) -> Result<Transfer, EranosError>;

    // --- Description ---------------------------------------------------------

    /// The group's description.
    #[read]
    #[infallible]
    fn description(&self) -> Result<Option<String>, EranosError>;

    /// Sets the description; blank text clears it.
    #[write]
    #[infallible]
    fn set_description(&self, description: &str) -> Result<(), EranosError>;

    // --- Language ------------------------------------------------------------

    /// The language the apps show the group in, as a BCP 47 tag.
    #[read]
    #[infallible]
    fn language(&self) -> Result<Option<String>, EranosError>;

    /// Sets the language from a BCP 47 tag, or clears it when there is none.
    #[write]
    fn set_language(&self, language: Option<&str>) -> Result<(), EranosError>;

    // --- The saved form ------------------------------------------------------

    /// The JSON of the group's saved form.
    #[read]
    #[infallible]
    fn save(&self) -> Result<String, EranosError>;
}
