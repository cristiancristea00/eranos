//! Every group call, once, over a locked group: the group handle runs each
//! one under the lock its synchronous or asynchronous form takes, and the
//! read handle runs the reads under the lock it holds.

use eranos::error::ExpenseGroupError;
use eranos::{CategoryId, ExpenseGroup, ExpenseId, Language, MemberId, TransferId};

use crate::expense::Expense;
use crate::ledger::{ExpenseShares, Summary};
use crate::member::{Category, Member};
use crate::money::{self, CurrencyRate, Money};
use crate::settlement::{SettlementPlan, SettlementStrategy};
use crate::transfer::Transfer;
use crate::{EranosError, identifier};

// --- Identifier --------------------------------------------------------------

/// The group's identifier.
pub(crate) fn id(group: &ExpenseGroup) -> String {
    group.id().to_string()
}

// --- Categories --------------------------------------------------------------

/// Every category: the built-in one first, then the others in identifier
/// order, which is creation order for generated identifiers.
pub(crate) fn categories(group: &ExpenseGroup) -> Vec<Category> {
    group.categories().map(Category::from_library).collect()
}

/// The category with the identifier `id`.
pub(crate) fn category(group: &ExpenseGroup, id: &str) -> Result<Option<Category>, EranosError> {
    let id: CategoryId = identifier(id)?;
    Ok(group.category(id).map(Category::from_library))
}

/// Adds `category`, returning its identifier.
pub(crate) fn add_category(
    group: &mut ExpenseGroup,
    category: &Category,
) -> Result<String, EranosError> {
    Ok(group.add_category(category.to_new()?)?.to_string())
}

/// Renames the category `category` names, returning it as it was.
pub(crate) fn replace_category(
    group: &mut ExpenseGroup,
    category: &Category,
) -> Result<Category, EranosError> {
    let id: CategoryId = identifier(&category.id)?;
    let existing = group
        .category(id)
        .ok_or(ExpenseGroupError::UnknownCategory(id))?;
    let replacement = category.replacing(existing)?;
    Ok(Category::from_library(
        &group.replace_category(replacement)?,
    ))
}

/// Removes the category with the identifier `id`, returning it.
pub(crate) fn remove_category(group: &mut ExpenseGroup, id: &str) -> Result<Category, EranosError> {
    let id: CategoryId = identifier(id)?;
    Ok(Category::from_library(&group.remove_category(id)?))
}

// --- Currency ----------------------------------------------------------------

/// The ISO 4217 code of the group currency.
pub(crate) fn currency(group: &ExpenseGroup) -> String {
    group.currency().code().to_owned()
}

/// A currency change, in the library's values.
pub(crate) struct CurrencyChange {
    /// The new group currency.
    pub(crate) currency: eranos::CurrencyCode,
    /// The rate into it of every other currency in use.
    pub(crate) rates: Vec<eranos::CurrencyRate>,
}

impl CurrencyChange {
    /// The change to the currency `code` at `rates`.
    pub(crate) fn new(code: &str, rates: &[CurrencyRate]) -> Result<Self, EranosError> {
        let rates = rates
            .iter()
            .map(CurrencyRate::to_library)
            .collect::<Result<Vec<_>, EranosError>>()?;
        Ok(Self {
            currency: money::currency(code)?,
            rates,
        })
    }
}

/// Moves the group to the currency `code`, with a rate for every other
/// currency its entries use.
pub(crate) fn change_currency(
    group: &mut ExpenseGroup,
    code: &str,
    rates: &[CurrencyRate],
) -> Result<(), EranosError> {
    let change = CurrencyChange::new(code, rates)?;
    Ok(group.change_currency(change.currency, &change.rates)?)
}

// --- Expenses ----------------------------------------------------------------

/// Every expense, in identifier order, which is creation order for
/// generated identifiers.
pub(crate) fn expenses(group: &ExpenseGroup) -> Vec<Expense> {
    group.expenses().map(Expense::from_library).collect()
}

/// The expense with the identifier `id`.
pub(crate) fn expense(group: &ExpenseGroup, id: &str) -> Result<Option<Expense>, EranosError> {
    let id: ExpenseId = identifier(id)?;
    Ok(group.expense(id).map(Expense::from_library))
}

/// How the expense with the identifier `id` divides.
pub(crate) fn expense_shares(
    group: &ExpenseGroup,
    id: &str,
) -> Result<Option<ExpenseShares>, EranosError> {
    let id: ExpenseId = identifier(id)?;
    Ok(group.expense_shares(id).map(ExpenseShares::from_library))
}

/// Adds `expense`, returning its identifier.
pub(crate) fn add_expense(
    group: &mut ExpenseGroup,
    expense: &Expense,
) -> Result<String, EranosError> {
    Ok(group.add_expense(expense.to_new()?)?.to_string())
}

/// Replaces the expense `expense` names, returning it as it was.
pub(crate) fn replace_expense(
    group: &mut ExpenseGroup,
    expense: &Expense,
) -> Result<Expense, EranosError> {
    let id: ExpenseId = identifier(&expense.id)?;
    let existing = group
        .expense(id)
        .ok_or(ExpenseGroupError::UnknownExpense(id))?;
    let replacement = expense.replacing(existing)?;
    Ok(Expense::from_library(&group.replace_expense(replacement)?))
}

/// Removes the expense with the identifier `id`, returning it.
pub(crate) fn remove_expense(group: &mut ExpenseGroup, id: &str) -> Result<Expense, EranosError> {
    let id: ExpenseId = identifier(id)?;
    Ok(Expense::from_library(&group.remove_expense(id)?))
}

// --- Ledger ------------------------------------------------------------------

/// The group's totals.
pub(crate) fn summary(group: &ExpenseGroup) -> Summary {
    Summary::from_library(group.summary())
}

// --- Members -----------------------------------------------------------------

/// Every member, archived ones included, in identifier order, which is
/// creation order for generated identifiers.
pub(crate) fn members(group: &ExpenseGroup) -> Vec<Member> {
    group.members().map(Member::from_library).collect()
}

/// The member with the identifier `id`.
pub(crate) fn member(group: &ExpenseGroup, id: &str) -> Result<Option<Member>, EranosError> {
    let id: MemberId = identifier(id)?;
    Ok(group.member(id).map(Member::from_library))
}

/// Whether the member with the identifier `id` is archived.
pub(crate) fn is_archived(group: &ExpenseGroup, id: &str) -> Result<bool, EranosError> {
    let id: MemberId = identifier(id)?;
    Ok(group.is_archived(id))
}

/// Adds `member`, returning their identifier.
pub(crate) fn add_member(group: &mut ExpenseGroup, member: &Member) -> Result<String, EranosError> {
    let member = member.to_new()?;
    group
        .add_member(member)
        .map(|id| id.to_string())
        .map_err(EranosError::from)
}

/// Replaces the member `member` names, returning them as they were.
pub(crate) fn replace_member(
    group: &mut ExpenseGroup,
    member: &Member,
) -> Result<Member, EranosError> {
    let id: MemberId = identifier(&member.id)?;
    let existing = group
        .member(id)
        .ok_or(ExpenseGroupError::UnknownMember(id))?;
    let replacement = member.replacing(existing)?;
    // The group refuses a replacement only for a member it does not hold,
    // which the lookup above rules out.
    group
        .replace_member(replacement)
        .map(|replaced| Member::from_library(&replaced))
        .map_err(EranosError::from)
}

/// Archives the member with the identifier `id`.
pub(crate) fn archive_member(group: &mut ExpenseGroup, id: &str) -> Result<(), EranosError> {
    let id: MemberId = identifier(id)?;
    Ok(group.archive_member(id)?)
}

/// Unarchives the member with the identifier `id`.
pub(crate) fn unarchive_member(group: &mut ExpenseGroup, id: &str) -> Result<(), EranosError> {
    let id: MemberId = identifier(id)?;
    Ok(group.unarchive_member(id)?)
}

/// Removes the member with the identifier `id`, returning them.
pub(crate) fn remove_member(group: &mut ExpenseGroup, id: &str) -> Result<Member, EranosError> {
    let id: MemberId = identifier(id)?;
    Ok(Member::from_library(&group.remove_member(id)?))
}

// --- Name --------------------------------------------------------------------

/// The group's name.
pub(crate) fn name(group: &ExpenseGroup) -> String {
    group.name().to_owned()
}

/// Renames the group.
pub(crate) fn set_name(group: &mut ExpenseGroup, name: &str) -> Result<(), EranosError> {
    Ok(group.set_name(name)?)
}

// --- Settlements -------------------------------------------------------------

/// The payments that settle every balance by `strategy`.
pub(crate) fn settlement(
    group: &ExpenseGroup,
    strategy: SettlementStrategy,
) -> Result<SettlementPlan, EranosError> {
    Ok(SettlementPlan::from_library(
        group.settlement(strategy.to_library())?,
    ))
}

// --- Transfers ---------------------------------------------------------------

/// Every transfer, in identifier order, which is creation order for
/// generated identifiers.
pub(crate) fn transfers(group: &ExpenseGroup) -> Vec<Transfer> {
    group.transfers().map(Transfer::from_library).collect()
}

/// The transfer with the identifier `id`.
pub(crate) fn transfer(group: &ExpenseGroup, id: &str) -> Result<Option<Transfer>, EranosError> {
    let id: TransferId = identifier(id)?;
    Ok(group.transfer(id).map(Transfer::from_library))
}

/// The amount in the group currency of the transfer with the identifier
/// `id`.
pub(crate) fn transfer_amount(
    group: &ExpenseGroup,
    id: &str,
) -> Result<Option<Money>, EranosError> {
    let id: TransferId = identifier(id)?;
    Ok(group.transfer_amount(id).map(Money::from_library))
}

/// Adds `transfer`, returning its identifier.
pub(crate) fn add_transfer(
    group: &mut ExpenseGroup,
    transfer: &Transfer,
) -> Result<String, EranosError> {
    Ok(group.add_transfer(transfer.to_new()?)?.to_string())
}

/// Replaces the transfer `transfer` names, returning it as it was.
pub(crate) fn replace_transfer(
    group: &mut ExpenseGroup,
    transfer: &Transfer,
) -> Result<Transfer, EranosError> {
    let id: TransferId = identifier(&transfer.id)?;
    let existing = group
        .transfer(id)
        .ok_or(ExpenseGroupError::UnknownTransfer(id))?;
    let replacement = transfer.replacing(existing)?;
    Ok(Transfer::from_library(
        &group.replace_transfer(replacement)?,
    ))
}

/// Removes the transfer with the identifier `id`, returning it.
pub(crate) fn remove_transfer(group: &mut ExpenseGroup, id: &str) -> Result<Transfer, EranosError> {
    let id: TransferId = identifier(id)?;
    Ok(Transfer::from_library(&group.remove_transfer(id)?))
}

// --- Description -------------------------------------------------------------

/// The group's description.
pub(crate) fn description(group: &ExpenseGroup) -> Option<String> {
    group.description().map(str::to_owned)
}

/// Sets the description; blank text clears it.
pub(crate) fn set_description(group: &mut ExpenseGroup, description: &str) {
    group.set_description(description);
}

// --- Language ----------------------------------------------------------------

/// The language the apps show the group in, as a BCP 47 tag.
pub(crate) fn language(group: &ExpenseGroup) -> Option<String> {
    group
        .language()
        .map(|language| language.as_str().to_owned())
}

/// Sets the language from a BCP 47 tag, or clears it.
pub(crate) fn set_language(
    group: &mut ExpenseGroup,
    language: Option<&str>,
) -> Result<(), EranosError> {
    let language = language.map(Language::parse).transpose()?;
    group.set_language(language);
    Ok(())
}

// --- The saved form ----------------------------------------------------------

/// The group in its saved form, as JSON indented by two spaces, one field to
/// a line: the text `serde_json::to_string_pretty` writes, which the Rust
/// examples print.
pub(crate) fn save(group: &ExpenseGroup) -> String {
    // A group serialises to strings, numbers, arrays, and objects with
    // string keys only, which cannot fail, so the fallback never applies.
    serde_json::to_string_pretty(group).unwrap_or_default()
}

#[cfg(test)]
mod tests;
