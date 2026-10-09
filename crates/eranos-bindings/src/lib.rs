//! What every binding of eranos exposes, in plain Rust: the records and
//! enums of values and entries, one error and its flat form for a binding
//! without enums that hold data, the group handle with a synchronous and an
//! asynchronous form of every group call, the read and write handles,
//! cancellation, the value helpers, and the saved form.
//!
//! The glue crates, `eranos-uniffi`, `eranos-web`, and `eranos-capi`, mirror
//! these types and forward every call here, converting nothing and deciding
//! nothing, so everything that can be wrong is in this crate, where Rust
//! tests reach it. Values cross as records of text and integers, which the
//! library's rules check on their way in, and the types stay within what
//! `uniffi` 0.31 can mirror: records, enums with data, one error enum, and
//! objects, with no methods on records and no optional list. What crosses,
//! and how each value is written, is in docs/BINDINGS.md § 2.
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

mod calendar;
mod calls;
mod cancellation;
mod error;
mod expense;
mod expense_group;
mod flat;
mod ledger;
mod member;
mod mirrors;
mod money;
mod reader;
mod settlement;
mod transfer;
mod values;
mod writer;

use std::str::FromStr;

use eranos::error::IdError;

pub use calendar::{CalendarDate, TimeOfDay};
pub use cancellation::Cancellation;
pub use error::{
    CalendarFailure, CategoryFailure, ContactFailure, EranosError, ExpenseFailure,
    ExpenseGroupFailure, IbanReason, IdFailure, LanguageFailure, MemberFailure, MismatchedField,
    MoneyFailure, PaymentAccountFailure, SettlementFailure, TransferFailure,
};
pub use expense::{
    Discount, DiscountScope, DiscountValue, Expense, ExpenseItem, ExpenseSplit, MemberAmount,
    MemberParts, MemberPercentage, Split, TipDistribution, TipRule, TipValue,
};
pub use expense_group::{ExpenseGroupHandle, NewExpenseGroup};
pub use flat::{ErrorKind, FailureKind, FlatError, IbanReasonKind, Payload};
pub use ledger::{
    Balance, CategoryTotal, ExpenseShares, MemberCategoryTotals, MemberShare, MemberSummary,
    Summary,
};
pub use member::{Category, Iban, Member, PaymentAccount, PaymentAccountDetails, PhoneNumber};
pub use money::{Cost, CurrencyDetails, CurrencyRate, Money};
pub use reader::ExpenseGroupReader;
pub use settlement::{Payment, SettlementPlan, SettlementStrategy};
pub use transfer::Transfer;
pub use values::{
    balance_format, calendar_date_check, calendar_date_format, calendar_date_parse,
    category_id_uncategorised, currencies, currency_details, email_parse, exchange_rate_convert,
    exchange_rate_from_f64, exchange_rate_parse, expense_item_discount_amount,
    expense_item_line_total, expense_item_line_total_before_discount, expense_members,
    expense_split_participants, expense_tip_amount, expense_total, iban_parse, identifier_generate,
    identifier_parse, language_parse, money_add, money_format, money_from_f64, money_parse,
    money_subtract, parts_check, parts_format, parts_from_f64, parts_parse, percentage_check,
    percentage_format, percentage_from_f64, percentage_parse, phone_number_parse, quantity_check,
    quantity_format, quantity_from_f64, quantity_from_whole, quantity_parse, split_members,
    time_of_day_check, time_of_day_format, time_of_day_parse, tip_rule_none, username_parse,
};
pub use writer::ExpenseGroupWriter;

/// The identifier `text`, as it is.
///
/// # Errors
///
/// An [`EranosError::Id`] for text that is not an identifier: empty, longer
/// than 64 bytes, with a control character, or with whitespace at either
/// end.
fn identifier<Id: FromStr<Err = IdError>>(text: &str) -> Result<Id, EranosError> {
    Ok(text.parse()?)
}

#[cfg(test)]
mod tests;
