//! A group refuses to remove a member that entries still reference, and its
//! message counts those entries. These tests keep each count in agreement
//! with its noun, and keep every failure converting through
//! `FromExpenseGroupError` and `FromLanguageError` with what it holds and its
//! message. The messages themselves are pinned through the public API in
//! `tests/errors.rs`.

use super::*;

/// One variant in a table of conversions: the error, the method it converts
/// through, and what the conversion finds it holding.
#[derive(Debug)]
struct Case<Failure> {
    error: Failure,
    method: &'static str,
    held: String,
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

/// A [`LanguageError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedLanguageError {
    method: &'static str,
    held: String,
    message: String,
}

impl FromLanguageError for ConvertedLanguageError {
    fn invalid_language(message: String) -> Self {
        Self {
            method: "invalid_language",
            held: String::new(),
            message,
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds or its message, would show the apps another failure than the
/// library's.
#[test]
fn every_failure_of_language_error_converts_through_the_method_of_its_variant() {
    let cases = [Case {
        error: LanguageError::InvalidLanguage,
        method: "invalid_language",
        held: String::new(),
    }];
    for case in cases {
        assert_eq!(
            ConvertedLanguageError::from_language_error(case.error),
            ConvertedLanguageError {
                method: case.method,
                held: case.held,
                message: case.error.to_string(),
            }
        );
    }
}

/// A [`ExpenseGroupError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedExpenseGroupError {
    method: &'static str,
    held: String,
    message: String,
}

impl FromExpenseGroupError for ConvertedExpenseGroupError {
    fn unsupported_format(format: u32, message: String) -> Self {
        Self {
            method: "unsupported_format",
            held: format!("{format}"),
            message,
        }
    }

    fn empty_name(message: String) -> Self {
        Self {
            method: "empty_name",
            held: String::new(),
            message,
        }
    }

    fn missing_exchange_rate(currency: Currency, message: String) -> Self {
        Self {
            method: "missing_exchange_rate",
            held: format!("{currency}"),
            message,
        }
    }

    fn unexpected_exchange_rate(message: String) -> Self {
        Self {
            method: "unexpected_exchange_rate",
            held: String::new(),
            message,
        }
    }

    fn duplicate_exchange_rate(currency: Currency, message: String) -> Self {
        Self {
            method: "duplicate_exchange_rate",
            held: format!("{currency}"),
            message,
        }
    }

    fn duplicate_member(member: MemberId, message: String) -> Self {
        Self {
            method: "duplicate_member",
            held: format!("{member}"),
            message,
        }
    }

    fn unknown_member(member: MemberId, message: String) -> Self {
        Self {
            method: "unknown_member",
            held: format!("{member}"),
            message,
        }
    }

    fn archived_member(member: MemberId, message: String) -> Self {
        Self {
            method: "archived_member",
            held: format!("{member}"),
            message,
        }
    }

    fn member_in_use(
        member: MemberId,
        expenses: &[ExpenseId],
        transfers: &[TransferId],
        message: String,
    ) -> Self {
        Self {
            method: "member_in_use",
            held: format!("{member} {expenses:?} {transfers:?}"),
            message,
        }
    }

    fn duplicate_category(category: CategoryId, message: String) -> Self {
        Self {
            method: "duplicate_category",
            held: format!("{category}"),
            message,
        }
    }

    fn duplicate_category_name(existing: CategoryId, message: String) -> Self {
        Self {
            method: "duplicate_category_name",
            held: format!("{existing}"),
            message,
        }
    }

    fn unknown_category(category: CategoryId, message: String) -> Self {
        Self {
            method: "unknown_category",
            held: format!("{category}"),
            message,
        }
    }

    fn built_in_category(message: String) -> Self {
        Self {
            method: "built_in_category",
            held: String::new(),
            message,
        }
    }

    fn duplicate_expense(expense: ExpenseId, message: String) -> Self {
        Self {
            method: "duplicate_expense",
            held: format!("{expense}"),
            message,
        }
    }

    fn unknown_expense(expense: ExpenseId, message: String) -> Self {
        Self {
            method: "unknown_expense",
            held: format!("{expense}"),
            message,
        }
    }

    fn duplicate_transfer(transfer: TransferId, message: String) -> Self {
        Self {
            method: "duplicate_transfer",
            held: format!("{transfer}"),
            message,
        }
    }

    fn unknown_transfer(transfer: TransferId, message: String) -> Self {
        Self {
            method: "unknown_transfer",
            held: format!("{transfer}"),
            message,
        }
    }

    fn busy(message: String) -> Self {
        Self {
            method: "busy",
            held: String::new(),
            message,
        }
    }

    fn overflow(message: String) -> Self {
        Self {
            method: "overflow",
            held: String::new(),
            message,
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds or its message, would show the apps another failure than the
/// library's.
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one row for each of the nineteen variants, which read best as one table"
)]
fn every_failure_of_group_error_converts_through_the_method_of_its_variant() {
    let format = 11;
    let currency = Currency::EUR;
    let member = MemberId::from_bytes([1; 16]);
    let expenses = vec![ExpenseId::from_bytes([11; 16])];
    let transfers = vec![TransferId::from_bytes([14; 16])];
    let category = CategoryId::from_bytes([2; 16]);
    let existing = CategoryId::from_bytes([3; 16]);
    let expense = ExpenseId::from_bytes([5; 16]);
    let transfer = TransferId::from_bytes([6; 16]);
    let cases = [
        Case {
            error: ExpenseGroupError::UnsupportedFormat { format },
            method: "unsupported_format",
            held: format!("{format}"),
        },
        Case {
            error: ExpenseGroupError::EmptyName,
            method: "empty_name",
            held: String::new(),
        },
        Case {
            error: ExpenseGroupError::MissingExchangeRate { currency },
            method: "missing_exchange_rate",
            held: format!("{currency}"),
        },
        Case {
            error: ExpenseGroupError::UnexpectedExchangeRate,
            method: "unexpected_exchange_rate",
            held: String::new(),
        },
        Case {
            error: ExpenseGroupError::DuplicateExchangeRate { currency },
            method: "duplicate_exchange_rate",
            held: format!("{currency}"),
        },
        Case {
            error: ExpenseGroupError::DuplicateMember(member),
            method: "duplicate_member",
            held: format!("{member}"),
        },
        Case {
            error: ExpenseGroupError::UnknownMember(member),
            method: "unknown_member",
            held: format!("{member}"),
        },
        Case {
            error: ExpenseGroupError::ArchivedMember(member),
            method: "archived_member",
            held: format!("{member}"),
        },
        Case {
            error: ExpenseGroupError::MemberInUse {
                member,
                expenses: expenses.clone(),
                transfers: transfers.clone(),
            },
            method: "member_in_use",
            held: format!("{member} {expenses:?} {transfers:?}"),
        },
        Case {
            error: ExpenseGroupError::DuplicateCategory(category),
            method: "duplicate_category",
            held: format!("{category}"),
        },
        Case {
            error: ExpenseGroupError::DuplicateCategoryName { existing },
            method: "duplicate_category_name",
            held: format!("{existing}"),
        },
        Case {
            error: ExpenseGroupError::UnknownCategory(category),
            method: "unknown_category",
            held: format!("{category}"),
        },
        Case {
            error: ExpenseGroupError::BuiltInCategory,
            method: "built_in_category",
            held: String::new(),
        },
        Case {
            error: ExpenseGroupError::DuplicateExpense(expense),
            method: "duplicate_expense",
            held: format!("{expense}"),
        },
        Case {
            error: ExpenseGroupError::UnknownExpense(expense),
            method: "unknown_expense",
            held: format!("{expense}"),
        },
        Case {
            error: ExpenseGroupError::DuplicateTransfer(transfer),
            method: "duplicate_transfer",
            held: format!("{transfer}"),
        },
        Case {
            error: ExpenseGroupError::UnknownTransfer(transfer),
            method: "unknown_transfer",
            held: format!("{transfer}"),
        },
        Case {
            error: ExpenseGroupError::Busy,
            method: "busy",
            held: String::new(),
        },
        Case {
            error: ExpenseGroupError::Overflow,
            method: "overflow",
            held: String::new(),
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedExpenseGroupError::from_expense_group_error(&case.error),
            ConvertedExpenseGroupError {
                method: case.method,
                held: case.held,
                message: case.error.to_string(),
            }
        );
    }
}
