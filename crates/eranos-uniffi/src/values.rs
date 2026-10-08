//! The value helpers, exported as functions: every text a binding shows
//! comes from the library through them.

use eranos_bindings::{
    Balance, CalendarDate, CurrencyDetails, EranosError, Expense, Iban, Money, PhoneNumber,
    TimeOfDay, TipRule,
};

/// A new identifier, which any entry or group can take.
///
/// It is a UUID v7 as lowercase hyphenated text. Every kind of identifier,
/// of a group, a member, a category, an expense, or a transfer, draws from
/// the one generator of the library.
#[uniffi::export]
pub(crate) fn identifier_generate() -> String {
    eranos_bindings::identifier_generate()
}

/// The identifier written `text`, as it is: not trimmed, and in its case.
/// Every kind of identifier, of a group, a member, a category, an expense,
/// or a transfer, follows the same rules.
///
/// # Errors
///
/// `IdFailure::Empty`, `IdFailure::TooLong` for more than 64 bytes of
/// UTF-8, `IdFailure::ControlCharacter`, and
/// `IdFailure::SurroundingWhitespace`: the first rule the text breaks.
#[uniffi::export]
pub(crate) fn identifier_parse(text: &str) -> Result<String, EranosError> {
    eranos_bindings::identifier_parse(text)
}

/// The identifier of the built-in category, Uncategorised, which every
/// group has and every expense is in until it is given another: the text of
/// the nil UUID, `00000000-0000-0000-0000-000000000000`.
#[must_use]
#[uniffi::export]
pub(crate) fn category_id_uncategorised() -> String {
    eranos_bindings::category_id_uncategorised()
}

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

/// The amount of the double `value` in the currency `code`: 1.005 in "EUR"
/// is 1.01 EUR.
///
/// The library writes the double in its shortest decimal form, rounds it
/// half away from zero to the currency's decimal places, and parses it, so
/// every binding gets the same amount from the same double.
///
/// # Errors
///
/// `MoneyFailure::UnknownCurrency`,
/// `MoneyFailure::NotFinite` for NaN and the infinities, and
/// `MoneyFailure::Overflow` for an amount that does not fit in 64-bit minor
/// units.
#[uniffi::export]
pub(crate) fn money_from_f64(value: f64, code: &str) -> Result<Money, EranosError> {
    eranos_bindings::money_from_f64(value, code)
}

/// The amount as the library writes it: "12.50 EUR".
///
/// It writes the minor units with the decimal places the record's currency
/// details give, then the code, as the library's text form of an amount
/// does, so it cannot fail.
#[must_use]
#[uniffi::export]
pub(crate) fn money_format(money: &Money) -> String {
    eranos_bindings::money_format(money)
}

/// `left` plus `right`, both in one currency.
///
/// # Errors
///
/// `MoneyFailure::UnknownCurrency` for an amount in a currency the library
/// does not list, or whose details are not the library's for its code,
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

/// The exchange rate of the double `value`, as the library writes it:
/// 4.9765 is "4.9765".
///
/// The library writes the double in its shortest decimal form, rounds it
/// half away from zero to 18 decimal places, and parses it, so the rate
/// keeps the digits of that form: 25.5 is "25.5".
///
/// # Errors
///
/// `MoneyFailure::NotFinite` for NaN and the infinities, and
/// `MoneyFailure::InvalidExchangeRate` for a value that is not positive at
/// 18 decimal places or has more significant digits than fit in 64 bits.
#[uniffi::export]
pub(crate) fn exchange_rate_from_f64(value: f64) -> Result<String, EranosError> {
    eranos_bindings::exchange_rate_from_f64(value)
}

/// `amount` converted at `rate` into the currency `code`, rounded to its
/// minor unit, halves away from zero.
///
/// # Errors
///
/// `MoneyFailure::InvalidExchangeRate`,
/// `MoneyFailure::UnknownCurrency` for a code the library does not list and
/// for an amount whose currency details are not the library's for its
/// code, and
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

/// The percentage of the double `value`, a number of per cent, in basis
/// points: 12.345 is 1 235.
///
/// The library writes the double in its shortest decimal form, rounds it
/// half away from zero to two decimal places, and parses it: `0.1 + 0.2`,
/// which is 0.30000000000000004 as a double, is 30.
///
/// # Errors
///
/// `MoneyFailure::NotFinite` for NaN and the infinities, and
/// `MoneyFailure::InvalidPercentage` for a negative value and one whose
/// basis points do not fit an `i32`.
#[uniffi::export]
pub(crate) fn percentage_from_f64(value: f64) -> Result<i32, EranosError> {
    eranos_bindings::percentage_from_f64(value)
}

/// The percentage of `basis_points`, as the library writes it: "12.5%".
///
/// It writes the number as the library's text form of a percentage does, so
/// it cannot fail: a negative number, which `percentage_check` refuses, is
/// written the same way after a minus sign, "-12.5%".
#[must_use]
#[uniffi::export]
pub(crate) fn percentage_format(basis_points: i32) -> String {
    eranos_bindings::percentage_format(basis_points)
}

/// Checks that `basis_points` is a percentage, as an entry that holds one
/// requires.
///
/// # Errors
///
/// `MoneyFailure::InvalidPercentage`
/// for a negative number.
#[uniffi::export]
pub(crate) fn percentage_check(basis_points: i32) -> Result<(), EranosError> {
    eranos_bindings::percentage_check(basis_points)
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

/// The parts of the double `value`, in hundredths: 1.005 is 101.
///
/// The library writes the double in its shortest decimal form, rounds it
/// half away from zero to two decimal places, and parses it.
///
/// # Errors
///
/// `MoneyFailure::NotFinite` for NaN and the infinities, and
/// `MoneyFailure::InvalidParts` for a value that rounds to zero or below
/// and one whose hundredths do not fit an `i32`.
#[uniffi::export]
pub(crate) fn parts_from_f64(value: f64) -> Result<i32, EranosError> {
    eranos_bindings::parts_from_f64(value)
}

/// The parts of `hundredths`, as the library writes them: "0.5".
///
/// It writes the number as the library's text form of parts does, so it
/// cannot fail: zero and a negative number, which `parts_check` refuses, are
/// written the same way, "0" and "-0.5".
#[must_use]
#[uniffi::export]
pub(crate) fn parts_format(hundredths: i32) -> String {
    eranos_bindings::parts_format(hundredths)
}

/// Checks that `hundredths` is a number of parts, as a split of parts
/// requires.
///
/// # Errors
///
/// `MoneyFailure::InvalidParts` for a
/// number that is not positive.
#[uniffi::export]
pub(crate) fn parts_check(hundredths: i32) -> Result<(), EranosError> {
    eranos_bindings::parts_check(hundredths)
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
/// It writes the record's fields as the library's text form of a date
/// does, so it cannot fail: the fields of a day the calendar does not have,
/// which `calendar_date_check` refuses, are written the same way,
/// "2026-02-30".
#[must_use]
#[uniffi::export]
pub(crate) fn calendar_date_format(date: CalendarDate) -> String {
    eranos_bindings::calendar_date_format(date)
}

/// Checks that `date` is a day of the calendar, between years 1 and 9999,
/// as an entry's date must be.
///
/// # Errors
///
/// `CalendarFailure::InvalidDate`.
#[uniffi::export]
pub(crate) fn calendar_date_check(date: CalendarDate) -> Result<(), EranosError> {
    eranos_bindings::calendar_date_check(date)
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
/// It writes the record's fields as the library's text form of a time
/// does, so it cannot fail: the fields of a time the day does not have,
/// which `time_of_day_check` refuses, are written the same way, "24:00".
#[must_use]
#[uniffi::export]
pub(crate) fn time_of_day_format(time: TimeOfDay) -> String {
    eranos_bindings::time_of_day_format(time)
}

/// Checks that `time` is a time of day, from 00:00 to 23:59, as an entry's
/// time must be.
///
/// # Errors
///
/// `CalendarFailure::InvalidTime`.
#[uniffi::export]
pub(crate) fn time_of_day_check(time: TimeOfDay) -> Result<(), EranosError> {
    eranos_bindings::time_of_day_check(time)
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

/// The balance as the library writes it: "owes 12.00 EUR", "settled", or
/// "gets back 12.00 EUR".
///
/// It writes the library's words around the amount as `money_format`
/// writes it, so it cannot fail.
#[must_use]
#[uniffi::export]
pub(crate) fn balance_format(balance: &Balance) -> String {
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

/// The phone number written `text`, in E.164 form, "+40712345678", and in
/// international form, grouped as its country groups it, "+40 712 345 678".
///
/// # Errors
///
/// `ContactFailure::InvalidPhoneNumber`.
#[uniffi::export]
pub(crate) fn phone_number_parse(text: &str) -> Result<PhoneNumber, EranosError> {
    eranos_bindings::phone_number_parse(text)
}

/// The IBAN written `text`, without spaces, in capitals, with its country
/// code: "RO".
///
/// # Errors
///
/// `PaymentAccountFailure::InvalidIban`, with
/// the reason.
#[uniffi::export]
pub(crate) fn iban_parse(text: &str) -> Result<Iban, EranosError> {
    eranos_bindings::iban_parse(text)
}

/// The payment-app username written `text`, without surrounding
/// whitespace.
///
/// # Errors
///
/// `PaymentAccountFailure::EmptyUsername`.
#[uniffi::export]
pub(crate) fn username_parse(text: &str) -> Result<String, EranosError> {
    eranos_bindings::username_parse(text)
}

/// The no-tip rule, which an expense without a tip has: 0 % of the amount,
/// shared proportionally.
#[must_use]
#[uniffi::export]
pub(crate) fn tip_rule_none() -> TipRule {
    eranos_bindings::tip_rule_none()
}

/// The tip of `expense`, in its own currency.
///
/// # Errors
///
/// The errors of the cost's and the tip's conversions, among them
/// `MoneyFailure::UnknownCurrency` for an amount whose currency details are
/// not the library's for its code, and those of the library's
/// `TipRule::amount_on`.
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
/// The errors of the splits' and items' conversions, among them
/// `MoneyFailure::UnknownCurrency` for an amount whose currency details are
/// not the library's for its code.
#[uniffi::export]
pub(crate) fn expense_members(expense: &Expense) -> Result<Vec<String>, EranosError> {
    eranos_bindings::expense_members(expense)
}
