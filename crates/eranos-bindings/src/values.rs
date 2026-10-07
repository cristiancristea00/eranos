//! The value helpers: parsing, formatting, and the values an entry derives,
//! through the library's own rules, so that every text a binding shows is
//! the library's.

use std::collections::BTreeSet;

use eranos::error::MoneyError;
use eranos::expense as expenses;
use eranos::{Email, Iban, Language, MemberId, PhoneNumber, Username};
use eranos::{calendar, ledger, money as library};

use crate::calendar::{CalendarDate, TimeOfDay};
use crate::expense::{Expense, ExpenseSplit, Split};
use crate::ledger::Balance;
use crate::money::{CurrencyDetails, Money, currency, exchange_rate, parts, percentage};
use crate::{EranosError, identifier};

// --- Identifiers -------------------------------------------------------------

/// A new identifier, which any entry or group can take.
///
/// It is a UUID v7 as lowercase hyphenated text. Every kind of identifier
/// draws from the one generator of eranos-core, so the text is the same
/// whichever kind it stands for, and identifiers generated one after
/// another are in increasing order.
#[must_use]
pub fn identifier_generate() -> String {
    // A member's identifier stands for every kind, which all draw from the
    // same generator.
    MemberId::generate().as_str().to_owned()
}

/// The identifier written `text`, as it is: not trimmed, and in its case.
/// Every kind of identifier, of a group, a member, a category, an expense,
/// or a transfer, follows the same rules.
///
/// # Errors
///
/// An [`EranosError::Id`] naming the first rule the text breaks:
/// [`IdFailure::Empty`](crate::IdFailure::Empty),
/// [`IdFailure::TooLong`](crate::IdFailure::TooLong) for more than 64 bytes
/// of UTF-8,
/// [`IdFailure::ControlCharacter`](crate::IdFailure::ControlCharacter), and
/// [`IdFailure::SurroundingWhitespace`](crate::IdFailure::SurroundingWhitespace).
pub fn identifier_parse(text: &str) -> Result<String, EranosError> {
    // A member's identifier stands for every kind, whose rules are the same.
    Ok(identifier::<MemberId>(text)?.as_str().to_owned())
}

// --- Money -------------------------------------------------------------------

/// The amount written `text` in the currency `code`: "12.50".
///
/// # Errors
///
/// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency),
/// [`MoneyFailure::InvalidAmount`](crate::MoneyFailure::InvalidAmount), and
/// [`MoneyFailure::TooManyDecimals`](crate::MoneyFailure::TooManyDecimals).
pub fn money_parse(text: &str, code: &str) -> Result<Money, EranosError> {
    Ok(Money::from_library(library::Money::parse(
        text,
        currency(code)?,
    )?))
}

/// The amount as the library writes it: "12.50 EUR".
///
/// # Errors
///
/// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency).
pub fn money_format(money: &Money) -> Result<String, EranosError> {
    Ok(money.to_library()?.to_string())
}

/// `left` plus `right`, both in one currency.
///
/// # Errors
///
/// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency),
/// [`MoneyFailure::CurrencyMismatch`](crate::MoneyFailure::CurrencyMismatch),
/// and [`MoneyFailure::Overflow`](crate::MoneyFailure::Overflow).
pub fn money_add(left: &Money, right: &Money) -> Result<Money, EranosError> {
    Ok(Money::from_library(
        left.to_library()?.checked_add(right.to_library()?)?,
    ))
}

/// `left` minus `right`, both in one currency.
///
/// # Errors
///
/// As for [`money_add`].
pub fn money_subtract(left: &Money, right: &Money) -> Result<Money, EranosError> {
    Ok(Money::from_library(
        left.to_library()?.checked_sub(right.to_library()?)?,
    ))
}

// --- Exchange rates ----------------------------------------------------------

/// The exchange rate written `text`, as the library writes it back: "4.97".
///
/// # Errors
///
/// [`MoneyFailure::InvalidExchangeRate`](crate::MoneyFailure::InvalidExchangeRate).
pub fn exchange_rate_parse(text: &str) -> Result<String, EranosError> {
    Ok(exchange_rate(text)?.to_string())
}

/// `amount` converted at `rate` into the currency `code`, rounded to its
/// minor unit, halves away from zero.
///
/// # Errors
///
/// [`MoneyFailure::InvalidExchangeRate`](crate::MoneyFailure::InvalidExchangeRate),
/// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency), and
/// [`MoneyFailure::Overflow`](crate::MoneyFailure::Overflow).
pub fn exchange_rate_convert(rate: &str, amount: &Money, code: &str) -> Result<Money, EranosError> {
    Ok(Money::from_library(
        exchange_rate(rate)?.convert(amount.to_library()?, currency(code)?)?,
    ))
}

// --- Percentages and parts ---------------------------------------------------

/// The percentage written `text`, in basis points: "12.5" is 1 250.
///
/// # Errors
///
/// [`MoneyFailure::InvalidPercentage`](crate::MoneyFailure::InvalidPercentage).
pub fn percentage_parse(text: &str) -> Result<i32, EranosError> {
    let basis_points = library::Percentage::parse(text)?.basis_points();
    Ok(i32::try_from(basis_points).map_err(|_| MoneyError::InvalidPercentage)?)
}

/// The percentage of `basis_points`, as the library writes it: "12.5%".
///
/// # Errors
///
/// [`MoneyFailure::InvalidPercentage`](crate::MoneyFailure::InvalidPercentage)
/// for a negative number.
pub fn percentage_format(basis_points: i32) -> Result<String, EranosError> {
    Ok(percentage(basis_points)?.to_string())
}

/// The parts written `text`, in hundredths: "0.5" is 50.
///
/// # Errors
///
/// [`MoneyFailure::InvalidParts`](crate::MoneyFailure::InvalidParts).
pub fn parts_parse(text: &str) -> Result<i32, EranosError> {
    let hundredths = library::Parts::parse(text)?.hundredths();
    Ok(i32::try_from(hundredths).map_err(|_| MoneyError::InvalidParts)?)
}

/// The parts of `hundredths`, as the library writes them: "0.5".
///
/// # Errors
///
/// [`MoneyFailure::InvalidParts`](crate::MoneyFailure::InvalidParts) for a
/// number that is not positive.
pub fn parts_format(hundredths: i32) -> Result<String, EranosError> {
    Ok(parts(hundredths)?.to_string())
}

// --- Dates and times ---------------------------------------------------------

/// The date written `text` in ISO 8601: "2026-09-23".
///
/// # Errors
///
/// [`CalendarFailure::InvalidDate`](crate::CalendarFailure::InvalidDate).
pub fn calendar_date_parse(text: &str) -> Result<CalendarDate, EranosError> {
    Ok(CalendarDate::from_library(calendar::CalendarDate::parse(
        text,
    )?))
}

/// The date as the library writes it: "2026-09-23".
///
/// # Errors
///
/// [`CalendarFailure::InvalidDate`](crate::CalendarFailure::InvalidDate).
pub fn calendar_date_format(date: CalendarDate) -> Result<String, EranosError> {
    Ok(date.to_library()?.to_string())
}

/// The time of day written `text` in ISO 8601: "18:30".
///
/// # Errors
///
/// [`CalendarFailure::InvalidTime`](crate::CalendarFailure::InvalidTime).
pub fn time_of_day_parse(text: &str) -> Result<TimeOfDay, EranosError> {
    Ok(TimeOfDay::from_library(calendar::TimeOfDay::parse(text)?))
}

/// The time of day as the library writes it: "18:30".
///
/// # Errors
///
/// [`CalendarFailure::InvalidTime`](crate::CalendarFailure::InvalidTime).
pub fn time_of_day_format(time: TimeOfDay) -> Result<String, EranosError> {
    Ok(time.to_library()?.to_string())
}

// --- Languages ---------------------------------------------------------------

/// The language tag written `text`, in its canonical case: "pt-pt" is
/// "pt-PT".
///
/// # Errors
///
/// [`LanguageFailure::InvalidLanguage`](crate::LanguageFailure::InvalidLanguage).
pub fn language_parse(text: &str) -> Result<String, EranosError> {
    Ok(Language::parse(text)?.as_str().to_owned())
}

// --- Balances ----------------------------------------------------------------

/// The balance as the library writes it: "owes 12.00 EUR".
///
/// # Errors
///
/// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency).
pub fn balance_format(balance: &Balance) -> Result<String, EranosError> {
    let balance = match balance {
        Balance::GetsBack { amount } => ledger::Balance::GetsBack(amount.to_library()?),
        Balance::Owes { amount } => ledger::Balance::Owes(amount.to_library()?),
        Balance::Settled => ledger::Balance::Settled,
    };
    Ok(balance.to_string())
}

// --- Currencies --------------------------------------------------------------

/// What ISO 4217 says of the currency `code`.
///
/// # Errors
///
/// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency).
pub fn currency_details(code: &str) -> Result<CurrencyDetails, EranosError> {
    Ok(CurrencyDetails::from_library(currency(code)?))
}

/// Every currency with minor units, by code.
#[must_use]
pub fn currencies() -> Vec<CurrencyDetails> {
    library::CurrencyCode::all()
        .iter()
        .map(|&currency| CurrencyDetails::from_library(currency))
        .collect()
}

// --- Contact details ---------------------------------------------------------

/// The email address written `text`, without surrounding whitespace.
///
/// # Errors
///
/// [`ContactFailure::InvalidEmail`](crate::ContactFailure::InvalidEmail).
pub fn email_parse(text: &str) -> Result<String, EranosError> {
    Ok(Email::parse(text)?.as_str().to_owned())
}

/// The phone number written `text`, in E.164 form: "+40712345678".
///
/// # Errors
///
/// [`ContactFailure::InvalidPhoneNumber`](crate::ContactFailure::InvalidPhoneNumber).
pub fn phone_number_parse(text: &str) -> Result<String, EranosError> {
    Ok(PhoneNumber::parse(text)?.as_str().to_owned())
}

/// The phone number written `text`, in international form, grouped as its
/// country groups it: "+40 712 345 678".
///
/// # Errors
///
/// [`ContactFailure::InvalidPhoneNumber`](crate::ContactFailure::InvalidPhoneNumber).
pub fn phone_number_international(text: &str) -> Result<String, EranosError> {
    Ok(PhoneNumber::parse(text)?.international())
}

// --- Accounts ----------------------------------------------------------------

/// The IBAN written `text`, without spaces, in capitals.
///
/// # Errors
///
/// [`PaymentAccountFailure::InvalidIban`](crate::PaymentAccountFailure::InvalidIban), with
/// the reason.
pub fn iban_parse(text: &str) -> Result<String, EranosError> {
    Ok(Iban::parse(text)?.as_str().to_owned())
}

/// The country code of the IBAN written `text`: "RO".
///
/// # Errors
///
/// [`PaymentAccountFailure::InvalidIban`](crate::PaymentAccountFailure::InvalidIban), with
/// the reason.
pub fn iban_country_code(text: &str) -> Result<String, EranosError> {
    Ok(Iban::parse(text)?.country_code().to_owned())
}

/// The payment-app username written `text`, without surrounding
/// whitespace.
///
/// # Errors
///
/// [`PaymentAccountFailure::EmptyUsername`](crate::PaymentAccountFailure::EmptyUsername).
pub fn username_parse(text: &str) -> Result<String, EranosError> {
    Ok(Username::new(text)?.as_str().to_owned())
}

// --- What an expense derives -------------------------------------------------

/// The tip of `expense`, in its own currency.
///
/// # Errors
///
/// The errors of the cost's and the tip's conversions, and those of
/// [`TipRule::amount_on`](eranos::expense::TipRule::amount_on).
pub fn expense_tip_amount(expense: &Expense) -> Result<Money, EranosError> {
    Ok(Money::from_library(tip_amount(expense)?))
}

/// The total of `expense`, amount and tip, in its own currency: what its
/// payers paid.
///
/// # Errors
///
/// As for [`expense_tip_amount`], and
/// [`MoneyFailure::Overflow`](crate::MoneyFailure::Overflow).
pub fn expense_total(expense: &Expense) -> Result<Money, EranosError> {
    let amount = expense.cost.to_library()?.amount();
    Ok(Money::from_library(
        amount.checked_add(tip_amount(expense)?)?,
    ))
}

/// Every member `expense` involves, as a payer or a participant, as
/// identifiers in identifier order.
///
/// # Errors
///
/// The errors of the splits' and items' conversions.
pub fn expense_members(expense: &Expense) -> Result<Vec<String>, EranosError> {
    let payers = expense.payers.to_library()?;
    let split = match &expense.split {
        ExpenseSplit::Itemised { items } => expenses::ExpenseSplit::Itemised(
            items
                .iter()
                .map(crate::expense::ExpenseItem::to_library)
                .collect::<Result<Vec<_>, EranosError>>()?,
        ),
        ExpenseSplit::Whole { split } => expenses::ExpenseSplit::Whole(split.to_library()?),
    };
    let mut members: BTreeSet<MemberId> = payers.members().collect();
    members.extend(split.participants());
    Ok(members.iter().map(ToString::to_string).collect())
}

/// Every member `split` divides among, as identifiers in the order it lists
/// them.
///
/// # Errors
///
/// The errors of the split's conversion.
pub fn split_members(split: &Split) -> Result<Vec<String>, EranosError> {
    Ok(Split::members_of(&split.to_library()?))
}

/// The tip of `expense`, in the library's money.
fn tip_amount(expense: &Expense) -> Result<library::Money, EranosError> {
    let amount = expense.cost.to_library()?.amount();
    let tip = match &expense.tip {
        Some(tip) => tip.to_library()?,
        None => expenses::TipRule::default(),
    };
    Ok(tip.amount_on(amount)?)
}

#[cfg(test)]
mod tests;
