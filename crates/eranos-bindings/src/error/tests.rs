//! The one error every binding reports. A failure converted to another case,
//! with another payload, or without the library's message would show the
//! apps another failure than the library's, so every variant of every error
//! of the library is converted here, and every case's message is read.

use eranos::error::{
    AccountError, CalendarError, CategoryError, ContactError, ExpenseError, GroupError, IbanError,
    IdError, LanguageError, MemberError, MoneyError, SettlementError, TransferError,
};
use eranos::{CategoryId, Currency, ExpenseId, MemberId, TransferId};

use super::*;

/// A failure of the library, converted, beside the case it must become.
struct Conversion {
    converted: EranosError,
    expected: EranosError,
}

/// Every failure of the library, converted.
#[expect(
    clippy::too_many_lines,
    reason = "one entry for each of the 55 variants of the library's errors, which read best as one table"
)]
fn conversions() -> Vec<Conversion> {
    vec![
        {
            let error = IdError::InvalidId;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::InvalidId { detail: message },
            }
        },
        {
            let error = CalendarError::InvalidDate;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::InvalidDate { detail: message },
            }
        },
        {
            let error = CalendarError::InvalidTime;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::InvalidTime { detail: message },
            }
        },
        {
            let error = MoneyError::CurrencyMismatch {
                left: Currency::USD,
                right: Currency::GBP,
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::MoneyCurrencyMismatch {
                    detail: message,
                    left: "USD".to_owned(),
                    right: "GBP".to_owned(),
                },
            }
        },
        {
            let error = MoneyError::InvalidAmount;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::InvalidAmount { detail: message },
            }
        },
        {
            let error = MoneyError::InvalidExchangeRate;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::InvalidExchangeRate { detail: message },
            }
        },
        {
            let error = MoneyError::InvalidParts;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::InvalidParts { detail: message },
            }
        },
        {
            let error = MoneyError::InvalidPercentage;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::InvalidPercentage { detail: message },
            }
        },
        {
            let error = MoneyError::Overflow;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::MoneyOverflow { detail: message },
            }
        },
        {
            let error = MoneyError::TooManyDecimals {
                allowed: 3,
                currency: Currency::USD,
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::TooManyDecimals {
                    allowed: 3,
                    currency: "USD".to_owned(),
                    detail: message,
                },
            }
        },
        {
            let error = MoneyError::UnknownCurrency;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::UnknownCurrency { detail: message },
            }
        },
        {
            let error = AccountError::EmptyName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::AccountEmptyName { detail: message },
            }
        },
        {
            let error = AccountError::EmptyUsername;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::EmptyUsername { detail: message },
            }
        },
        {
            let error = AccountError::InvalidIban(IbanError::InvalidChecksum);
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::InvalidIban {
                    detail: message,
                    reason: IbanReason::InvalidChecksum,
                },
            }
        },
        {
            let error = ContactError::InvalidEmail;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::InvalidEmail { detail: message },
            }
        },
        {
            let error = ContactError::InvalidPhoneNumber;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::InvalidPhoneNumber { detail: message },
            }
        },
        {
            let error = MemberError::EmptyFirstName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::EmptyFirstName { detail: message },
            }
        },
        {
            let error = CategoryError::EmptyName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::CategoryEmptyName { detail: message },
            }
        },
        {
            let error = ExpenseError::AmountsDoNotAddUp {
                actual: eranos::Money::from_minor_units(Currency::EUR, 6),
                expected: eranos::Money::from_minor_units(Currency::EUR, 7),
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::AmountsDoNotAddUp {
                    actual: Money {
                        currency: "EUR".to_owned(),
                        minor_units: 6,
                    },
                    detail: message,
                    expected: Money {
                        currency: "EUR".to_owned(),
                        minor_units: 7,
                    },
                },
            }
        },
        {
            let error = ExpenseError::CurrencyMismatch {
                expected: Currency::GBP,
                found: Currency::RON,
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseCurrencyMismatch {
                    detail: message,
                    expected: "GBP".to_owned(),
                    found: "RON".to_owned(),
                },
            }
        },
        {
            let error = ExpenseError::DuplicateMember {
                member: MemberId::from_bytes([10; 16]),
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseDuplicateMember {
                    detail: message,
                    member: MemberId::from_bytes([10; 16]).to_string(),
                },
            }
        },
        {
            let error = ExpenseError::EmptyItemName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::EmptyItemName { detail: message },
            }
        },
        {
            let error = ExpenseError::EmptyName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseEmptyName { detail: message },
            }
        },
        {
            let error = ExpenseError::EmptySplit;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::EmptySplit { detail: message },
            }
        },
        {
            let error = ExpenseError::ItemsDoNotAddUp {
                actual: eranos::Money::from_minor_units(Currency::EUR, 11),
                expected: eranos::Money::from_minor_units(Currency::EUR, 12),
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ItemsDoNotAddUp {
                    actual: Money {
                        currency: "EUR".to_owned(),
                        minor_units: 11,
                    },
                    detail: message,
                    expected: Money {
                        currency: "EUR".to_owned(),
                        minor_units: 12,
                    },
                },
            }
        },
        {
            let error = ExpenseError::NegativeTip;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::NegativeTip { detail: message },
            }
        },
        {
            let error = ExpenseError::NoItems;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::NoItems { detail: message },
            }
        },
        {
            let error = ExpenseError::NonPositiveAmount;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseNonPositiveAmount { detail: message },
            }
        },
        {
            let error = ExpenseError::NonPositiveShare {
                member: MemberId::from_bytes([13; 16]),
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::NonPositiveShare {
                    detail: message,
                    member: MemberId::from_bytes([13; 16]).to_string(),
                },
            }
        },
        {
            let error = ExpenseError::Overflow;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseOverflow { detail: message },
            }
        },
        {
            let error = ExpenseError::PercentagesDoNotAddUp {
                total_basis_points: 14,
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::PercentagesDoNotAddUp {
                    detail: message,
                    total_basis_points: 14,
                },
            }
        },
        {
            let error = ExpenseError::ZeroQuantity;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ZeroQuantity { detail: message },
            }
        },
        {
            let error = SettlementError::OutOfMemory { balances: 15 };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::OutOfMemory {
                    balances: 15,
                    detail: message,
                },
            }
        },
        {
            let error = TransferError::NonPositiveAmount;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::TransferNonPositiveAmount { detail: message },
            }
        },
        {
            let error = TransferError::SameMember;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::SameMember { detail: message },
            }
        },
        {
            let error = LanguageError::InvalidLanguage;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::InvalidLanguage { detail: message },
            }
        },
        {
            let error = GroupError::ArchivedMember(MemberId::from_bytes([16; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ArchivedMember {
                    detail: message,
                    member: MemberId::from_bytes([16; 16]).to_string(),
                },
            }
        },
        {
            let error = GroupError::BuiltInCategory;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::BuiltInCategory { detail: message },
            }
        },
        {
            let error = GroupError::Busy;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Busy { detail: message },
            }
        },
        {
            let error = GroupError::DuplicateCategory(CategoryId::from_bytes([17; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::DuplicateCategory {
                    category: CategoryId::from_bytes([17; 16]).to_string(),
                    detail: message,
                },
            }
        },
        {
            let error = GroupError::DuplicateCategoryName {
                existing: CategoryId::from_bytes([18; 16]),
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::DuplicateCategoryName {
                    detail: message,
                    existing: CategoryId::from_bytes([18; 16]).to_string(),
                },
            }
        },
        {
            let error = GroupError::DuplicateExchangeRate {
                currency: Currency::USD,
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::DuplicateExchangeRate {
                    currency: "USD".to_owned(),
                    detail: message,
                },
            }
        },
        {
            let error = GroupError::DuplicateExpense(ExpenseId::from_bytes([20; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::DuplicateExpense {
                    detail: message,
                    expense: ExpenseId::from_bytes([20; 16]).to_string(),
                },
            }
        },
        {
            let error = GroupError::DuplicateMember(MemberId::from_bytes([21; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::GroupDuplicateMember {
                    detail: message,
                    member: MemberId::from_bytes([21; 16]).to_string(),
                },
            }
        },
        {
            let error = GroupError::DuplicateTransfer(TransferId::from_bytes([22; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::DuplicateTransfer {
                    detail: message,
                    transfer: TransferId::from_bytes([22; 16]).to_string(),
                },
            }
        },
        {
            let error = GroupError::EmptyName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::GroupEmptyName { detail: message },
            }
        },
        {
            let error = GroupError::MemberInUse {
                expenses: vec![ExpenseId::from_bytes([23; 16])],
                member: MemberId::from_bytes([24; 16]),
                transfers: vec![TransferId::from_bytes([25; 16])],
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::MemberInUse {
                    detail: message,
                    expenses: vec![ExpenseId::from_bytes([23; 16]).to_string()],
                    member: MemberId::from_bytes([24; 16]).to_string(),
                    transfers: vec![TransferId::from_bytes([25; 16]).to_string()],
                },
            }
        },
        {
            let error = GroupError::MissingExchangeRate {
                currency: Currency::GBP,
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::MissingExchangeRate {
                    currency: "GBP".to_owned(),
                    detail: message,
                },
            }
        },
        {
            let error = GroupError::Overflow;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::GroupOverflow { detail: message },
            }
        },
        {
            let error = GroupError::UnexpectedExchangeRate;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::UnexpectedExchangeRate { detail: message },
            }
        },
        {
            let error = GroupError::UnknownCategory(CategoryId::from_bytes([27; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::UnknownCategory {
                    category: CategoryId::from_bytes([27; 16]).to_string(),
                    detail: message,
                },
            }
        },
        {
            let error = GroupError::UnknownExpense(ExpenseId::from_bytes([28; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::UnknownExpense {
                    detail: message,
                    expense: ExpenseId::from_bytes([28; 16]).to_string(),
                },
            }
        },
        {
            let error = GroupError::UnknownMember(MemberId::from_bytes([29; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::UnknownMember {
                    detail: message,
                    member: MemberId::from_bytes([29; 16]).to_string(),
                },
            }
        },
        {
            let error = GroupError::UnknownTransfer(TransferId::from_bytes([30; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::UnknownTransfer {
                    detail: message,
                    transfer: TransferId::from_bytes([30; 16]).to_string(),
                },
            }
        },
        {
            let error = GroupError::UnsupportedFormat { format: 31 };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::UnsupportedFormat {
                    detail: message,
                    format: 31,
                },
            }
        },
    ]
}

/// A conversion that reached another case, dropped what the failure holds,
/// or lost the message would show the apps another failure than the
/// library's; every case also reads back the message it holds.
#[test]
fn every_failure_of_the_library_converts_to_its_case() {
    let conversions = conversions();
    assert_eq!(conversions.len(), 55);
    for Conversion {
        converted,
        expected,
    } in conversions
    {
        assert_eq!(converted, expected);
        assert!(!converted.detail().is_empty());
        assert_eq!(converted.to_string(), converted.detail());
    }
}

/// The binding's own cases would show an empty or a changed message.
#[test]
fn the_bindings_own_cases_carry_their_messages() {
    assert_eq!(EranosError::cancelled().detail(), "the call was cancelled");
    assert_eq!(
        EranosError::identifier_given().detail(),
        "an entry to add must not carry an identifier"
    );
    assert_eq!(
        EranosError::identifier_missing().detail(),
        "an entry to replace must carry its identifier"
    );
    assert_eq!(
        EranosError::released().detail(),
        "the read handle was released"
    );
}

/// An IBAN refused for another reason than the library's would send the
/// user to fix the wrong thing.
#[test]
fn every_reason_of_a_refused_iban_converts() {
    let reasons = [
        (IbanError::InvalidCharacter, IbanReason::InvalidCharacter),
        (IbanError::InvalidChecksum, IbanReason::InvalidChecksum),
        (IbanError::InvalidFormat, IbanReason::InvalidFormat),
        (IbanError::InvalidStructure, IbanReason::InvalidStructure),
        (IbanError::UnknownCountry, IbanReason::UnknownCountry),
        (
            IbanError::WrongLength {
                expected: 24,
                found: 22,
            },
            IbanReason::WrongLength {
                expected: 24,
                found: 22,
            },
        ),
    ];
    for (error, reason) in reasons {
        assert_eq!(IbanReason::from_iban_error(&error), reason);
    }
}

/// A saved group the reader refuses would be reported without where, or with
/// the text at that place, which can hold an account number.
#[test]
fn a_refused_saved_group_names_the_fault_and_its_place() {
    let refusal = |text: &str| {
        let error = serde_json::from_str::<eranos::group::SavedGroup>(text).unwrap_err();
        EranosError::invalid_json(&error)
    };
    assert_eq!(
        refusal("{]"),
        EranosError::InvalidJson {
            column: 2,
            detail: "the saved group is not valid JSON, at line 1, column 2".to_owned(),
            line: 1,
        }
    );
    assert_eq!(
        refusal("{\"format\": 2"),
        EranosError::InvalidJson {
            column: 12,
            detail: "the saved group ends early, at line 1, column 12".to_owned(),
            line: 1,
        }
    );
    let secret = refusal("{\"format\": \"RO49AAAA1B31007593840000\"}");
    assert_eq!(
        secret.detail(),
        "the saved group has a value its format does not allow, at line 1, column 37"
    );
}
