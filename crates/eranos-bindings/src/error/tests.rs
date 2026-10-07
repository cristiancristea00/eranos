//! The one error every binding reports. A failure converted to another
//! module's case, to another failure, with another payload, or without the
//! library's message would show the apps another failure than the
//! library's, so every variant of every error of the library is converted
//! here, and every case's message is read.

use eranos::error::{
    CalendarError, CategoryError, ContactError, ExpenseError, ExpenseGroupError, IbanError,
    IdError, LanguageError, MemberError, MoneyError, PaymentAccountError, SettlementError,
    TransferError,
};
use eranos::{CategoryId, CurrencyCode, ExpenseId, MemberId, TransferId};

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
                    failure: IdFailure::InvalidId,
                    detail: message,
                },
            }
        },
        {
            let error = CalendarError::InvalidDate;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Calendar {
                    failure: CalendarFailure::InvalidDate,
                    detail: message,
                },
            }
        },
        {
            let error = CalendarError::InvalidTime;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Calendar {
                    failure: CalendarFailure::InvalidTime,
                    detail: message,
                },
            }
        },
        {
            let error = MoneyError::CurrencyMismatch {
                left: CurrencyCode::USD,
                right: CurrencyCode::GBP,
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Money {
                    failure: MoneyFailure::CurrencyMismatch {
                        left: "USD".to_owned(),
                        right: "GBP".to_owned(),
                    },
                    detail: message,
                },
            }
        },
        {
            let error = MoneyError::InvalidAmount;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Money {
                    failure: MoneyFailure::InvalidAmount,
                    detail: message,
                },
            }
        },
        {
            let error = MoneyError::InvalidExchangeRate;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Money {
                    failure: MoneyFailure::InvalidExchangeRate,
                    detail: message,
                },
            }
        },
        {
            let error = MoneyError::InvalidParts;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Money {
                    failure: MoneyFailure::InvalidParts,
                    detail: message,
                },
            }
        },
        {
            let error = MoneyError::InvalidPercentage;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Money {
                    failure: MoneyFailure::InvalidPercentage,
                    detail: message,
                },
            }
        },
        {
            let error = MoneyError::Overflow;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Money {
                    failure: MoneyFailure::Overflow,
                    detail: message,
                },
            }
        },
        {
            let error = MoneyError::TooManyDecimals {
                currency: CurrencyCode::USD,
                allowed: 3,
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Money {
                    failure: MoneyFailure::TooManyDecimals {
                        currency: "USD".to_owned(),
                        allowed: 3,
                    },
                    detail: message,
                },
            }
        },
        {
            let error = MoneyError::UnknownCurrency;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Money {
                    failure: MoneyFailure::UnknownCurrency,
                    detail: message,
                },
            }
        },
        {
            let error = PaymentAccountError::EmptyName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::PaymentAccount {
                    failure: PaymentAccountFailure::EmptyName,
                    detail: message,
                },
            }
        },
        {
            let error = PaymentAccountError::EmptyUsername;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::PaymentAccount {
                    failure: PaymentAccountFailure::EmptyUsername,
                    detail: message,
                },
            }
        },
        {
            let error = PaymentAccountError::InvalidIban(IbanError::InvalidChecksum);
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::PaymentAccount {
                    failure: PaymentAccountFailure::InvalidIban {
                        reason: IbanReason::InvalidChecksum,
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ContactError::InvalidEmail;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Contact {
                    failure: ContactFailure::InvalidEmail,
                    detail: message,
                },
            }
        },
        {
            let error = ContactError::InvalidPhoneNumber;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Contact {
                    failure: ContactFailure::InvalidPhoneNumber,
                    detail: message,
                },
            }
        },
        {
            let error = MemberError::EmptyFirstName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Member {
                    failure: MemberFailure::EmptyFirstName,
                    detail: message,
                },
            }
        },
        {
            let error = CategoryError::EmptyName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Category {
                    failure: CategoryFailure::EmptyName,
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseError::AmountsDoNotAddUp {
                expected: eranos::Money::from_minor_units(7, CurrencyCode::EUR),
                actual: eranos::Money::from_minor_units(6, CurrencyCode::EUR),
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    failure: ExpenseFailure::AmountsDoNotAddUp {
                        expected: Money {
                            minor_units: 7,
                            currency: "EUR".to_owned(),
                        },
                        actual: Money {
                            minor_units: 6,
                            currency: "EUR".to_owned(),
                        },
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseError::CurrencyMismatch {
                expected: CurrencyCode::GBP,
                found: CurrencyCode::RON,
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    failure: ExpenseFailure::CurrencyMismatch {
                        expected: "GBP".to_owned(),
                        found: "RON".to_owned(),
                    },
                    detail: message,
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
                    failure: ExpenseFailure::DuplicateMember {
                        member: MemberId::from_bytes([10; 16]).to_string(),
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseError::EmptyItemName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    failure: ExpenseFailure::EmptyItemName,
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseError::EmptyName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    failure: ExpenseFailure::EmptyName,
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseError::EmptySplit;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    failure: ExpenseFailure::EmptySplit,
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseError::ItemsDoNotAddUp {
                expected: eranos::Money::from_minor_units(12, CurrencyCode::EUR),
                actual: eranos::Money::from_minor_units(11, CurrencyCode::EUR),
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    failure: ExpenseFailure::ItemsDoNotAddUp {
                        expected: Money {
                            minor_units: 12,
                            currency: "EUR".to_owned(),
                        },
                        actual: Money {
                            minor_units: 11,
                            currency: "EUR".to_owned(),
                        },
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseError::NegativeTip;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    failure: ExpenseFailure::NegativeTip,
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseError::NoItems;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    failure: ExpenseFailure::NoItems,
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseError::NonPositiveAmount;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    failure: ExpenseFailure::NonPositiveAmount,
                    detail: message,
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
                    failure: ExpenseFailure::NonPositiveShare {
                        member: MemberId::from_bytes([13; 16]).to_string(),
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseError::Overflow;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    failure: ExpenseFailure::Overflow,
                    detail: message,
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
                    failure: ExpenseFailure::PercentagesDoNotAddUp {
                        total_basis_points: 14,
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseError::ZeroQuantity;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Expense {
                    failure: ExpenseFailure::ZeroQuantity,
                    detail: message,
                },
            }
        },
        {
            let error = SettlementError::OutOfMemory { balances: 15 };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Settlement {
                    failure: SettlementFailure::OutOfMemory { balances: 15 },
                    detail: message,
                },
            }
        },
        {
            let error = TransferError::NonPositiveAmount;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Transfer {
                    failure: TransferFailure::NonPositiveAmount,
                    detail: message,
                },
            }
        },
        {
            let error = TransferError::SameMember;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Transfer {
                    failure: TransferFailure::SameMember,
                    detail: message,
                },
            }
        },
        {
            let error = LanguageError::InvalidLanguage;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::Language {
                    failure: LanguageFailure::InvalidLanguage,
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::ArchivedMember(MemberId::from_bytes([16; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::ArchivedMember {
                        member: MemberId::from_bytes([16; 16]).to_string(),
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::BuiltInCategory;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::BuiltInCategory,
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::Busy;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::Busy,
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::DuplicateCategory(CategoryId::from_bytes([17; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::DuplicateCategory {
                        category: CategoryId::from_bytes([17; 16]).to_string(),
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::DuplicateCategoryName {
                existing: CategoryId::from_bytes([18; 16]),
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::DuplicateCategoryName {
                        existing: CategoryId::from_bytes([18; 16]).to_string(),
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::DuplicateExchangeRate {
                currency: CurrencyCode::USD,
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::DuplicateExchangeRate {
                        currency: "USD".to_owned(),
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::DuplicateExpense(ExpenseId::from_bytes([20; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::DuplicateExpense {
                        expense: ExpenseId::from_bytes([20; 16]).to_string(),
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::DuplicateMember(MemberId::from_bytes([21; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::DuplicateMember {
                        member: MemberId::from_bytes([21; 16]).to_string(),
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::DuplicateTransfer(TransferId::from_bytes([22; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::DuplicateTransfer {
                        transfer: TransferId::from_bytes([22; 16]).to_string(),
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::EmptyName;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::EmptyName,
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::MemberInUse {
                member: MemberId::from_bytes([24; 16]),
                expenses: vec![ExpenseId::from_bytes([23; 16])],
                transfers: vec![TransferId::from_bytes([25; 16])],
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::MemberInUse {
                        member: MemberId::from_bytes([24; 16]).to_string(),
                        expenses: vec![ExpenseId::from_bytes([23; 16]).to_string()],
                        transfers: vec![TransferId::from_bytes([25; 16]).to_string()],
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::MissingExchangeRate {
                currency: CurrencyCode::GBP,
            };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::MissingExchangeRate {
                        currency: "GBP".to_owned(),
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::Overflow;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::Overflow,
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::UnexpectedExchangeRate;
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnexpectedExchangeRate,
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::UnknownCategory(CategoryId::from_bytes([27; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownCategory {
                        category: CategoryId::from_bytes([27; 16]).to_string(),
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::UnknownExpense(ExpenseId::from_bytes([28; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownExpense {
                        expense: ExpenseId::from_bytes([28; 16]).to_string(),
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::UnknownMember(MemberId::from_bytes([29; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownMember {
                        member: MemberId::from_bytes([29; 16]).to_string(),
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::UnknownTransfer(TransferId::from_bytes([30; 16]));
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownTransfer {
                        transfer: TransferId::from_bytes([30; 16]).to_string(),
                    },
                    detail: message,
                },
            }
        },
        {
            let error = ExpenseGroupError::UnsupportedFormat { format: 31 };
            let message = error.to_string();
            Conversion {
                converted: EranosError::from(error),
                expected: EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnsupportedFormat { format: 31 },
                    detail: message,
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
        assert_ne!(converted.detail(), "");
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

/// A reason the library refuses an IBAN for, beside the reason it must
/// become.
struct ReasonConversion {
    error: IbanError,
    reason: IbanReason,
}

/// An IBAN refused for another reason than the library's would send the
/// user to fix the wrong thing.
#[test]
fn every_reason_of_a_refused_iban_converts() {
    let reasons = [
        ReasonConversion {
            error: IbanError::InvalidCharacter,
            reason: IbanReason::InvalidCharacter,
        },
        ReasonConversion {
            error: IbanError::InvalidChecksum,
            reason: IbanReason::InvalidChecksum,
        },
        ReasonConversion {
            error: IbanError::InvalidFormat,
            reason: IbanReason::InvalidFormat,
        },
        ReasonConversion {
            error: IbanError::InvalidStructure,
            reason: IbanReason::InvalidStructure,
        },
        ReasonConversion {
            error: IbanError::UnknownCountry,
            reason: IbanReason::UnknownCountry,
        },
        ReasonConversion {
            error: IbanError::WrongLength {
                expected: 24,
                found: 22,
            },
            reason: IbanReason::WrongLength {
                expected: 24,
                found: 22,
            },
        },
    ];
    for ReasonConversion { error, reason } in reasons {
        assert_eq!(IbanReason::from_iban_error(&error), reason);
    }
}

/// A saved group the reader refuses would be reported without where, or with
/// the text at that place, which can hold an account number.
#[test]
fn a_refused_saved_group_names_the_fault_and_its_place() {
    let refusal = |text: &str| {
        let error =
            serde_json::from_str::<eranos::expense_group::SavedExpenseGroup>(text).unwrap_err();
        EranosError::invalid_json(&error)
    };
    assert_eq!(
        refusal("{]"),
        EranosError::InvalidJson {
            line: 1,
            column: 2,
            detail: "the saved group is not valid JSON, at line 1, column 2".to_owned(),
        }
    );
    assert_eq!(
        refusal("{\"format\": 2"),
        EranosError::InvalidJson {
            line: 1,
            column: 12,
            detail: "the saved group ends early, at line 1, column 12".to_owned(),
        }
    );
    let secret = refusal("{\"format\": \"RO49AAAA1B31007593840000\"}");
    assert_eq!(
        secret.detail(),
        "the saved group has a value its format does not allow, at line 1, column 37"
    );
}
