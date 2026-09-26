//! What every binding of eranos exposes, in plain Rust: the records and
//! enums of values and entries, one error and its flat form for a binding
//! without enums that hold data, the group handle with a synchronous and an
//! asynchronous form of every group call, the read handle, cancellation, the
//! value helpers, and the saved form.
//!
//! The glue crates (built with `uniffi` for Swift, Kotlin, Java, KMP, and the
//! web, and with `diplomat` for C and C++) mirror these types and forward
//! every call here, converting nothing and deciding nothing, so everything
//! that can be wrong is in this crate, where Rust tests reach it. Values
//! cross as records of text and integers: identifiers as hyphenated UUIDs,
//! currencies as ISO 4217 codes, exchange rates as decimals, amounts as
//! 64-bit minor units, percentages in basis points, and parts in hundredths.
//! Every value is checked by the library's own rules on its way in, so a bad
//! one is a typed [`EranosError`], and an entry to add carries no identifier:
//! the group gives it one.
//!
//! The types stay within what `uniffi` 0.29 can mirror: records, enums with
//! data, one error enum, and objects; no methods on records, and no
//! optional list, which is an empty one instead.
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

mod calendar;
mod calls;
mod cancellation;
mod error;
mod expense;
mod flat;
mod group;
mod ledger;
mod member;
mod mirrors;
mod money;
mod operations;
mod reader;
mod settlement;
mod transfer;
mod values;

use std::str::FromStr;

use eranos::error::IdError;

pub use calendar::{CalendarDate, TimeOfDay};
pub use cancellation::Cancellation;
pub use error::{
    AccountFailure, CalendarFailure, CategoryFailure, ContactFailure, EranosError, ExpenseFailure,
    GroupFailure, IbanReason, IdFailure, LanguageFailure, MemberFailure, MoneyFailure,
    SettlementFailure, TransferFailure,
};
pub use expense::{
    Expense, ExpenseSplit, Item, MemberAmount, MemberParts, MemberPercentage, Split, Tip,
    TipDistribution, TipValue,
};
pub use flat::{ErrorKind, FailureKind, FlatError, IbanReasonKind, Payload};
pub use group::{GroupHandle, NewGroup};
pub use ledger::{
    Balance, CategoryTotal, ExpenseShares, MemberCategoryTotals, MemberShare, MemberSummary,
    Summary,
};
pub use member::{Account, AccountDetails, Category, Member};
pub use money::{Cost, CurrencyDetails, CurrencyRate, Money};
pub use reader::GroupReader;
pub use settlement::{Payment, Settlement, SettlementStrategy};
pub use transfer::Transfer;
pub use values::{
    balance_format, calendar_date_format, calendar_date_parse, currencies, currency_details,
    email_parse, exchange_rate_convert, exchange_rate_parse, expense_members, expense_tip_amount,
    expense_total, iban_country_code, iban_parse, language_parse, money_add, money_format,
    money_parse, money_subtract, parts_format, parts_parse, percentage_format, percentage_parse,
    phone_number_international, phone_number_parse, split_members, time_of_day_format,
    time_of_day_parse, username_parse,
};

/// The identifier written `text`, a hyphenated UUID.
///
/// # Errors
///
/// [`IdFailure::InvalidId`] for text that is not a
/// UUID.
fn identifier<Id: FromStr<Err = IdError>>(text: &str) -> Result<Id, EranosError> {
    Ok(text.parse()?)
}

/// Refuses a record to add whose identifier `id` is present: the group
/// gives a new entry its own.
///
/// # Errors
///
/// [`EranosError::IdentifierGiven`] when `id` is present.
fn check_new(id: Option<&str>) -> Result<(), EranosError> {
    if id.is_some() {
        return Err(EranosError::identifier_given());
    }
    Ok(())
}

/// The identifier `id` of a record to replace, which names the entry it
/// replaces.
///
/// # Errors
///
/// [`EranosError::IdentifierMissing`] when `id` is absent, and
/// [`IdFailure::InvalidId`] for text that is not a UUID.
fn identifier_to_replace<Id: FromStr<Err = IdError>>(id: Option<&str>) -> Result<Id, EranosError> {
    identifier(id.ok_or_else(EranosError::identifier_missing)?)
}

#[cfg(test)]
mod tests;
