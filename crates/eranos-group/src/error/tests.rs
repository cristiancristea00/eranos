//! A group refuses to remove a member that entries still reference, and its
//! message counts those entries. These tests keep each count in agreement
//! with its noun, and keep every failure converting through
//! `FromGroupError` and `FromLanguageError` with what it holds and its
//! message. The messages themselves are pinned through the public API in
//! `tests/errors.rs`.

use super::*;

/// One variant in a table of conversions: the error, what the conversion
/// finds it holding, and the method it converts through.
#[derive(Debug)]
struct Case<Failure> {
    error: Failure,
    held: String,
    method: &'static str,
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
    held: String,
    message: String,
    method: &'static str,
}

impl FromLanguageError for ConvertedLanguageError {
    fn invalid_language(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "invalid_language",
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
        held: String::new(),
        method: "invalid_language",
    }];
    for case in cases {
        assert_eq!(
            ConvertedLanguageError::from_language_error(case.error),
            ConvertedLanguageError {
                held: case.held,
                message: case.error.to_string(),
                method: case.method,
            }
        );
    }
}

/// A [`GroupError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedGroupError {
    held: String,
    message: String,
    method: &'static str,
}

impl FromGroupError for ConvertedGroupError {
    fn archived_member(member: MemberId, message: String) -> Self {
        Self {
            held: format!("{member}"),
            message,
            method: "archived_member",
        }
    }

    fn built_in_category(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "built_in_category",
        }
    }

    fn busy(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "busy",
        }
    }

    fn duplicate_category(category: CategoryId, message: String) -> Self {
        Self {
            held: format!("{category}"),
            message,
            method: "duplicate_category",
        }
    }

    fn duplicate_category_name(existing: CategoryId, message: String) -> Self {
        Self {
            held: format!("{existing}"),
            message,
            method: "duplicate_category_name",
        }
    }

    fn duplicate_exchange_rate(currency: Currency, message: String) -> Self {
        Self {
            held: format!("{currency}"),
            message,
            method: "duplicate_exchange_rate",
        }
    }

    fn duplicate_expense(expense: ExpenseId, message: String) -> Self {
        Self {
            held: format!("{expense}"),
            message,
            method: "duplicate_expense",
        }
    }

    fn duplicate_member(member: MemberId, message: String) -> Self {
        Self {
            held: format!("{member}"),
            message,
            method: "duplicate_member",
        }
    }

    fn duplicate_transfer(message: String, transfer: TransferId) -> Self {
        Self {
            held: format!("{transfer}"),
            message,
            method: "duplicate_transfer",
        }
    }

    fn empty_name(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "empty_name",
        }
    }

    fn member_in_use(
        expenses: &[ExpenseId],
        member: MemberId,
        message: String,
        transfers: &[TransferId],
    ) -> Self {
        Self {
            held: format!("{expenses:?} {member} {transfers:?}"),
            message,
            method: "member_in_use",
        }
    }

    fn missing_exchange_rate(currency: Currency, message: String) -> Self {
        Self {
            held: format!("{currency}"),
            message,
            method: "missing_exchange_rate",
        }
    }

    fn overflow(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "overflow",
        }
    }

    fn unexpected_exchange_rate(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "unexpected_exchange_rate",
        }
    }

    fn unknown_category(category: CategoryId, message: String) -> Self {
        Self {
            held: format!("{category}"),
            message,
            method: "unknown_category",
        }
    }

    fn unknown_expense(expense: ExpenseId, message: String) -> Self {
        Self {
            held: format!("{expense}"),
            message,
            method: "unknown_expense",
        }
    }

    fn unknown_member(member: MemberId, message: String) -> Self {
        Self {
            held: format!("{member}"),
            message,
            method: "unknown_member",
        }
    }

    fn unknown_transfer(message: String, transfer: TransferId) -> Self {
        Self {
            held: format!("{transfer}"),
            message,
            method: "unknown_transfer",
        }
    }

    fn unsupported_format(format: u32, message: String) -> Self {
        Self {
            held: format!("{format}"),
            message,
            method: "unsupported_format",
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
    let member = MemberId::from_bytes([1; 16]);
    let category = CategoryId::from_bytes([2; 16]);
    let existing = CategoryId::from_bytes([3; 16]);
    let currency = Currency::EUR;
    let expense = ExpenseId::from_bytes([5; 16]);
    let transfer = TransferId::from_bytes([6; 16]);
    let expenses = vec![ExpenseId::from_bytes([11; 16])];
    let transfers = vec![TransferId::from_bytes([14; 16])];
    let format = 11;
    let cases = [
        Case {
            error: GroupError::ArchivedMember(member),
            held: format!("{member}"),
            method: "archived_member",
        },
        Case {
            error: GroupError::BuiltInCategory,
            held: String::new(),
            method: "built_in_category",
        },
        Case {
            error: GroupError::Busy,
            held: String::new(),
            method: "busy",
        },
        Case {
            error: GroupError::DuplicateCategory(category),
            held: format!("{category}"),
            method: "duplicate_category",
        },
        Case {
            error: GroupError::DuplicateCategoryName { existing },
            held: format!("{existing}"),
            method: "duplicate_category_name",
        },
        Case {
            error: GroupError::DuplicateExchangeRate { currency },
            held: format!("{currency}"),
            method: "duplicate_exchange_rate",
        },
        Case {
            error: GroupError::DuplicateExpense(expense),
            held: format!("{expense}"),
            method: "duplicate_expense",
        },
        Case {
            error: GroupError::DuplicateMember(member),
            held: format!("{member}"),
            method: "duplicate_member",
        },
        Case {
            error: GroupError::DuplicateTransfer(transfer),
            held: format!("{transfer}"),
            method: "duplicate_transfer",
        },
        Case {
            error: GroupError::EmptyName,
            held: String::new(),
            method: "empty_name",
        },
        Case {
            error: GroupError::MemberInUse {
                expenses: expenses.clone(),
                member,
                transfers: transfers.clone(),
            },
            held: format!("{expenses:?} {member} {transfers:?}"),
            method: "member_in_use",
        },
        Case {
            error: GroupError::MissingExchangeRate { currency },
            held: format!("{currency}"),
            method: "missing_exchange_rate",
        },
        Case {
            error: GroupError::Overflow,
            held: String::new(),
            method: "overflow",
        },
        Case {
            error: GroupError::UnexpectedExchangeRate,
            held: String::new(),
            method: "unexpected_exchange_rate",
        },
        Case {
            error: GroupError::UnknownCategory(category),
            held: format!("{category}"),
            method: "unknown_category",
        },
        Case {
            error: GroupError::UnknownExpense(expense),
            held: format!("{expense}"),
            method: "unknown_expense",
        },
        Case {
            error: GroupError::UnknownMember(member),
            held: format!("{member}"),
            method: "unknown_member",
        },
        Case {
            error: GroupError::UnknownTransfer(transfer),
            held: format!("{transfer}"),
            method: "unknown_transfer",
        },
        Case {
            error: GroupError::UnsupportedFormat { format },
            held: format!("{format}"),
            method: "unsupported_format",
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedGroupError::from_group_error(&case.error),
            ConvertedGroupError {
                held: case.held,
                message: case.error.to_string(),
                method: case.method,
            }
        );
    }
}
