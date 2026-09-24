//! Error messages reach logs and the alerts the apps show. These tests pin
//! the message of every public error, keep them in the lowercase,
//! unpunctuated style that lets the apps embed them in longer sentences, and
//! check that no message repeats the rejected input, which may be an account
//! number or a contact detail.

use std::fmt::Display;

use eranos::error::{
    AccountError, CalendarError, CategoryError, ContactError, ExpenseError, GroupError, IbanError,
    IdError, LanguageError, MemberError, MoneyError, SettlementError, TransferError,
};
use eranos::{
    CalendarDate, CategoryId, Currency, Email, ExchangeRate, ExpenseId, Iban, Language, MemberId,
    Money, Parts, Percentage, PhoneNumber, TimeOfDay, TransferId,
};

/// The member every example message names.
const MEMBER: MemberId = MemberId::from_bytes([0x11; 16]);
/// The category every example message names.
const CATEGORY: CategoryId = CategoryId::from_bytes([0x22; 16]);
/// The expense every example message names.
const EXPENSE: ExpenseId = ExpenseId::from_bytes([0x33; 16]);
/// The transfer every example message names.
const TRANSFER: TransferId = TransferId::from_bytes([0x44; 16]);

const fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(Currency::EUR, minor_units)
}

/// Every `IdError` variant, with its exact message.
fn id_errors() -> Vec<(IdError, &'static str)> {
    vec![(IdError::InvalidId, "invalid identifier: expected a UUID")]
}

/// Every `CalendarError` variant, with its exact message.
fn calendar_errors() -> Vec<(CalendarError, &'static str)> {
    vec![
        (
            CalendarError::InvalidDate,
            "invalid date: expected an existing ISO 8601 date between years 1 and 9999",
        ),
        (
            CalendarError::InvalidTime,
            "invalid time: expected an ISO 8601 time between 00:00 and 23:59 with zero seconds",
        ),
    ]
}

/// One example of every `MoneyError` variant, with its exact message.
fn money_errors() -> Vec<(MoneyError, &'static str)> {
    vec![
        (
            MoneyError::CurrencyMismatch {
                left: Currency::EUR,
                right: Currency::USD,
            },
            "cannot combine EUR and USD amounts",
        ),
        (
            MoneyError::InvalidAmount,
            "invalid amount: expected digits with an optional `.` and sign",
        ),
        (
            MoneyError::InvalidExchangeRate,
            "invalid exchange rate: expected a positive number with at most 18 decimal places",
        ),
        (
            MoneyError::InvalidParts,
            "invalid parts: expected a positive number with at most 2 decimal places",
        ),
        (
            MoneyError::InvalidPercentage,
            "invalid percentage: expected a number with at most 2 decimal places",
        ),
        (MoneyError::Overflow, "value out of range"),
        (
            MoneyError::TooManyDecimals {
                allowed: 0,
                currency: Currency::JPY,
            },
            "JPY amounts have at most 0 decimal places",
        ),
        (MoneyError::UnknownCurrency, "unknown currency code"),
    ]
}

/// One example of every `AccountError` variant, with its exact message.
fn account_errors() -> Vec<(AccountError, &'static str)> {
    vec![
        (
            AccountError::EmptyName,
            "the account name must not be empty",
        ),
        (
            AccountError::EmptyUsername,
            "the username must not be empty",
        ),
        (
            AccountError::InvalidIban(IbanError::InvalidChecksum),
            "invalid IBAN: the check digits do not match",
        ),
    ]
}

/// One example of every `IbanError` variant, with its exact message.
fn iban_errors() -> Vec<(IbanError, &'static str)> {
    vec![
        (
            IbanError::InvalidCharacter,
            "only letters, digits, and spaces are allowed",
        ),
        (IbanError::InvalidChecksum, "the check digits do not match"),
        (
            IbanError::InvalidFormat,
            "expected a two-letter country code followed by two check digits",
        ),
        (
            IbanError::InvalidStructure,
            "the account number does not match this country's format",
        ),
        (
            IbanError::UnknownCountry,
            "the country code is not in the IBAN registry",
        ),
        (
            IbanError::WrongLength {
                expected: 22,
                found: 21,
            },
            "expected 22 characters for this country, found 21",
        ),
    ]
}

/// Every `ContactError` variant, with its exact message.
fn contact_errors() -> Vec<(ContactError, &'static str)> {
    vec![
        (ContactError::InvalidEmail, "invalid email address"),
        (
            ContactError::InvalidPhoneNumber,
            "invalid phone number: expected `+` and a number valid in its country",
        ),
    ]
}

/// Every `MemberError` variant, with its exact message.
fn member_errors() -> Vec<(MemberError, &'static str)> {
    vec![(
        MemberError::EmptyFirstName,
        "the first name must not be empty",
    )]
}

/// Every `CategoryError` variant, with its exact message.
fn category_errors() -> Vec<(CategoryError, &'static str)> {
    vec![(
        CategoryError::EmptyName,
        "the category name must not be empty",
    )]
}

/// Every `LanguageError` variant, with its exact message.
fn language_errors() -> Vec<(LanguageError, &'static str)> {
    vec![(
        LanguageError::InvalidLanguage,
        "invalid language tag: expected a BCP 47 tag such as `en` or `ro-RO`",
    )]
}

/// One example of every `ExpenseError` variant, with its exact message.
fn expense_errors() -> Vec<(ExpenseError, &'static str)> {
    vec![
        (
            ExpenseError::AmountsDoNotAddUp {
                actual: euros(999),
                expected: euros(1_000),
            },
            "the amounts add up to 9.99 EUR instead of 10.00 EUR",
        ),
        (
            ExpenseError::CurrencyMismatch {
                expected: Currency::EUR,
                found: Currency::USD,
            },
            "expected an amount in EUR, found one in USD",
        ),
        (
            ExpenseError::DuplicateMember { member: MEMBER },
            "member 11111111-1111-1111-1111-111111111111 appears more than once in a split",
        ),
        (
            ExpenseError::EmptyItemName,
            "the item name must not be empty",
        ),
        (
            ExpenseError::EmptyName,
            "the expense name must not be empty",
        ),
        (
            ExpenseError::EmptySplit,
            "a split needs at least one member",
        ),
        (
            ExpenseError::ItemsDoNotAddUp {
                actual: euros(8_000),
                expected: euros(9_000),
            },
            "the items add up to 80.00 EUR instead of 90.00 EUR",
        ),
        (ExpenseError::NegativeTip, "a tip cannot be negative"),
        (
            ExpenseError::NoItems,
            "an itemised expense needs at least one item",
        ),
        (
            ExpenseError::NonPositiveAmount,
            "the amount must be positive",
        ),
        (
            ExpenseError::NonPositiveShare { member: MEMBER },
            "member 11111111-1111-1111-1111-111111111111 has a share that is not positive",
        ),
        (ExpenseError::Overflow, "the amounts are out of range"),
        (
            ExpenseError::PercentagesDoNotAddUp {
                total_basis_points: 9_999,
            },
            "the percentages add up to 9999 basis points instead of 10000 (100%)",
        ),
        (
            ExpenseError::ZeroQuantity,
            "an item quantity must be at least 1",
        ),
    ]
}

/// Every `TransferError` variant, with its exact message.
fn transfer_errors() -> Vec<(TransferError, &'static str)> {
    vec![
        (
            TransferError::NonPositiveAmount,
            "the amount must be positive",
        ),
        (
            TransferError::SameMember,
            "a transfer needs two different members",
        ),
    ]
}

/// One example of every `GroupError` variant, with its exact message.
fn group_errors() -> Vec<(GroupError, &'static str)> {
    vec![
        (
            GroupError::ArchivedMember(MEMBER),
            "member 11111111-1111-1111-1111-111111111111 is archived",
        ),
        (
            GroupError::BuiltInCategory,
            "the built-in category cannot be renamed or removed",
        ),
        (
            GroupError::DuplicateCategory(CATEGORY),
            "category 22222222-2222-2222-2222-222222222222 already exists",
        ),
        (
            GroupError::DuplicateCategoryName { existing: CATEGORY },
            "category 22222222-2222-2222-2222-222222222222 already has this name",
        ),
        (
            GroupError::DuplicateExchangeRate {
                currency: Currency::USD,
            },
            "two exchange rates were given for USD",
        ),
        (
            GroupError::DuplicateExpense(EXPENSE),
            "expense 33333333-3333-3333-3333-333333333333 already exists",
        ),
        (
            GroupError::DuplicateMember(MEMBER),
            "member 11111111-1111-1111-1111-111111111111 already exists",
        ),
        (
            GroupError::DuplicateTransfer(TRANSFER),
            "transfer 44444444-4444-4444-4444-444444444444 already exists",
        ),
        (GroupError::EmptyName, "the group name must not be empty"),
        (
            GroupError::MemberInUse {
                expenses: vec![EXPENSE, ExpenseId::from_bytes([0x55; 16])],
                member: MEMBER,
                transfers: vec![TRANSFER],
            },
            "member 11111111-1111-1111-1111-111111111111 is used by 2 expenses and 1 transfer",
        ),
        (
            GroupError::MissingExchangeRate {
                currency: Currency::USD,
            },
            "an exchange rate from USD into the group currency is required",
        ),
        (
            GroupError::Overflow,
            "the group totals would be out of range",
        ),
        (
            GroupError::UnexpectedExchangeRate,
            "an entry in the group currency cannot have an exchange rate",
        ),
        (
            GroupError::UnknownCategory(CATEGORY),
            "unknown category 22222222-2222-2222-2222-222222222222",
        ),
        (
            GroupError::UnknownExpense(EXPENSE),
            "unknown expense 33333333-3333-3333-3333-333333333333",
        ),
        (
            GroupError::UnknownMember(MEMBER),
            "unknown member 11111111-1111-1111-1111-111111111111",
        ),
        (
            GroupError::UnknownTransfer(TRANSFER),
            "unknown transfer 44444444-4444-4444-4444-444444444444",
        ),
        (
            GroupError::UnsupportedFormat { format: 3 },
            "unsupported group format 3: this version reads format 2",
        ),
    ]
}

/// Every `SettlementError` variant, with its exact message.
fn settlement_errors() -> Vec<(SettlementError, &'static str)> {
    vec![(
        SettlementError::OutOfMemory { balances: 40 },
        "not enough memory to find the minimum transfers between 40 members",
    )]
}

/// Fails unless every error of `errors` prints its expected message.
fn assert_messages<Error: Display>(errors: Vec<(Error, &'static str)>) {
    for (error, message) in errors {
        assert_eq!(error.to_string(), message);
    }
}

/// The expected messages of `errors`.
fn messages<Error>(errors: Vec<(Error, &'static str)>) -> impl Iterator<Item = &'static str> {
    errors.into_iter().map(|(_, message)| message)
}

/// Every expected message above.
fn every_message() -> Vec<&'static str> {
    let mut every = Vec::new();
    every.extend(messages(id_errors()));
    every.extend(messages(calendar_errors()));
    every.extend(messages(money_errors()));
    every.extend(messages(account_errors()));
    every.extend(messages(iban_errors()));
    every.extend(messages(contact_errors()));
    every.extend(messages(member_errors()));
    every.extend(messages(category_errors()));
    every.extend(messages(language_errors()));
    every.extend(messages(expense_errors()));
    every.extend(messages(transfer_errors()));
    every.extend(messages(group_errors()));
    every.extend(messages(settlement_errors()));
    every
}

/// Whether `message` starts with a lowercase word or with a code such as
/// `EUR`, which is uppercase by nature.
fn starts_in_lowercase_or_with_a_code(message: &str) -> bool {
    let first_word = message.split(' ').next().unwrap_or_default();
    first_word.starts_with(char::is_lowercase)
        || first_word
            .chars()
            .all(|character| character.is_ascii_uppercase())
}

/// Fails when any word of `input` of three or more characters appears in the
/// message of `error`.
fn assert_not_echoed(input: &str, error: &impl Display) {
    let message = error.to_string();
    for word in input.split_whitespace().filter(|word| word.len() >= 3) {
        assert!(!message.contains(word), "{message:?} repeats {word:?}");
    }
}

/// A reworded identifier message would change what users read when a link
/// or a file holds a broken ID.
#[test]
fn id_error_messages_are_pinned() {
    // Every variant.
    assert_messages(id_errors());
}

/// A reworded calendar message would change what users read when a date or
/// time is refused.
#[test]
fn calendar_error_messages_are_pinned() {
    // Every variant.
    assert_messages(calendar_errors());
}

/// A reworded money message would change what users read when an amount, a
/// rate, a percentage, or a number of parts is refused.
#[test]
fn money_error_messages_are_pinned() {
    // Every variant.
    assert_messages(money_errors());
}

/// A reworded account message would change what users read when payment
/// details are refused.
#[test]
fn account_error_messages_are_pinned() {
    // Every variant.
    assert_messages(account_errors());
    // The IBAN reason is appended to the prefix.
    assert_eq!(
        AccountError::InvalidIban(IbanError::WrongLength {
            expected: 24,
            found: 20
        })
        .to_string(),
        "invalid IBAN: expected 24 characters for this country, found 20"
    );
}

/// A reworded IBAN message would change the hint users get for a typo.
#[test]
fn iban_error_messages_are_pinned() {
    // Every variant.
    assert_messages(iban_errors());
}

/// A reworded contact message would change what users read when an email
/// address or a phone number is refused.
#[test]
fn contact_error_messages_are_pinned() {
    // Every variant.
    assert_messages(contact_errors());
}

/// A reworded member message would tell the user to fill in the wrong input.
#[test]
fn member_error_messages_are_pinned() {
    // Every variant.
    assert_messages(member_errors());
}

/// A reworded category message would tell the user to fill in the wrong
/// input.
#[test]
fn category_error_messages_are_pinned() {
    // Every variant.
    assert_messages(category_errors());
}

/// A reworded language message would change the hint users get for a tag.
#[test]
fn language_error_messages_are_pinned() {
    // Every variant.
    assert_messages(language_errors());
}

/// A reworded expense message would change what users read when an expense
/// is rejected.
#[test]
fn expense_error_messages_are_pinned() {
    // Every variant.
    assert_messages(expense_errors());
}

/// A reworded transfer message would change what users read when a
/// repayment is rejected.
#[test]
fn transfer_error_messages_are_pinned() {
    // Every variant.
    assert_messages(transfer_errors());
}

/// A reworded group message would change what users read when an edit is
/// refused.
#[test]
fn group_error_messages_are_pinned() {
    // Every variant.
    assert_messages(group_errors());
}

/// A reworded settlement message would change what users read when a
/// repayment plan cannot be computed.
#[test]
fn settlement_error_messages_are_pinned() {
    // Every variant.
    assert_messages(settlement_errors());
}

/// A capitalised or punctuated message would read wrongly when the apps
/// embed it in a longer sentence.
#[test]
fn no_message_starts_like_a_sentence_or_ends_with_a_period() {
    // Lowercase start, or a currency code.
    for message in every_message() {
        assert!(!message.is_empty());
        assert!(starts_in_lowercase_or_with_a_code(message), "{message:?}");
    }
    // No trailing full stop.
    for message in every_message() {
        assert!(!message.ends_with('.'), "{message:?}");
    }
    // The helper itself tells a sentence start from a code.
    assert!(!starts_in_lowercase_or_with_a_code("Invalid amount"));
    assert!(starts_in_lowercase_or_with_a_code("EUR amounts"));
}

/// An account number or contact detail typed by a user would leak into logs.
#[test]
fn rejected_input_never_appears_in_a_message() {
    // Account numbers: a typo, a truncated number, a digit where the bank
    // code has letters, and check digits no bank issues.
    for input in [
        "RO49 AAAA 1B31 0075 9384 0001",
        "RO49 AAAA 1B31 0075 9384",
        "RO49-AAAA-1B31-0075-9384-0000",
        "RO49 1AAA 1B31 0075 9384 0000",
        "DE00 3704 0044 0532 0000 52",
    ] {
        assert_not_echoed(input, &Iban::parse(input).unwrap_err());
    }
    // Contact details.
    for email in ["ana.popescu@@example.com", "ana..popescu@example.com"] {
        assert_not_echoed(email, &Email::parse(email).unwrap_err());
    }
    for phone in ["+40 712 345 67x", "+40 612 345 678", "+1 800 FLOWERS"] {
        assert_not_echoed(phone, &PhoneNumber::parse(phone).unwrap_err());
    }
    // Amounts, ratios, and rates.
    assert_not_echoed(
        "12,345",
        &Money::parse("12,345", Currency::EUR).unwrap_err(),
    );
    assert_not_echoed(
        "12.345",
        &Money::parse("12.345", Currency::EUR).unwrap_err(),
    );
    assert_not_echoed("33.333", &Percentage::parse("33.333").unwrap_err());
    assert_not_echoed("0.001", &Parts::parse("0.001").unwrap_err());
    assert_not_echoed("4,9765", &ExchangeRate::parse("4,9765").unwrap_err());
    // Codes, dates, identifiers, and tags.
    assert_not_echoed("XYZ", &Currency::from_code("XYZ").unwrap_err());
    assert_not_echoed(
        "2026-02-30",
        &CalendarDate::parse("2026-02-30").unwrap_err(),
    );
    assert_not_echoed("24:60", &TimeOfDay::parse("24:60").unwrap_err());
    assert_not_echoed("not-a-uuid", &"not-a-uuid".parse::<MemberId>().unwrap_err());
    assert_not_echoed("ro_RO", &Language::parse("ro_RO").unwrap_err());
    assert_not_echoed("en-US-US", &Language::parse("en-US-US").unwrap_err());
}
