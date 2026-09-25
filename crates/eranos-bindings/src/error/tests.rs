//! The one error every binding reports. A failure converted to another
//! module's case, to another failure, with another payload, or without the
//! library's message would show the apps another failure than the
//! library's, so every variant of every error of the library is converted
//! here, and every case's message is read.

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
                expected: EranosError::Id {
                    detail: message,
                    failure: IdFailure::InvalidId,
                },
            }
        },
        {
            let error = CalendarError::InvalidDate;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Calendar {
                    detail: message,
                    failure: CalendarFailure::InvalidDate,
                },
            }
        },
        {
            let error = CalendarError::InvalidTime;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Calendar {
                    detail: message,
                    failure: CalendarFailure::InvalidTime,
                },
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
                expected: EranosError::Money {
                    detail: message,
                    failure: MoneyFailure::CurrencyMismatch {
                        left: "USD".to_owned(),
                        right: "GBP".to_owned(),
                    },
                },
            }
        },
        {
            let error = MoneyError::InvalidAmount;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Money {
                    detail: message,
                    failure: MoneyFailure::InvalidAmount,
                },
            }
        },
        {
            let error = MoneyError::InvalidExchangeRate;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Money {
                    detail: message,
                    failure: MoneyFailure::InvalidExchangeRate,
                },
            }
        },
        {
            let error = MoneyError::InvalidParts;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Money {
                    detail: message,
                    failure: MoneyFailure::InvalidParts,
                },
            }
        },
        {
            let error = MoneyError::InvalidPercentage;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Money {
                    detail: message,
                    failure: MoneyFailure::InvalidPercentage,
                },
            }
        },
        {
            let error = MoneyError::Overflow;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Money {
                    detail: message,
                    failure: MoneyFailure::Overflow,
                },
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
                expected: EranosError::Money {
                    detail: message,
                    failure: MoneyFailure::TooManyDecimals {
                        allowed: 3,
                        currency: "USD".to_owned(),
                    },
                },
            }
        },
        {
            let error = MoneyError::UnknownCurrency;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Money {
                    detail: message,
                    failure: MoneyFailure::UnknownCurrency,
                },
            }
        },
        {
            let error = AccountError::EmptyName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Account {
                    detail: message,
                    failure: AccountFailure::EmptyName,
                },
            }
        },
        {
            let error = AccountError::EmptyUsername;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Account {
                    detail: message,
                    failure: AccountFailure::EmptyUsername,
                },
            }
        },
        {
            let error = AccountError::InvalidIban(IbanError::InvalidChecksum);
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Account {
                    detail: message,
                    failure: AccountFailure::InvalidIban {
                        reason: IbanReason::InvalidChecksum,
                    },
                },
            }
        },
        {
            let error = ContactError::InvalidEmail;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Contact {
                    detail: message,
                    failure: ContactFailure::InvalidEmail,
                },
            }
        },
        {
            let error = ContactError::InvalidPhoneNumber;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Contact {
                    detail: message,
                    failure: ContactFailure::InvalidPhoneNumber,
                },
            }
        },
        {
            let error = MemberError::EmptyFirstName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Member {
                    detail: message,
                    failure: MemberFailure::EmptyFirstName,
                },
            }
        },
        {
            let error = CategoryError::EmptyName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Category {
                    detail: message,
                    failure: CategoryFailure::EmptyName,
                },
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
                expected: EranosError::Expense {
                    detail: message,
                    failure: ExpenseFailure::AmountsDoNotAddUp {
                        actual: Money {
                            currency: "EUR".to_owned(),
                            minor_units: 6,
                        },
                        expected: Money {
                            currency: "EUR".to_owned(),
                            minor_units: 7,
                        },
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
                expected: EranosError::Expense {
                    detail: message,
                    failure: ExpenseFailure::CurrencyMismatch {
                        expected: "GBP".to_owned(),
                        found: "RON".to_owned(),
                    },
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
                expected: EranosError::Expense {
                    detail: message,
                    failure: ExpenseFailure::DuplicateMember {
                        member: MemberId::from_bytes([10; 16]).to_string(),
                    },
                },
            }
        },
        {
            let error = ExpenseError::EmptyItemName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    detail: message,
                    failure: ExpenseFailure::EmptyItemName,
                },
            }
        },
        {
            let error = ExpenseError::EmptyName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    detail: message,
                    failure: ExpenseFailure::EmptyName,
                },
            }
        },
        {
            let error = ExpenseError::EmptySplit;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    detail: message,
                    failure: ExpenseFailure::EmptySplit,
                },
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
                expected: EranosError::Expense {
                    detail: message,
                    failure: ExpenseFailure::ItemsDoNotAddUp {
                        actual: Money {
                            currency: "EUR".to_owned(),
                            minor_units: 11,
                        },
                        expected: Money {
                            currency: "EUR".to_owned(),
                            minor_units: 12,
                        },
                    },
                },
            }
        },
        {
            let error = ExpenseError::NegativeTip;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    detail: message,
                    failure: ExpenseFailure::NegativeTip,
                },
            }
        },
        {
            let error = ExpenseError::NoItems;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    detail: message,
                    failure: ExpenseFailure::NoItems,
                },
            }
        },
        {
            let error = ExpenseError::NonPositiveAmount;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    detail: message,
                    failure: ExpenseFailure::NonPositiveAmount,
                },
            }
        },
        {
            let error = ExpenseError::NonPositiveShare {
                member: MemberId::from_bytes([13; 16]),
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    detail: message,
                    failure: ExpenseFailure::NonPositiveShare {
                        member: MemberId::from_bytes([13; 16]).to_string(),
                    },
                },
            }
        },
        {
            let error = ExpenseError::Overflow;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    detail: message,
                    failure: ExpenseFailure::Overflow,
                },
            }
        },
        {
            let error = ExpenseError::PercentagesDoNotAddUp {
                total_basis_points: 14,
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    detail: message,
                    failure: ExpenseFailure::PercentagesDoNotAddUp {
                        total_basis_points: 14,
                    },
                },
            }
        },
        {
            let error = ExpenseError::ZeroQuantity;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    detail: message,
                    failure: ExpenseFailure::ZeroQuantity,
                },
            }
        },
        {
            let error = SettlementError::OutOfMemory { balances: 15 };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Settlement {
                    detail: message,
                    failure: SettlementFailure::OutOfMemory { balances: 15 },
                },
            }
        },
        {
            let error = TransferError::NonPositiveAmount;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Transfer {
                    detail: message,
                    failure: TransferFailure::NonPositiveAmount,
                },
            }
        },
        {
            let error = TransferError::SameMember;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Transfer {
                    detail: message,
                    failure: TransferFailure::SameMember,
                },
            }
        },
        {
            let error = LanguageError::InvalidLanguage;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Language {
                    detail: message,
                    failure: LanguageFailure::InvalidLanguage,
                },
            }
        },
        {
            let error = GroupError::ArchivedMember(MemberId::from_bytes([16; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::ArchivedMember {
                        member: MemberId::from_bytes([16; 16]).to_string(),
                    },
                },
            }
        },
        {
            let error = GroupError::BuiltInCategory;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::BuiltInCategory,
                },
            }
        },
        {
            let error = GroupError::Busy;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::Busy,
                },
            }
        },
        {
            let error = GroupError::DuplicateCategory(CategoryId::from_bytes([17; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::DuplicateCategory {
                        category: CategoryId::from_bytes([17; 16]).to_string(),
                    },
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
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::DuplicateCategoryName {
                        existing: CategoryId::from_bytes([18; 16]).to_string(),
                    },
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
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::DuplicateExchangeRate {
                        currency: "USD".to_owned(),
                    },
                },
            }
        },
        {
            let error = GroupError::DuplicateExpense(ExpenseId::from_bytes([20; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::DuplicateExpense {
                        expense: ExpenseId::from_bytes([20; 16]).to_string(),
                    },
                },
            }
        },
        {
            let error = GroupError::DuplicateMember(MemberId::from_bytes([21; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::DuplicateMember {
                        member: MemberId::from_bytes([21; 16]).to_string(),
                    },
                },
            }
        },
        {
            let error = GroupError::DuplicateTransfer(TransferId::from_bytes([22; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::DuplicateTransfer {
                        transfer: TransferId::from_bytes([22; 16]).to_string(),
                    },
                },
            }
        },
        {
            let error = GroupError::EmptyName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::EmptyName,
                },
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
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::MemberInUse {
                        expenses: vec![ExpenseId::from_bytes([23; 16]).to_string()],
                        member: MemberId::from_bytes([24; 16]).to_string(),
                        transfers: vec![TransferId::from_bytes([25; 16]).to_string()],
                    },
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
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::MissingExchangeRate {
                        currency: "GBP".to_owned(),
                    },
                },
            }
        },
        {
            let error = GroupError::Overflow;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::Overflow,
                },
            }
        },
        {
            let error = GroupError::UnexpectedExchangeRate;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::UnexpectedExchangeRate,
                },
            }
        },
        {
            let error = GroupError::UnknownCategory(CategoryId::from_bytes([27; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::UnknownCategory {
                        category: CategoryId::from_bytes([27; 16]).to_string(),
                    },
                },
            }
        },
        {
            let error = GroupError::UnknownExpense(ExpenseId::from_bytes([28; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::UnknownExpense {
                        expense: ExpenseId::from_bytes([28; 16]).to_string(),
                    },
                },
            }
        },
        {
            let error = GroupError::UnknownMember(MemberId::from_bytes([29; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::UnknownMember {
                        member: MemberId::from_bytes([29; 16]).to_string(),
                    },
                },
            }
        },
        {
            let error = GroupError::UnknownTransfer(TransferId::from_bytes([30; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::UnknownTransfer {
                        transfer: TransferId::from_bytes([30; 16]).to_string(),
                    },
                },
            }
        },
        {
            let error = GroupError::UnsupportedFormat { format: 31 };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Group {
                    detail: message,
                    failure: GroupFailure::UnsupportedFormat { format: 31 },
                },
            }
        },
    ]
}

/// A conversion that reached another module's case or another failure,
/// dropped what the failure holds, or lost the message would show the apps
/// another failure than the library's; every case also reads back the
/// message it holds.
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
