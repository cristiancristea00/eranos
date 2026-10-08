//! The value helpers: parsing, formatting, and the values an entry derives,
//! through the library's own rules, so that every text a binding shows is
//! the library's.

use std::collections::BTreeSet;

use eranos::error::MoneyError;
use eranos::expense as expenses;
use eranos::member as members;
use eranos::{CategoryId, Email, Language, MemberId, Username};
use eranos::{calendar, money as library};
use eranos_calendar::{format_calendar_date, format_time_of_day};
use eranos_money::{
    format_basis_points, format_hundredths, format_minor_units, format_ten_thousandths,
};

use crate::calendar::{CalendarDate, TimeOfDay};
use crate::expense::{Expense, ExpenseItem, ExpenseSplit, Split, TipRule};
use crate::ledger::Balance;
use crate::member::{Iban, PhoneNumber};
use crate::money::{
    CurrencyDetails, Money, currency, exchange_rate, parts, percentage, quantity, ten_thousandths,
};
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

/// The identifier of the built-in category, Uncategorised, which every
/// group has and every expense is in until it is given another: the text of
/// the nil UUID, `00000000-0000-0000-0000-000000000000`.
#[must_use]
pub fn category_id_uncategorised() -> String {
    CategoryId::UNCATEGORISED.as_str().to_owned()
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

/// The amount of the double `value` in the currency `code`: 1.005 in "EUR"
/// is 1.01 EUR.
///
/// The library writes the double in its shortest decimal form, rounds it
/// half away from zero to the currency's decimal places, and parses it, so
/// every binding gets the same amount from the same double.
///
/// # Errors
///
/// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency),
/// [`MoneyFailure::NotFinite`](crate::MoneyFailure::NotFinite) for NaN and
/// the infinities, and
/// [`MoneyFailure::Overflow`](crate::MoneyFailure::Overflow) for an amount
/// that does not fit in 64-bit minor units.
pub fn money_from_f64(value: f64, code: &str) -> Result<Money, EranosError> {
    Ok(Money::from_library(library::Money::from_f64(
        value,
        currency(code)?,
    )?))
}

/// The amount as the library writes it: "12.50 EUR".
///
/// It writes the minor units with the decimal places the record's currency
/// details give, then the code, as the library's text form of an amount
/// does, so it cannot fail: for every number of decimal places, and for
/// details the library would refuse as well.
#[must_use]
pub fn money_format(money: &Money) -> String {
    format!(
        "{} {}",
        format_minor_units(money.minor_units, money.currency.decimal_places),
        money.currency.code
    )
}

/// `left` plus `right`, both in one currency.
///
/// # Errors
///
/// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency)
/// for an amount in a currency the library does not list, or whose details
/// are not the library's for its code,
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

/// The exchange rate of the double `value`, as the library writes it:
/// 4.9765 is "4.9765".
///
/// The library writes the double in its shortest decimal form, rounds it
/// half away from zero to 18 decimal places, and parses it, so the rate
/// keeps the digits of that form: 25.5 is "25.5".
///
/// # Errors
///
/// [`MoneyFailure::NotFinite`](crate::MoneyFailure::NotFinite) for NaN and
/// the infinities, and
/// [`MoneyFailure::InvalidExchangeRate`](crate::MoneyFailure::InvalidExchangeRate)
/// for a value that is not positive at 18 decimal places or has more
/// significant digits than fit in 64 bits.
pub fn exchange_rate_from_f64(value: f64) -> Result<String, EranosError> {
    Ok(library::ExchangeRate::from_f64(value)?.to_string())
}

/// `amount` converted at `rate` into the currency `code`, rounded to its
/// minor unit, halves away from zero.
///
/// # Errors
///
/// [`MoneyFailure::InvalidExchangeRate`](crate::MoneyFailure::InvalidExchangeRate),
/// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency)
/// for a code the library does not list and for an amount whose currency
/// details are not the library's for its code, and
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
    basis_points(library::Percentage::parse(text)?)
}

/// The percentage of the double `value`, a number of per cent, in basis
/// points: 12.345 is 1 235.
///
/// The library writes the double in its shortest decimal form, rounds it
/// half away from zero to two decimal places, and parses it: `0.1 + 0.2`,
/// which is 0.30000000000000004 as a double, is 30.
///
/// # Errors
///
/// [`MoneyFailure::NotFinite`](crate::MoneyFailure::NotFinite) for NaN and
/// the infinities, and
/// [`MoneyFailure::InvalidPercentage`](crate::MoneyFailure::InvalidPercentage)
/// for a negative value and one whose basis points do not fit an `i32`.
pub fn percentage_from_f64(value: f64) -> Result<i32, EranosError> {
    basis_points(library::Percentage::from_f64(value)?)
}

/// The percentage of `basis_points`, as the library writes it: "12.5%".
///
/// It writes the number through `eranos_money::format_basis_points`, which
/// the library's text form of a percentage writes with, so it cannot fail:
/// a negative number, which [`percentage_check`] refuses, is written the
/// same way after a minus sign, "-12.5%".
#[must_use]
pub fn percentage_format(basis_points: i32) -> String {
    format_basis_points(i64::from(basis_points)).to_string()
}

/// Checks that `basis_points` is a percentage, as an entry that holds one
/// requires.
///
/// # Errors
///
/// [`MoneyFailure::InvalidPercentage`](crate::MoneyFailure::InvalidPercentage)
/// for a negative number.
pub fn percentage_check(basis_points: i32) -> Result<(), EranosError> {
    percentage(basis_points)?;
    Ok(())
}

/// The parts written `text`, in hundredths: "0.5" is 50.
///
/// # Errors
///
/// [`MoneyFailure::InvalidParts`](crate::MoneyFailure::InvalidParts).
pub fn parts_parse(text: &str) -> Result<i32, EranosError> {
    hundredths(library::Parts::parse(text)?)
}

/// The parts of the double `value`, in hundredths: 1.005 is 101.
///
/// The library writes the double in its shortest decimal form, rounds it
/// half away from zero to two decimal places, and parses it.
///
/// # Errors
///
/// [`MoneyFailure::NotFinite`](crate::MoneyFailure::NotFinite) for NaN and
/// the infinities, and
/// [`MoneyFailure::InvalidParts`](crate::MoneyFailure::InvalidParts) for a
/// value that rounds to zero or below and one whose hundredths do not fit
/// an `i32`.
pub fn parts_from_f64(value: f64) -> Result<i32, EranosError> {
    hundredths(library::Parts::from_f64(value)?)
}

/// The parts of `hundredths`, as the library writes them: "0.5".
///
/// It writes the number through `eranos_money::format_hundredths`, which the
/// library's text form of parts writes with, so it cannot fail: zero and a
/// negative number, which [`parts_check`] refuses, are written the same way,
/// "0" and "-0.5".
#[must_use]
pub fn parts_format(hundredths: i32) -> String {
    format_hundredths(i64::from(hundredths)).to_string()
}

/// Checks that `hundredths` is a number of parts, as a split of parts
/// requires.
///
/// # Errors
///
/// [`MoneyFailure::InvalidParts`](crate::MoneyFailure::InvalidParts) for a
/// number that is not positive.
pub fn parts_check(hundredths: i32) -> Result<(), EranosError> {
    parts(hundredths)?;
    Ok(())
}

/// The basis points of `percentage`, as the bindings' 32-bit signed
/// integers hold them.
///
/// # Errors
///
/// [`MoneyFailure::InvalidPercentage`](crate::MoneyFailure::InvalidPercentage)
/// above 21 474 836.47 %, whose basis points do not fit an `i32`.
fn basis_points(percentage: library::Percentage) -> Result<i32, EranosError> {
    Ok(i32::try_from(percentage.basis_points()).map_err(|_| MoneyError::InvalidPercentage)?)
}

/// The hundredths of `parts`, as the bindings' 32-bit signed integers hold
/// them.
///
/// # Errors
///
/// [`MoneyFailure::InvalidParts`](crate::MoneyFailure::InvalidParts) above
/// 21 474 836.47 parts, whose hundredths do not fit an `i32`.
fn hundredths(parts: library::Parts) -> Result<i32, EranosError> {
    Ok(i32::try_from(parts.hundredths()).map_err(|_| MoneyError::InvalidParts)?)
}

// --- Quantities --------------------------------------------------------------

/// The quantity written `text`, in ten-thousandths: "0.452" is 4 520.
///
/// # Errors
///
/// [`MoneyFailure::InvalidQuantity`](crate::MoneyFailure::InvalidQuantity)
/// for text that is not a positive decimal with at most four decimal
/// places, and for a quantity whose ten-thousandths do not fit an `i64`.
pub fn quantity_parse(text: &str) -> Result<i64, EranosError> {
    Ok(ten_thousandths(library::Quantity::parse(text)?))
}

/// The quantity of `units` whole units, in ten-thousandths: 3 is 30 000.
///
/// # Errors
///
/// [`MoneyFailure::InvalidQuantity`](crate::MoneyFailure::InvalidQuantity)
/// for a number that is not positive, and for one whose ten-thousandths do
/// not fit an `i64`.
pub fn quantity_from_whole(units: i64) -> Result<i64, EranosError> {
    let units = u64::try_from(units).map_err(|_| MoneyError::InvalidQuantity)?;
    Ok(ten_thousandths(library::Quantity::from_whole(units)?))
}

/// The quantity of the double `value`, in ten-thousandths: 0.45205 is
/// 4 521.
///
/// The library writes the double in its shortest decimal form, rounds it
/// half away from zero to four decimal places, and parses it.
///
/// # Errors
///
/// [`MoneyFailure::NotFinite`](crate::MoneyFailure::NotFinite) for NaN and
/// the infinities, and
/// [`MoneyFailure::InvalidQuantity`](crate::MoneyFailure::InvalidQuantity)
/// for a value that rounds to zero or below and one whose ten-thousandths do
/// not fit an `i64`.
pub fn quantity_from_f64(value: f64) -> Result<i64, EranosError> {
    Ok(ten_thousandths(library::Quantity::from_f64(value)?))
}

/// The quantity of `ten_thousandths`, as the library writes it: "0.452".
///
/// It writes the number through `eranos_money::format_ten_thousandths`,
/// which the library's text form of a quantity writes with, so it cannot
/// fail: zero and a negative number, which [`quantity_check`] refuses, are
/// written the same way, "0" and "-0.452".
#[must_use]
pub fn quantity_format(ten_thousandths: i64) -> String {
    format_ten_thousandths(ten_thousandths).to_string()
}

/// Checks that `ten_thousandths` is a quantity, as an item requires.
///
/// # Errors
///
/// [`MoneyFailure::InvalidQuantity`](crate::MoneyFailure::InvalidQuantity)
/// for a number that is not positive.
pub fn quantity_check(ten_thousandths: i64) -> Result<(), EranosError> {
    quantity(ten_thousandths)?;
    Ok(())
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
/// It writes the record's fields through
/// `eranos_calendar::format_calendar_date`, which the library's text form
/// of a date writes with, so it cannot fail: the fields of a day the
/// calendar does not have, which [`calendar_date_check`] refuses, are
/// written the same way, "2026-02-30".
#[must_use]
pub fn calendar_date_format(date: CalendarDate) -> String {
    let CalendarDate { year, month, day } = date;
    format_calendar_date(year, month, day).to_string()
}

/// Checks that `date` is a day of the calendar, between years 1 and 9999,
/// as an entry's date must be.
///
/// # Errors
///
/// [`CalendarFailure::InvalidDate`](crate::CalendarFailure::InvalidDate).
pub fn calendar_date_check(date: CalendarDate) -> Result<(), EranosError> {
    date.to_library()?;
    Ok(())
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
/// It writes the record's fields through
/// `eranos_calendar::format_time_of_day`, which the library's text form of
/// a time writes with, so it cannot fail: the fields of a time the day does
/// not have, which [`time_of_day_check`] refuses, are written the same way,
/// "24:00".
#[must_use]
pub fn time_of_day_format(time: TimeOfDay) -> String {
    let TimeOfDay { hour, minute } = time;
    format_time_of_day(hour, minute).to_string()
}

/// Checks that `time` is a time of day, from 00:00 to 23:59, as an entry's
/// time must be.
///
/// # Errors
///
/// [`CalendarFailure::InvalidTime`](crate::CalendarFailure::InvalidTime).
pub fn time_of_day_check(time: TimeOfDay) -> Result<(), EranosError> {
    time.to_library()?;
    Ok(())
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

/// The balance as the library writes it: "owes 12.00 EUR", "settled", or
/// "gets back 12.00 EUR".
///
/// It writes the library's words around the amount as [`money_format`]
/// writes it, so it cannot fail; a test holds the words to the library's
/// text form of a balance.
#[must_use]
pub fn balance_format(balance: &Balance) -> String {
    match balance {
        Balance::Owes { amount } => format!("owes {}", money_format(amount)),
        Balance::Settled => "settled".to_owned(),
        Balance::GetsBack { amount } => format!("gets back {}", money_format(amount)),
    }
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

/// The phone number written `text`, in E.164 form, "+40712345678", and in
/// international form, grouped as its country groups it, "+40 712 345 678".
///
/// # Errors
///
/// [`ContactFailure::InvalidPhoneNumber`](crate::ContactFailure::InvalidPhoneNumber).
pub fn phone_number_parse(text: &str) -> Result<PhoneNumber, EranosError> {
    Ok(PhoneNumber::from_library(&members::PhoneNumber::parse(
        text,
    )?))
}

// --- Accounts ----------------------------------------------------------------

/// The IBAN written `text`, without spaces, in capitals, with its country
/// code: "RO".
///
/// # Errors
///
/// [`PaymentAccountFailure::InvalidIban`](crate::PaymentAccountFailure::InvalidIban), with
/// the reason.
pub fn iban_parse(text: &str) -> Result<Iban, EranosError> {
    Ok(Iban::from_library(&members::Iban::parse(text)?))
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

// --- Tips --------------------------------------------------------------------

/// The no-tip rule, which an expense without a tip has: 0 % of the amount,
/// shared proportionally.
#[must_use]
pub fn tip_rule_none() -> TipRule {
    TipRule::from_library(expenses::TipRule::NONE)
}

// --- What an expense derives -------------------------------------------------

/// The tip of `expense`, in its own currency.
///
/// # Errors
///
/// The errors of the cost's and the tip's conversions, among them
/// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency)
/// for an amount whose currency details are not the library's for its
/// code, and those of
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

/// The line total of `item` before its discount, as the library computes
/// it, so every binding shows the same one.
///
/// It is the unit price times the quantity, rounded half away from zero to
/// the currency's minor unit: zero when the product is below half a minor
/// unit.
///
/// # Errors
///
/// As for [`expense_item_line_total`].
pub fn expense_item_line_total_before_discount(item: &ExpenseItem) -> Result<Money, EranosError> {
    Ok(Money::from_library(
        item.to_library()?.line_total_before_discount(),
    ))
}

/// What the discount of `item` takes off its line, as the library computes
/// it: the line total before the discount minus the line total, zero
/// without a discount.
///
/// # Errors
///
/// As for [`expense_item_line_total`].
pub fn expense_item_discount_amount(item: &ExpenseItem) -> Result<Money, EranosError> {
    Ok(Money::from_library(item.to_library()?.discount_amount()))
}

/// The line total of `item`, as the library computes it, so every binding
/// shows the same line total.
///
/// It is the unit price times the quantity, rounded half away from zero to
/// the currency's minor unit, less the discount: zero for a free line,
/// whose product is below half a minor unit or whose discount takes the
/// whole price.
///
/// # Errors
///
/// The errors of the item's conversion: those of its unit price, split,
/// and discount, among them
/// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency)
/// for an amount whose currency details are not the library's for its
/// code,
/// [`MoneyFailure::InvalidQuantity`](crate::MoneyFailure::InvalidQuantity)
/// for a quantity that is not positive,
/// [`MoneyFailure::InvalidPercentage`](crate::MoneyFailure::InvalidPercentage)
/// for a percentage discount below zero, and the library's checks of an
/// item, among them a discount the line cannot take
/// ([`ExpenseFailure::NegativeDiscount`](crate::ExpenseFailure::NegativeDiscount),
/// [`ExpenseFailure::DiscountAboveHundredPercent`](crate::ExpenseFailure::DiscountAboveHundredPercent),
/// and
/// [`ExpenseFailure::DiscountAbovePrice`](crate::ExpenseFailure::DiscountAbovePrice)),
/// [`ExpenseFailure::AmountsDoNotAddUp`](crate::ExpenseFailure::AmountsDoNotAddUp)
/// for a split of amounts that does not add up to the line total, a free
/// line's included, and
/// [`ExpenseFailure::Overflow`](crate::ExpenseFailure::Overflow) for a line
/// total out of range.
pub fn expense_item_line_total(item: &ExpenseItem) -> Result<Money, EranosError> {
    Ok(Money::from_library(item.to_library()?.line_total()))
}

/// Every member `expense` involves, as a payer or a participant, as
/// identifiers in identifier order.
///
/// # Errors
///
/// The errors of the splits' and items' conversions, among them
/// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency)
/// for an amount whose currency details are not the library's for its
/// code.
pub fn expense_members(expense: &Expense) -> Result<Vec<String>, EranosError> {
    let payers = expense.payers.to_library()?;
    let split = match &expense.split {
        ExpenseSplit::Itemised { items } => expenses::ExpenseSplit::Itemised(
            items
                .iter()
                .map(ExpenseItem::to_library)
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
/// The errors of the split's conversion, among them
/// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency)
/// for an amount whose currency details are not the library's for its
/// code.
pub fn split_members(split: &Split) -> Result<Vec<String>, EranosError> {
    Ok(Split::members_of(&split.to_library()?))
}

/// The tip of `expense`, in the library's money.
fn tip_amount(expense: &Expense) -> Result<library::Money, EranosError> {
    let amount = expense.cost.to_library()?.amount();
    Ok(expense.tip.to_library()?.amount_on(amount)?)
}

#[cfg(test)]
mod tests;
