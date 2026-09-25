//! The value helpers, exported as functions: every text a binding shows
//! comes from the library through them.

use eranos_bindings::{
    Balance, CalendarDate, CurrencyDetails, EranosError, Expense, Money, Split, TimeOfDay,
};

/// The amount written `text` in the currency `code`: "12.50".
///
/// # Errors
///
/// `MoneyFailure::UnknownCurrency`,
/// `MoneyFailure::InvalidAmount`, and
/// `MoneyFailure::TooManyDecimals`.
#[uniffi::export]
pub(crate) fn money_parse(text: &str, code: &str) -> Result<Money, EranosError> {
    eranos_bindings::money_parse(text, code)
}

/// The amount as the library writes it: "12.50 EUR".
///
/// # Errors
///
/// `MoneyFailure::UnknownCurrency`.
#[uniffi::export]
pub(crate) fn money_format(money: &Money) -> Result<String, EranosError> {
    eranos_bindings::money_format(money)
}

/// `left` plus `right`, both in one currency.
///
/// # Errors
///
/// `MoneyFailure::UnknownCurrency`,
/// `MoneyFailure::CurrencyMismatch`,
/// and `MoneyFailure::Overflow`.
#[uniffi::export]
pub(crate) fn money_add(left: &Money, right: &Money) -> Result<Money, EranosError> {
    eranos_bindings::money_add(left, right)
}

/// `left` minus `right`, both in one currency.
///
/// # Errors
///
/// As for `money_add`.
#[uniffi::export]
pub(crate) fn money_subtract(left: &Money, right: &Money) -> Result<Money, EranosError> {
    eranos_bindings::money_subtract(left, right)
}

/// The exchange rate written `text`, as the library writes it back: "4.97".
///
/// # Errors
///
/// `MoneyFailure::InvalidExchangeRate`.
#[uniffi::export]
pub(crate) fn exchange_rate_parse(text: &str) -> Result<String, EranosError> {
    eranos_bindings::exchange_rate_parse(text)
}

/// `amount` converted at `rate` into the currency `code`, rounded to its
/// minor unit, halves away from zero.
///
/// # Errors
///
/// `MoneyFailure::InvalidExchangeRate`,
/// `MoneyFailure::UnknownCurrency`, and
/// `MoneyFailure::Overflow`.
#[uniffi::export]
pub(crate) fn exchange_rate_convert(
    rate: &str,
    amount: &Money,
    code: &str,
) -> Result<Money, EranosError> {
    eranos_bindings::exchange_rate_convert(rate, amount, code)
}

/// The percentage written `text`, in basis points: "12.5" is 1 250.
///
/// # Errors
///
/// `MoneyFailure::InvalidPercentage`.
#[uniffi::export]
pub(crate) fn percentage_parse(text: &str) -> Result<i32, EranosError> {
    eranos_bindings::percentage_parse(text)
}

/// The percentage of `basis_points`, as the library writes it: "12.5%".
///
/// # Errors
///
/// `MoneyFailure::InvalidPercentage`
/// for a negative number.
#[uniffi::export]
pub(crate) fn percentage_format(basis_points: i32) -> Result<String, EranosError> {
    eranos_bindings::percentage_format(basis_points)
}

/// The parts written `text`, in hundredths: "0.5" is 50.
///
/// # Errors
///
/// `MoneyFailure::InvalidParts`.
#[uniffi::export]
pub(crate) fn parts_parse(text: &str) -> Result<i32, EranosError> {
    eranos_bindings::parts_parse(text)
}

/// The parts of `hundredths`, as the library writes them: "0.5".
///
/// # Errors
///
/// `MoneyFailure::InvalidParts` for a
/// number that is not positive.
#[uniffi::export]
pub(crate) fn parts_format(hundredths: i32) -> Result<String, EranosError> {
    eranos_bindings::parts_format(hundredths)
}

/// The date written `text` in ISO 8601: "2026-09-23".
///
/// # Errors
///
/// `CalendarFailure::InvalidDate`.
#[uniffi::export]
pub(crate) fn calendar_date_parse(text: &str) -> Result<CalendarDate, EranosError> {
    eranos_bindings::calendar_date_parse(text)
}

/// The date as the library writes it: "2026-09-23".
///
/// # Errors
///
/// `CalendarFailure::InvalidDate`.
#[uniffi::export]
pub(crate) fn calendar_date_format(date: CalendarDate) -> Result<String, EranosError> {
    eranos_bindings::calendar_date_format(date)
}

/// The time of day written `text` in ISO 8601: "18:30".
///
/// # Errors
///
/// `CalendarFailure::InvalidTime`.
#[uniffi::export]
pub(crate) fn time_of_day_parse(text: &str) -> Result<TimeOfDay, EranosError> {
    eranos_bindings::time_of_day_parse(text)
}

/// The time of day as the library writes it: "18:30".
///
/// # Errors
///
/// `CalendarFailure::InvalidTime`.
#[uniffi::export]
pub(crate) fn time_of_day_format(time: TimeOfDay) -> Result<String, EranosError> {
    eranos_bindings::time_of_day_format(time)
}

/// The language tag written `text`, in its canonical case: "pt-pt" is
/// "pt-PT".
///
/// # Errors
///
/// `LanguageFailure::InvalidLanguage`.
#[uniffi::export]
pub(crate) fn language_parse(text: &str) -> Result<String, EranosError> {
    eranos_bindings::language_parse(text)
}

/// The balance as the library writes it: "owes 12.00 EUR".
///
/// # Errors
///
/// `MoneyFailure::UnknownCurrency`.
#[uniffi::export]
pub(crate) fn balance_format(balance: &Balance) -> Result<String, EranosError> {
    eranos_bindings::balance_format(balance)
}

/// What ISO 4217 says of the currency `code`.
///
/// # Errors
///
/// `MoneyFailure::UnknownCurrency`.
#[uniffi::export]
pub(crate) fn currency_details(code: &str) -> Result<CurrencyDetails, EranosError> {
    eranos_bindings::currency_details(code)
}

/// Every currency with minor units, by code.
#[must_use]
#[uniffi::export]
pub(crate) fn currencies() -> Vec<CurrencyDetails> {
    eranos_bindings::currencies()
}

/// The email address written `text`, without surrounding whitespace.
///
/// # Errors
///
/// `ContactFailure::InvalidEmail`.
#[uniffi::export]
pub(crate) fn email_parse(text: &str) -> Result<String, EranosError> {
    eranos_bindings::email_parse(text)
}

/// The phone number written `text`, in E.164 form: "+40712345678".
///
/// # Errors
///
/// `ContactFailure::InvalidPhoneNumber`.
#[uniffi::export]
pub(crate) fn phone_number_parse(text: &str) -> Result<String, EranosError> {
    eranos_bindings::phone_number_parse(text)
}

/// The phone number written `text`, in international form, grouped as its
/// country groups it: "+40 712 345 678".
///
/// # Errors
///
/// `ContactFailure::InvalidPhoneNumber`.
#[uniffi::export]
pub(crate) fn phone_number_international(text: &str) -> Result<String, EranosError> {
    eranos_bindings::phone_number_international(text)
}

/// The IBAN written `text`, without spaces, in capitals.
///
/// # Errors
///
/// `AccountFailure::InvalidIban`, with
/// the reason.
#[uniffi::export]
pub(crate) fn iban_parse(text: &str) -> Result<String, EranosError> {
    eranos_bindings::iban_parse(text)
}

/// The country code of the IBAN written `text`: "RO".
///
/// # Errors
///
/// `AccountFailure::InvalidIban`, with
/// the reason.
#[uniffi::export]
pub(crate) fn iban_country_code(text: &str) -> Result<String, EranosError> {
    eranos_bindings::iban_country_code(text)
}

/// The payment-app username written `text`, without surrounding
/// whitespace.
///
/// # Errors
///
/// `AccountFailure::EmptyUsername`.
#[uniffi::export]
pub(crate) fn username_parse(text: &str) -> Result<String, EranosError> {
    eranos_bindings::username_parse(text)
}

/// The tip of `expense`, in its own currency.
///
/// # Errors
///
/// The errors of the cost's and the tip's conversions, and those of
/// the library's `Tip::amount_on`.
#[uniffi::export]
pub(crate) fn expense_tip_amount(expense: &Expense) -> Result<Money, EranosError> {
    eranos_bindings::expense_tip_amount(expense)
}

/// The total of `expense`, amount and tip, in its own currency: what its
/// payers paid.
///
/// # Errors
///
/// As for `expense_tip_amount`, and
/// `MoneyFailure::Overflow`.
#[uniffi::export]
pub(crate) fn expense_total(expense: &Expense) -> Result<Money, EranosError> {
    eranos_bindings::expense_total(expense)
}

/// Every member `expense` involves, as a payer or a participant, as
/// identifiers in identifier order.
///
/// # Errors
///
/// The errors of the splits' and items' conversions.
#[uniffi::export]
pub(crate) fn expense_members(expense: &Expense) -> Result<Vec<String>, EranosError> {
    eranos_bindings::expense_members(expense)
}

/// Every member `split` divides among, as identifiers in the order it lists
/// them.
///
/// # Errors
///
/// The errors of the split's conversion.
#[uniffi::export]
pub(crate) fn split_members(split: &Split) -> Result<Vec<String>, EranosError> {
    eranos_bindings::split_members(split)
}
