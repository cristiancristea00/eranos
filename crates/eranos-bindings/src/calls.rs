//! Every group call, once: its name, parameters, answer, documentation,
//! and body. From the trait below, `eranos_glue_macros::calls` writes each
//! body as a function of this module, the group handle's synchronous and
//! asynchronous form of every call, the read handle's form of every read,
//! the write handle's form of every call, and the exported macro
//! `expense_group_calls!`, which hands the trait to the glue crates. A call
//! that takes the group as `&ExpenseGroup` reads it under the shared lock,
//! and one that takes `&mut ExpenseGroup` changes it under the exclusive
//! lock; a call that answers with a `Result` can fail, and any other cannot;
//! `#[async_body]` names the hand-written asynchronous form of a call that
//! yields while it holds the lock.

use eranos::error::ExpenseGroupError;
use eranos::{CategoryId, ExpenseGroup, ExpenseId, MemberId, TransferId};

use crate::cancellation::Cancellation;
use crate::expense::Expense;
use crate::expense_group::{self, ExpenseGroupHandle};
use crate::ledger::{ExpenseShares, Summary};
use crate::member::{Category, Member};
use crate::money::{CurrencyChange, CurrencyDetails, CurrencyRate, Money};
use crate::reader::ExpenseGroupReader;
use crate::settlement::{SettlementPlan, SettlementStrategy};
use crate::transfer::Transfer;
use crate::values::parsed_language;
use crate::writer::ExpenseGroupWriter;
use crate::{EranosError, identifier};

#[eranos_glue_macros::calls]
trait ExpenseGroupCalls {
    // --- Identifier ----------------------------------------------------------

    /// The group's identifier.
    fn id(group: &ExpenseGroup) -> String {
        group.id().to_string()
    }

    // --- Categories ----------------------------------------------------------

    /// Every category: the built-in one first, then the others in identifier
    /// order, which is creation order for generated identifiers.
    fn categories(group: &ExpenseGroup) -> Vec<Category> {
        group.categories().map(Category::from_library).collect()
    }

    /// The category with the identifier `id`.
    fn category(group: &ExpenseGroup, id: &str) -> Result<Option<Category>, EranosError> {
        let id: CategoryId = identifier(id)?;
        Ok(group.category(id).map(Category::from_library))
    }

    /// Adds `category` under its identifier, and returns the identifier.
    fn add_category(group: &mut ExpenseGroup, category: &Category) -> Result<String, EranosError> {
        Ok(group.add_category(category.to_new()?)?.to_string())
    }

    /// Renames the category that `category` names by its identifier, and
    /// returns it as it was.
    fn replace_category(
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

    /// Removes the category with the identifier `id` and returns it; its
    /// expenses move to the built-in one.
    fn remove_category(group: &mut ExpenseGroup, id: &str) -> Result<Category, EranosError> {
        let id: CategoryId = identifier(id)?;
        Ok(Category::from_library(&group.remove_category(id)?))
    }

    // --- Currency ------------------------------------------------------------

    /// The group currency, with what ISO 4217 says of it.
    fn currency(group: &ExpenseGroup) -> CurrencyDetails {
        CurrencyDetails::from_library(group.currency())
    }

    /// Moves the group to `currency`, with a rate into it for every other
    /// currency its entries use.
    #[async_body = expense_group::change_currency]
    fn change_currency(
        group: &mut ExpenseGroup,
        currency: &str,
        rates: &[CurrencyRate],
    ) -> Result<(), EranosError> {
        let change = CurrencyChange::new(currency, rates)?;
        Ok(group.change_currency(change.currency, &change.rates)?)
    }

    // --- Expenses ------------------------------------------------------------

    /// Every expense, in identifier order, which is creation order for
    /// generated identifiers.
    fn expenses(group: &ExpenseGroup) -> Vec<Expense> {
        group.expenses().map(Expense::from_library).collect()
    }

    /// The expense with the identifier `id`.
    fn expense(group: &ExpenseGroup, id: &str) -> Result<Option<Expense>, EranosError> {
        let id: ExpenseId = identifier(id)?;
        Ok(group.expense(id).map(Expense::from_library))
    }

    /// How the expense with the identifier `id` divides, in the group currency.
    fn expense_shares(
        group: &ExpenseGroup,
        id: &str,
    ) -> Result<Option<ExpenseShares>, EranosError> {
        let id: ExpenseId = identifier(id)?;
        Ok(group.expense_shares(id).map(ExpenseShares::from_library))
    }

    /// Adds `expense` under its identifier, and returns the identifier.
    ///
    /// An amount in a currency whose details are not the library's for its
    /// code is refused with
    /// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency).
    fn add_expense(group: &mut ExpenseGroup, expense: &Expense) -> Result<String, EranosError> {
        Ok(group.add_expense(expense.to_new()?)?.to_string())
    }

    /// Replaces the expense that `expense` names by its identifier, and returns
    /// it as it was.
    ///
    /// An amount in a currency whose details are not the library's for its
    /// code is refused with
    /// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency).
    fn replace_expense(
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

    /// Removes the expense with the identifier `id` and returns it.
    fn remove_expense(group: &mut ExpenseGroup, id: &str) -> Result<Expense, EranosError> {
        let id: ExpenseId = identifier(id)?;
        Ok(Expense::from_library(&group.remove_expense(id)?))
    }

    // --- Ledger --------------------------------------------------------------

    /// The group's totals.
    fn summary(group: &ExpenseGroup) -> Summary {
        Summary::from_library(group.summary())
    }

    // --- Members -------------------------------------------------------------

    /// Every member, archived ones included, in identifier order, which is
    /// creation order for generated identifiers.
    fn members(group: &ExpenseGroup) -> Vec<Member> {
        group.members().map(Member::from_library).collect()
    }

    /// The member with the identifier `id`.
    fn member(group: &ExpenseGroup, id: &str) -> Result<Option<Member>, EranosError> {
        let id: MemberId = identifier(id)?;
        Ok(group.member(id).map(Member::from_library))
    }

    /// Whether the member with the identifier `id` is archived.
    fn is_archived(group: &ExpenseGroup, id: &str) -> Result<Option<bool>, EranosError> {
        let id: MemberId = identifier(id)?;
        Ok(group.is_archived(id))
    }

    /// Adds `member` under their identifier, and returns the identifier.
    ///
    /// A phone number whose international form, or an IBAN whose country
    /// code, is not the one the library derives is refused with
    /// [`EranosError::MismatchedRecord`].
    fn add_member(group: &mut ExpenseGroup, member: &Member) -> Result<String, EranosError> {
        let member = member.to_new()?;
        group
            .add_member(member)
            .map(|id| id.to_string())
            .map_err(EranosError::from)
    }

    /// Replaces the member that `member` names by their identifier, and returns
    /// them as they were.
    ///
    /// A phone number whose international form, or an IBAN whose country
    /// code, is not the one the library derives is refused with
    /// [`EranosError::MismatchedRecord`].
    fn replace_member(group: &mut ExpenseGroup, member: &Member) -> Result<Member, EranosError> {
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
    fn archive_member(group: &mut ExpenseGroup, id: &str) -> Result<(), EranosError> {
        let id: MemberId = identifier(id)?;
        Ok(group.archive_member(id)?)
    }

    /// Unarchives the member with the identifier `id`.
    fn unarchive_member(group: &mut ExpenseGroup, id: &str) -> Result<(), EranosError> {
        let id: MemberId = identifier(id)?;
        Ok(group.unarchive_member(id)?)
    }

    /// Removes the member with the identifier `id`, whom no entry references,
    /// and returns them.
    fn remove_member(group: &mut ExpenseGroup, id: &str) -> Result<Member, EranosError> {
        let id: MemberId = identifier(id)?;
        Ok(Member::from_library(&group.remove_member(id)?))
    }

    // --- Name ----------------------------------------------------------------

    /// The group's name.
    fn name(group: &ExpenseGroup) -> String {
        group.name().to_owned()
    }

    /// Renames the group.
    fn set_name(group: &mut ExpenseGroup, name: &str) -> Result<(), EranosError> {
        Ok(group.set_name(name)?)
    }

    // --- Settlements ---------------------------------------------------------

    /// The payments that settle every balance by `strategy`.
    #[async_body = expense_group::settlement]
    fn settlement(
        group: &ExpenseGroup,
        strategy: SettlementStrategy,
    ) -> Result<SettlementPlan, EranosError> {
        Ok(SettlementPlan::from_library(
            group.settlement(strategy.to_library())?,
        ))
    }

    // --- Transfers -----------------------------------------------------------

    /// Every transfer, in identifier order, which is creation order for
    /// generated identifiers.
    fn transfers(group: &ExpenseGroup) -> Vec<Transfer> {
        group.transfers().map(Transfer::from_library).collect()
    }

    /// The transfer with the identifier `id`.
    fn transfer(group: &ExpenseGroup, id: &str) -> Result<Option<Transfer>, EranosError> {
        let id: TransferId = identifier(id)?;
        Ok(group.transfer(id).map(Transfer::from_library))
    }

    /// The amount in the group currency of the transfer with the identifier
    /// `id`.
    fn transfer_amount(group: &ExpenseGroup, id: &str) -> Result<Option<Money>, EranosError> {
        let id: TransferId = identifier(id)?;
        Ok(group.transfer_amount(id).map(Money::from_library))
    }

    /// Adds `transfer` under its identifier, and returns the identifier.
    ///
    /// An amount in a currency whose details are not the library's for its
    /// code is refused with
    /// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency).
    fn add_transfer(group: &mut ExpenseGroup, transfer: &Transfer) -> Result<String, EranosError> {
        Ok(group.add_transfer(transfer.to_new()?)?.to_string())
    }

    /// Replaces the transfer that `transfer` names by its identifier, and
    /// returns it as it was.
    ///
    /// An amount in a currency whose details are not the library's for its
    /// code is refused with
    /// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency).
    fn replace_transfer(
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

    /// Removes the transfer with the identifier `id` and returns it.
    fn remove_transfer(group: &mut ExpenseGroup, id: &str) -> Result<Transfer, EranosError> {
        let id: TransferId = identifier(id)?;
        Ok(Transfer::from_library(&group.remove_transfer(id)?))
    }

    // --- Note ----------------------------------------------------------------

    /// The group's note.
    fn note(group: &ExpenseGroup) -> Option<String> {
        group.note().map(str::to_owned)
    }

    /// Sets the note, trimmed, or clears it when there is none or the text is
    /// blank.
    fn set_note(group: &mut ExpenseGroup, note: Option<&str>) {
        match note {
            Some(note) => group.set_note(note),
            None => group.clear_note(),
        }
    }

    // --- Language ------------------------------------------------------------

    /// The language the apps show the group in, as a BCP 47 tag.
    fn language(group: &ExpenseGroup) -> Option<String> {
        group
            .language()
            .map(|language| language.as_str().to_owned())
    }

    /// Sets the language from a BCP 47 tag, or clears it when there is none
    /// or the text is blank.
    fn set_language(group: &mut ExpenseGroup, language: Option<&str>) -> Result<(), EranosError> {
        group.set_language(parsed_language(language)?);
        Ok(())
    }

    // --- The saved form ------------------------------------------------------

    /// The JSON of the group's saved form.
    fn save(group: &ExpenseGroup) -> String {
        // A group serialises to strings, numbers, arrays, and objects with
        // string keys only, which cannot fail, so the fallback never applies.
        serde_json::to_string_pretty(group).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests;
