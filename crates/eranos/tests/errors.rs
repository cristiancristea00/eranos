//! Error messages reach logs and the alerts the apps show. These tests pin
//! the message of every public error, keep them in the lowercase,
//! unpunctuated style that lets the apps embed them in longer sentences, and
//! check that no message repeats the rejected input, which may be an account
//! number or a contact detail.
#![forbid(unsafe_code)]
#![expect(
    clippy::unwrap_used,
    reason = "a failing test helper panics to report the failure, as clippy.toml allows inside test functions"
)]

use std::fmt::Display;

use eranos::error::{
    CalendarError, CategoryError, ContactError, ExpenseError, ExpenseGroupError, IbanError,
    IdError, LanguageError, MemberError, MoneyError, PaymentAccountError, SettlementError,
    TransferError,
};
use eranos::{
    CalendarDate, CategoryId, CurrencyCode, Email, ExchangeRate, ExpenseId, Iban, Language,
    MemberId, Money, Parts, Percentage, PhoneNumber, TimeOfDay, TransferId,
};

/// The member every example message names.
fn member() -> MemberId {
    MemberId::new("11111111-1111-1111-1111-111111111111").unwrap()
}

/// The category every example message names.
fn category() -> CategoryId {
    CategoryId::new("22222222-2222-2222-2222-222222222222").unwrap()
}

/// The expense every example message names.
fn expense() -> ExpenseId {
    ExpenseId::new("33333333-3333-3333-3333-333333333333").unwrap()
}

/// The transfer every example message names.
fn transfer() -> TransferId {
    TransferId::new("44444444-4444-4444-4444-444444444444").unwrap()
}

/// An error and the exact message it prints.
struct Expected<Error> {
    error: Error,
    message: &'static str,
}

const fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, CurrencyCode::EUR)
}

/// Every `IdError` variant, with its exact message.
fn id_errors() -> Vec<Expected<IdError>> {
    vec![
        Expected {
            error: IdError::Empty,
            message: "an identifier must not be empty",
        },
        Expected {
            error: IdError::TooLong,
            message: "an identifier can be at most 64 bytes long",
        },
        Expected {
            error: IdError::ControlCharacter,
            message: "an identifier cannot contain control characters",
        },
        Expected {
            error: IdError::SurroundingWhitespace,
            message: "an identifier cannot start or end with whitespace",
        },
    ]
}

/// Every `CalendarError` variant, with its exact message.
fn calendar_errors() -> Vec<Expected<CalendarError>> {
    vec![
        Expected {
            error: CalendarError::InvalidDate,
            message: "invalid date: expected an existing ISO 8601 date between years 1 and 9999",
        },
        Expected {
            error: CalendarError::InvalidTime,
            message: "invalid time: expected an ISO 8601 time between 00:00 and 23:59 with zero seconds",
        },
    ]
}

/// One example of every `MoneyError` variant, with its exact message.
fn money_errors() -> Vec<Expected<MoneyError>> {
    vec![
        Expected {
            error: MoneyError::CurrencyMismatch {
                left: CurrencyCode::EUR,
                right: CurrencyCode::USD,
            },
            message: "cannot combine EUR and USD amounts",
        },
        Expected {
            error: MoneyError::InvalidAmount,
            message: "invalid amount: expected digits with an optional `.` and sign",
        },
        Expected {
            error: MoneyError::InvalidExchangeRate,
            message: "invalid exchange rate: expected a positive number with at most 18 decimal places",
        },
        Expected {
            error: MoneyError::InvalidParts,
            message: "invalid parts: expected a positive number with at most 2 decimal places",
        },
        Expected {
            error: MoneyError::InvalidPercentage,
            message: "invalid percentage: expected a number with at most 2 decimal places",
        },
        Expected {
            error: MoneyError::InvalidQuantity,
            message: "invalid quantity: expected a positive number with at most 4 decimal places",
        },
        Expected {
            error: MoneyError::NotFinite,
            message: "the number is not finite",
        },
        Expected {
            error: MoneyError::Overflow,
            message: "value out of range",
        },
        Expected {
            error: MoneyError::TooManyDecimals {
                currency: CurrencyCode::JPY,
                allowed: 0,
            },
            message: "JPY amounts have at most 0 decimal places",
        },
        Expected {
            error: MoneyError::UnknownCurrency,
            message: "unknown currency code",
        },
    ]
}

/// One example of every `PaymentAccountError` variant, with its exact message.
fn account_errors() -> Vec<Expected<PaymentAccountError>> {
    vec![
        Expected {
            error: PaymentAccountError::EmptyName,
            message: "the account name must not be empty",
        },
        Expected {
            error: PaymentAccountError::EmptyUsername,
            message: "the username must not be empty",
        },
        Expected {
            error: PaymentAccountError::InvalidIban(IbanError::InvalidChecksum),
            message: "invalid IBAN: the check digits do not match",
        },
    ]
}

/// One example of every `IbanError` variant, with its exact message.
fn iban_errors() -> Vec<Expected<IbanError>> {
    vec![
        Expected {
            error: IbanError::InvalidCharacter,
            message: "only letters, digits, and spaces are allowed",
        },
        Expected {
            error: IbanError::InvalidChecksum,
            message: "the check digits do not match",
        },
        Expected {
            error: IbanError::InvalidFormat,
            message: "expected a two-letter country code followed by two check digits",
        },
        Expected {
            error: IbanError::InvalidStructure,
            message: "the account number does not match this country's format",
        },
        Expected {
            error: IbanError::UnknownCountry,
            message: "the country code is not in the IBAN registry",
        },
        Expected {
            error: IbanError::WrongLength {
                expected: 22,
                found: 21,
            },
            message: "expected 22 characters for this country, found 21",
        },
    ]
}

/// Every `ContactError` variant, with its exact message.
fn contact_errors() -> Vec<Expected<ContactError>> {
    vec![
        Expected {
            error: ContactError::InvalidEmail,
            message: "invalid email address",
        },
        Expected {
            error: ContactError::InvalidPhoneNumber,
            message: "invalid phone number: expected `+` and a number valid in its country",
        },
    ]
}

/// Every `MemberError` variant, with its exact message.
fn member_errors() -> Vec<Expected<MemberError>> {
    vec![Expected {
        error: MemberError::EmptyFirstName,
        message: "the first name must not be empty",
    }]
}

/// Every `CategoryError` variant, with its exact message.
fn category_errors() -> Vec<Expected<CategoryError>> {
    vec![Expected {
        error: CategoryError::EmptyName,
        message: "the category name must not be empty",
    }]
}

/// Every `LanguageError` variant, with its exact message.
fn language_errors() -> Vec<Expected<LanguageError>> {
    vec![Expected {
        error: LanguageError::InvalidLanguage,
        message: "invalid language tag: expected a BCP 47 tag such as `en` or `ro-RO`",
    }]
}

/// One example of every `ExpenseError` variant, with its exact message.
fn expense_errors() -> Vec<Expected<ExpenseError>> {
    vec![
        Expected {
            error: ExpenseError::AmountsDoNotAddUp {
                expected: euros(1_000),
                actual: euros(999),
            },
            message: "the amounts add up to 9.99 EUR instead of 10.00 EUR",
        },
        Expected {
            error: ExpenseError::CurrencyMismatch {
                expected: CurrencyCode::EUR,
                found: CurrencyCode::USD,
            },
            message: "expected an amount in EUR, found one in USD",
        },
        Expected {
            error: ExpenseError::DiscountAboveHundredPercent,
            message: "a discount cannot be more than 100%",
        },
        Expected {
            error: ExpenseError::DiscountAbovePrice,
            message: "the discount is larger than the price it reduces",
        },
        Expected {
            error: ExpenseError::DuplicateMember { member: member() },
            message: "a member appears more than once in the split",
        },
        Expected {
            error: ExpenseError::EmptyItemName,
            message: "the item name must not be empty",
        },
        Expected {
            error: ExpenseError::EmptyName,
            message: "the expense name must not be empty",
        },
        Expected {
            error: ExpenseError::EmptySplit,
            message: "a split needs at least one member",
        },
        Expected {
            error: ExpenseError::ItemsDoNotAddUp {
                expected: euros(9_000),
                actual: euros(8_000),
            },
            message: "the items add up to 80.00 EUR instead of 90.00 EUR",
        },
        Expected {
            error: ExpenseError::NegativeDiscount,
            message: "a discount cannot be negative",
        },
        Expected {
            error: ExpenseError::NegativeTip,
            message: "a tip cannot be negative",
        },
        Expected {
            error: ExpenseError::NoItems,
            message: "an itemised expense needs at least one item",
        },
        Expected {
            error: ExpenseError::NonPositiveAmount,
            message: "the amount must be positive",
        },
        Expected {
            error: ExpenseError::NonPositiveShare { member: member() },
            message: "a member's share is not positive",
        },
        Expected {
            error: ExpenseError::Overflow,
            message: "the amounts are out of range",
        },
        Expected {
            error: ExpenseError::PercentagesDoNotAddUp {
                total_basis_points: 9_999,
            },
            message: "the percentages add up to 9999 basis points instead of 10000 (100%)",
        },
    ]
}

/// Every `TransferError` variant, with its exact message.
fn transfer_errors() -> Vec<Expected<TransferError>> {
    vec![
        Expected {
            error: TransferError::NonPositiveAmount,
            message: "the amount must be positive",
        },
        Expected {
            error: TransferError::SameMember,
            message: "a transfer needs two different members",
        },
    ]
}

/// One example of every `ExpenseGroupError` variant, with its exact message.
fn group_errors() -> Vec<Expected<ExpenseGroupError>> {
    vec![
        Expected {
            error: ExpenseGroupError::ArchivedMember(member()),
            message: "the member is archived",
        },
        Expected {
            error: ExpenseGroupError::BuiltInCategory,
            message: "the built-in category cannot be renamed or removed",
        },
        Expected {
            error: ExpenseGroupError::Busy,
            message: "the group is locked by another call",
        },
        Expected {
            error: ExpenseGroupError::DuplicateCategory(category()),
            message: "a category with this identifier already exists",
        },
        Expected {
            error: ExpenseGroupError::DuplicateCategoryName {
                existing: category(),
            },
            message: "another category already has this name",
        },
        Expected {
            error: ExpenseGroupError::DuplicateExchangeRate {
                currency: CurrencyCode::USD,
            },
            message: "two exchange rates were given for USD",
        },
        Expected {
            error: ExpenseGroupError::DuplicateExpense(expense()),
            message: "an expense with this identifier already exists",
        },
        Expected {
            error: ExpenseGroupError::DuplicateMember(member()),
            message: "a member with this identifier already exists",
        },
        Expected {
            error: ExpenseGroupError::DuplicateTransfer(transfer()),
            message: "a transfer with this identifier already exists",
        },
        Expected {
            error: ExpenseGroupError::EmptyName,
            message: "the group name must not be empty",
        },
        Expected {
            error: ExpenseGroupError::MemberInUse {
                member: member(),
                expenses: vec![
                    expense(),
                    ExpenseId::new("55555555-5555-5555-5555-555555555555").unwrap(),
                ],
                transfers: vec![transfer()],
            },
            message: "the member is used by 2 expenses and 1 transfer",
        },
        Expected {
            error: ExpenseGroupError::MissingExchangeRate {
                currency: CurrencyCode::USD,
            },
            message: "an exchange rate from USD into the group currency is required",
        },
        Expected {
            error: ExpenseGroupError::Overflow,
            message: "the group totals would be out of range",
        },
        Expected {
            error: ExpenseGroupError::UnexpectedExchangeRate,
            message: "an entry in the group currency cannot have an exchange rate",
        },
        Expected {
            error: ExpenseGroupError::UnknownCategory(category()),
            message: "no category has this identifier",
        },
        Expected {
            error: ExpenseGroupError::UnknownExpense(expense()),
            message: "no expense has this identifier",
        },
        Expected {
            error: ExpenseGroupError::UnknownMember(member()),
            message: "no member has this identifier",
        },
        Expected {
            error: ExpenseGroupError::UnknownTransfer(transfer()),
            message: "no transfer has this identifier",
        },
        Expected {
            error: ExpenseGroupError::UnsupportedFormat { format: 5 },
            message: "unsupported group format 5: this version reads format 4",
        },
    ]
}

/// Every `SettlementError` variant, with its exact message.
fn settlement_errors() -> Vec<Expected<SettlementError>> {
    vec![Expected {
        error: SettlementError::OutOfMemory { balances: 40 },
        message: "not enough memory to find the minimum transfers between 40 members",
    }]
}

/// Fails unless every error of `errors` prints its expected message.
fn assert_messages<Error: Display>(errors: Vec<Expected<Error>>) {
    for Expected { error, message } in errors {
        assert_eq!(error.to_string(), message);
    }
}

/// The expected messages of `errors`.
fn messages<Error>(errors: Vec<Expected<Error>>) -> impl Iterator<Item = &'static str> {
    errors.into_iter().map(|expected| expected.message)
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
    for word in input.split_whitespace().filter(|word| 3 <= word.len()) {
        assert!(!message.contains(word), "{message:?} repeats {word:?}");
    }
}

/// A reworded message would change what users read when an input, an entry,
/// an edit, or a repayment plan is refused, or the hint they get for a typo.
#[test]
fn every_message_is_pinned() {
    // One table per error type.
    assert_messages(id_errors());
    assert_messages(calendar_errors());
    assert_messages(money_errors());
    assert_messages(account_errors());
    assert_messages(iban_errors());
    assert_messages(contact_errors());
    assert_messages(member_errors());
    assert_messages(category_errors());
    assert_messages(language_errors());
    assert_messages(expense_errors());
    assert_messages(transfer_errors());
    assert_messages(group_errors());
    assert_messages(settlement_errors());
    // The IBAN reason is appended to the account failure's prefix.
    assert_eq!(
        PaymentAccountError::InvalidIban(IbanError::WrongLength {
            expected: 24,
            found: 20
        })
        .to_string(),
        "invalid IBAN: expected 24 characters for this country, found 20"
    );
}

/// A capitalised or punctuated message would read wrongly when the apps
/// embed it in a longer sentence.
#[test]
fn no_message_starts_like_a_sentence_or_ends_with_a_period() {
    // Lowercase start, or a currency code.
    for message in every_message() {
        assert_ne!(message, "");
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
        &Money::parse("12,345", CurrencyCode::EUR).unwrap_err(),
    );
    assert_not_echoed(
        "12.345",
        &Money::parse("12.345", CurrencyCode::EUR).unwrap_err(),
    );
    assert_not_echoed("33.333", &Percentage::parse("33.333").unwrap_err());
    assert_not_echoed("0.001", &Parts::parse("0.001").unwrap_err());
    assert_not_echoed("4,9765", &ExchangeRate::parse("4,9765").unwrap_err());
    // Codes, dates, identifiers, and tags.
    assert_not_echoed("XYZ", &CurrencyCode::from_code("XYZ").unwrap_err());
    assert_not_echoed(
        "2026-02-30",
        &CalendarDate::parse("2026-02-30").unwrap_err(),
    );
    assert_not_echoed("24:60", &TimeOfDay::parse("24:60").unwrap_err());
    for identifier in [" popescu", "popescu\tmarin"] {
        assert_not_echoed(identifier, &identifier.parse::<MemberId>().unwrap_err());
    }
    assert_not_echoed("ro_RO", &Language::parse("ro_RO").unwrap_err());
    assert_not_echoed("en-US-US", &Language::parse("en-US-US").unwrap_err());
}
