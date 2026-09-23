//! Error messages reach logs and the alerts the apps show. These tests pin
//! every message, keep them in the lowercase, unpunctuated style that lets
//! the apps embed them in longer sentences, and check that no message repeats
//! the rejected input, which may be an account number or a contact detail.

use super::*;

use crate::date::{Date, Time};
use crate::group::Language;
use crate::member::{Email, Iban, PhoneNumber};
use crate::money::{ExchangeRate, Parts, Percentage};

/// The member every example message names.
const MEMBER: MemberId = MemberId::from_bytes([0x11; 16]);
/// The category every example message names.
const CATEGORY: CategoryId = CategoryId::from_bytes([0x22; 16]);
/// The expense every example message names.
const EXPENSE: ExpenseId = ExpenseId::from_bytes([0x33; 16]);
/// The transfer every example message names.
const TRANSFER: TransferId = TransferId::from_bytes([0x44; 16]);

fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, Currency::EUR)
}

/// One example of every `ValueError` variant, with its exact message.
fn value_errors() -> Vec<(ValueError, &'static str)> {
    vec![
        (
            ValueError::Empty {
                field: Field::FirstName,
            },
            "the first name must not be empty",
        ),
        (ValueError::InvalidEmail, "invalid email address"),
        (
            ValueError::InvalidPhoneNumber,
            "invalid phone number: expected `+` followed by 7 to 15 digits",
        ),
        (
            ValueError::InvalidIban(IbanError::InvalidChecksum),
            "invalid IBAN: the check digits do not match",
        ),
        (ValueError::InvalidId, "invalid identifier: expected a UUID"),
        (ValueError::UnknownCurrency, "unknown currency code"),
        (
            ValueError::InvalidAmount,
            "invalid amount: expected digits with an optional `.` and sign",
        ),
        (
            ValueError::TooManyDecimals {
                currency: Currency::JPY,
                allowed: 0,
            },
            "JPY amounts have at most 0 decimal places",
        ),
        (
            ValueError::InvalidPercentage,
            "invalid percentage: expected a number with at most 2 decimal places",
        ),
        (
            ValueError::InvalidParts,
            "invalid parts: expected a positive number with at most 2 decimal places",
        ),
        (
            ValueError::InvalidExchangeRate,
            "invalid exchange rate: expected a positive number with at most 18 decimal places",
        ),
        (
            ValueError::InvalidDate,
            "invalid date: expected an existing `YYYY-MM-DD` date between years 1 and 9999",
        ),
        (
            ValueError::InvalidTime,
            "invalid time: expected `hh:mm` between 00:00 and 23:59",
        ),
        (
            ValueError::InvalidLanguage,
            "invalid language tag: expected a BCP 47 tag such as `en` or `ro-RO`",
        ),
        (
            ValueError::CurrencyMismatch {
                left: Currency::EUR,
                right: Currency::USD,
            },
            "cannot combine EUR and USD amounts",
        ),
        (ValueError::Overflow, "value out of range"),
    ]
}

/// Every `Field`, with the name messages use for it.
fn fields() -> Vec<(Field, &'static str)> {
    vec![
        (Field::GroupName, "group name"),
        (Field::FirstName, "first name"),
        (Field::AccountName, "account name"),
        (Field::Username, "username"),
        (Field::CategoryName, "category name"),
        (Field::ExpenseName, "expense name"),
        (Field::ItemName, "item name"),
    ]
}

/// Every `IbanError` variant, with its exact message.
fn iban_errors() -> Vec<(IbanError, &'static str)> {
    vec![
        (
            IbanError::InvalidCharacter,
            "only letters, digits, and spaces are allowed",
        ),
        (
            IbanError::InvalidFormat,
            "expected a two-letter country code followed by two check digits",
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
        (IbanError::InvalidChecksum, "the check digits do not match"),
    ]
}

/// One example of every `ExpenseError` variant, with its exact message.
fn expense_errors() -> Vec<(ExpenseError, &'static str)> {
    vec![
        (
            ExpenseError::Value(ValueError::Empty {
                field: Field::ExpenseName,
            }),
            "the expense name must not be empty",
        ),
        (
            ExpenseError::NonPositiveAmount,
            "the amount must be positive",
        ),
        (ExpenseError::MissingPayers, "the expense has no payers"),
        (
            ExpenseError::ConflictingPayers,
            "the payers were set twice, through `paid_by` and `payers`",
        ),
        (ExpenseError::MissingSplit, "the expense has no split"),
        (
            ExpenseError::ConflictingSplit,
            "the split was set twice, as a whole and item by item",
        ),
        (
            ExpenseError::EmptySplit,
            "a split needs at least one member",
        ),
        (
            ExpenseError::NoItems,
            "an itemized expense needs at least one item",
        ),
        (
            ExpenseError::DuplicateMember { member: MEMBER },
            "member 11111111-1111-1111-1111-111111111111 appears more than once in a split",
        ),
        (
            ExpenseError::NonPositiveShare { member: MEMBER },
            "member 11111111-1111-1111-1111-111111111111 has a share that is not positive",
        ),
        (
            ExpenseError::PercentagesDoNotAddUp {
                total_basis_points: 9_999,
            },
            "the percentages add up to 9999 basis points instead of 10000 (100%)",
        ),
        (
            ExpenseError::AmountsDoNotAddUp {
                expected: euros(1_000),
                actual: euros(999),
            },
            "the amounts add up to 9.99 EUR instead of 10.00 EUR",
        ),
        (
            ExpenseError::ItemsDoNotAddUp {
                expected: euros(9_000),
                actual: euros(8_000),
            },
            "the items add up to 80.00 EUR instead of 90.00 EUR",
        ),
        (
            ExpenseError::CurrencyMismatch {
                expected: Currency::EUR,
                found: Currency::USD,
            },
            "expected an amount in EUR, found one in USD",
        ),
        (
            ExpenseError::ZeroQuantity,
            "an item quantity must be at least 1",
        ),
        (ExpenseError::NegativeTip, "a tip cannot be negative"),
        (ExpenseError::Overflow, "the amounts are out of range"),
    ]
}

/// Every `TransferError` variant, with its exact message.
fn transfer_errors() -> Vec<(TransferError, &'static str)> {
    vec![
        (
            TransferError::Value(ValueError::InvalidDate),
            "invalid date: expected an existing `YYYY-MM-DD` date between years 1 and 9999",
        ),
        (
            TransferError::SameMember,
            "a transfer needs two different members",
        ),
        (
            TransferError::NonPositiveAmount,
            "the amount must be positive",
        ),
    ]
}

/// One example of every `GroupError` variant, with its exact message.
fn group_errors() -> Vec<(GroupError, &'static str)> {
    vec![
        (
            GroupError::UnknownMember(MEMBER),
            "unknown member 11111111-1111-1111-1111-111111111111",
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
            GroupError::UnknownTransfer(TRANSFER),
            "unknown transfer 44444444-4444-4444-4444-444444444444",
        ),
        (
            GroupError::DuplicateMember(MEMBER),
            "member 11111111-1111-1111-1111-111111111111 already exists",
        ),
        (
            GroupError::DuplicateCategory(CATEGORY),
            "category 22222222-2222-2222-2222-222222222222 already exists",
        ),
        (
            GroupError::DuplicateExpense(EXPENSE),
            "expense 33333333-3333-3333-3333-333333333333 already exists",
        ),
        (
            GroupError::DuplicateTransfer(TRANSFER),
            "transfer 44444444-4444-4444-4444-444444444444 already exists",
        ),
        (
            GroupError::DuplicateCategoryName { existing: CATEGORY },
            "category 22222222-2222-2222-2222-222222222222 already has this name",
        ),
        (
            GroupError::ArchivedMember(MEMBER),
            "member 11111111-1111-1111-1111-111111111111 is archived",
        ),
        (
            GroupError::MemberInUse {
                member: MEMBER,
                expenses: vec![EXPENSE, ExpenseId::from_bytes([0x55; 16])],
                transfers: vec![TRANSFER],
            },
            "member 11111111-1111-1111-1111-111111111111 is used by 2 expenses and 1 transfer",
        ),
        (
            GroupError::CategoryInUse {
                category: CATEGORY,
                expenses: vec![EXPENSE],
            },
            "category 22222222-2222-2222-2222-222222222222 is used by 1 expense",
        ),
        (
            GroupError::MissingExchangeRate {
                currency: Currency::USD,
            },
            "an exchange rate from USD into the group currency is required",
        ),
        (
            GroupError::UnexpectedExchangeRate,
            "an entry in the group currency cannot have an exchange rate",
        ),
        (
            GroupError::DuplicateExchangeRate {
                currency: Currency::USD,
            },
            "two exchange rates were given for USD",
        ),
        (
            GroupError::Overflow,
            "the group totals would be out of range",
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

/// Every expected message above.
fn every_message() -> Vec<&'static str> {
    let mut messages = Vec::new();
    messages.extend(value_errors().into_iter().map(|(_, message)| message));
    messages.extend(fields().into_iter().map(|(_, name)| name));
    messages.extend(iban_errors().into_iter().map(|(_, message)| message));
    messages.extend(expense_errors().into_iter().map(|(_, message)| message));
    messages.extend(transfer_errors().into_iter().map(|(_, message)| message));
    messages.extend(group_errors().into_iter().map(|(_, message)| message));
    messages.extend(settlement_errors().into_iter().map(|(_, message)| message));
    messages
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
fn assert_not_echoed(input: &str, error: &ValueError) {
    let message = error.to_string();
    for word in input.split_whitespace().filter(|word| word.len() >= 3) {
        assert!(!message.contains(word), "{message:?} repeats {word:?}");
    }
}

/// A reworded validation message would change what users read in the apps.
#[test]
fn value_error_messages_are_pinned() {
    // Every variant.
    for (error, message) in value_errors() {
        assert_eq!(error.to_string(), message);
    }
    // The IBAN reason is appended to the prefix.
    assert_eq!(
        ValueError::InvalidIban(IbanError::WrongLength {
            expected: 24,
            found: 20
        })
        .to_string(),
        "invalid IBAN: expected 24 characters for this country, found 20"
    );
}

/// A field named wrongly would tell the user to fill in the wrong input.
#[test]
fn every_field_has_a_lowercase_name() {
    // The name on its own.
    for (field, name) in fields() {
        assert_eq!(field.to_string(), name);
    }
    // The name inside the empty-field message.
    for (field, name) in fields() {
        assert_eq!(
            ValueError::Empty { field }.to_string(),
            format!("the {name} must not be empty")
        );
    }
}

/// A reworded IBAN message would change the hint users get for a typo.
#[test]
fn iban_error_messages_are_pinned() {
    // Every variant.
    for (error, message) in iban_errors() {
        assert_eq!(error.to_string(), message);
    }
}

/// A reworded expense message would change what users read when an expense
/// is rejected.
#[test]
fn expense_error_messages_are_pinned() {
    // Every variant.
    for (error, message) in expense_errors() {
        assert_eq!(error.to_string(), message);
    }
}

/// A reworded transfer message would change what users read when a
/// repayment is rejected.
#[test]
fn transfer_error_messages_are_pinned() {
    // Every variant.
    for (error, message) in transfer_errors() {
        assert_eq!(error.to_string(), message);
    }
}

/// A reworded group message would change what users read when an edit is
/// refused.
#[test]
fn group_error_messages_are_pinned() {
    // Every variant.
    for (error, message) in group_errors() {
        assert_eq!(error.to_string(), message);
    }
}

/// A reworded settlement message would change what users read when a
/// repayment plan cannot be computed.
#[test]
fn settlement_error_messages_are_pinned() {
    // Every variant.
    for (error, message) in settlement_errors() {
        assert_eq!(error.to_string(), message);
    }
}

/// A capitalized or punctuated message would read wrongly when the apps
/// embed it in a longer sentence.
#[test]
fn no_message_starts_like_a_sentence_or_ends_with_a_period() {
    // Lowercase start, or a currency code.
    for message in every_message() {
        assert!(!message.is_empty());
        assert!(starts_in_lowercase_or_with_a_code(message), "{message:?}");
    }
    // No trailing period.
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
    // Account numbers: a typo and a truncated number.
    for input in [
        "RO49 AAAA 1B31 0075 9384 0001",
        "RO49 AAAA 1B31 0075 9384",
        "RO49-AAAA-1B31-0075-9384-0000",
    ] {
        assert_not_echoed(input, &Iban::parse(input).unwrap_err());
    }
    // Contact details.
    let email = "ana.popescu@@example.com";
    assert_not_echoed(email, &Email::parse(email).unwrap_err());
    let phone = "+40 712 345 67x";
    assert_not_echoed(phone, &PhoneNumber::parse(phone).unwrap_err());
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
    assert_not_echoed("2026-02-30", &Date::parse("2026-02-30").unwrap_err());
    assert_not_echoed("24:60", &Time::parse("24:60").unwrap_err());
    assert_not_echoed("not-a-uuid", &"not-a-uuid".parse::<MemberId>().unwrap_err());
    assert_not_echoed("ro_RO", &Language::parse("ro_RO").unwrap_err());
}

/// A validation failure inside a builder would fail to propagate through `?`.
#[test]
fn value_errors_convert_into_expense_and_transfer_errors() {
    // Expense errors.
    assert_eq!(
        ExpenseError::from(ValueError::InvalidAmount),
        ExpenseError::Value(ValueError::InvalidAmount)
    );
    // Transfer errors.
    assert_eq!(
        TransferError::from(ValueError::InvalidDate),
        TransferError::Value(ValueError::InvalidDate)
    );
    // The `?` operator uses the conversion.
    let parse_amount =
        || -> Result<Money, ExpenseError> { Ok(Money::parse("1,50", Currency::EUR)?) };
    assert_eq!(
        parse_amount(),
        Err(ExpenseError::Value(ValueError::InvalidAmount))
    );
}

/// A wrapped validation failure would show a different message than the same
/// failure on its own.
#[test]
fn wrapped_value_errors_keep_their_message() {
    // Every value error, wrapped both ways.
    for (error, message) in value_errors() {
        assert_eq!(ExpenseError::Value(error.clone()).to_string(), message);
        assert_eq!(TransferError::Value(error).to_string(), message);
    }
}

/// A count in an error message agrees with its noun: "1 expense", never
/// "1 expenses", however many of each kind of entry there are.
#[test]
fn usage_counts_agree_with_their_nouns() {
    // Only one kind of entry is named when the other is absent.
    assert_eq!(usage(1, 0), "1 expense");
    assert_eq!(usage(3, 0), "3 expenses");
    assert_eq!(usage(0, 1), "1 transfer");
    assert_eq!(usage(0, 2), "2 transfers");
    // Both kinds, each in its own number.
    assert_eq!(usage(1, 2), "1 expense and 2 transfers");
    assert_eq!(count(1, "item", "items"), "1 item");
    assert_eq!(count(0, "item", "items"), "0 items");
}
