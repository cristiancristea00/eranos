//! Every failure of this crate converts through `FromExpenseError`, the trait
//! through which a crate outside this one, such as the binding core, reports
//! it. These tests keep each variant reaching its own method with what it
//! holds and its message.

use super::*;

/// One variant in a table of conversions: the error, the method it converts
/// through, and what the conversion finds it holding.
#[derive(Debug)]
struct Case {
    error: ExpenseError,
    method: &'static str,
    held: String,
}

/// An [`ExpenseError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedExpenseError {
    method: &'static str,
    held: String,
    message: String,
}

impl FromExpenseError for ConvertedExpenseError {
    fn empty_name(message: String) -> Self {
        Self {
            method: "empty_name",
            held: String::new(),
            message,
        }
    }

    fn non_positive_amount(message: String) -> Self {
        Self {
            method: "non_positive_amount",
            held: String::new(),
            message,
        }
    }

    fn currency_mismatch(expected: CurrencyCode, found: CurrencyCode, message: String) -> Self {
        Self {
            method: "currency_mismatch",
            held: format!("{expected} {found}"),
            message,
        }
    }

    fn empty_split(message: String) -> Self {
        Self {
            method: "empty_split",
            held: String::new(),
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

    fn non_positive_share(member: MemberId, message: String) -> Self {
        Self {
            method: "non_positive_share",
            held: format!("{member}"),
            message,
        }
    }

    fn amounts_do_not_add_up(expected: Money, actual: Money, message: String) -> Self {
        Self {
            method: "amounts_do_not_add_up",
            held: format!("{expected} {actual}"),
            message,
        }
    }

    fn percentages_do_not_add_up(total_basis_points: u64, message: String) -> Self {
        Self {
            method: "percentages_do_not_add_up",
            held: format!("{total_basis_points}"),
            message,
        }
    }

    fn no_items(message: String) -> Self {
        Self {
            method: "no_items",
            held: String::new(),
            message,
        }
    }

    fn empty_item_name(message: String) -> Self {
        Self {
            method: "empty_item_name",
            held: String::new(),
            message,
        }
    }

    fn zero_quantity(message: String) -> Self {
        Self {
            method: "zero_quantity",
            held: String::new(),
            message,
        }
    }

    fn items_do_not_add_up(expected: Money, actual: Money, message: String) -> Self {
        Self {
            method: "items_do_not_add_up",
            held: format!("{expected} {actual}"),
            message,
        }
    }

    fn negative_tip(message: String) -> Self {
        Self {
            method: "negative_tip",
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
fn every_failure_of_expense_error_converts_through_the_method_of_its_variant() {
    let expected_currency = CurrencyCode::USD;
    let found = CurrencyCode::EUR;
    let member = MemberId::from_bytes([5; 16]);
    let expected_money = Money::from_minor_units(200, CurrencyCode::EUR);
    let actual = Money::from_minor_units(100, CurrencyCode::EUR);
    let total_basis_points = 9005;
    let cases = [
        Case {
            error: ExpenseError::EmptyName,
            method: "empty_name",
            held: String::new(),
        },
        Case {
            error: ExpenseError::NonPositiveAmount,
            method: "non_positive_amount",
            held: String::new(),
        },
        Case {
            error: ExpenseError::CurrencyMismatch {
                expected: expected_currency,
                found,
            },
            method: "currency_mismatch",
            held: format!("{expected_currency} {found}"),
        },
        Case {
            error: ExpenseError::EmptySplit,
            method: "empty_split",
            held: String::new(),
        },
        Case {
            error: ExpenseError::DuplicateMember { member },
            method: "duplicate_member",
            held: format!("{member}"),
        },
        Case {
            error: ExpenseError::NonPositiveShare { member },
            method: "non_positive_share",
            held: format!("{member}"),
        },
        Case {
            error: ExpenseError::AmountsDoNotAddUp {
                expected: expected_money,
                actual,
            },
            method: "amounts_do_not_add_up",
            held: format!("{expected_money} {actual}"),
        },
        Case {
            error: ExpenseError::PercentagesDoNotAddUp { total_basis_points },
            method: "percentages_do_not_add_up",
            held: format!("{total_basis_points}"),
        },
        Case {
            error: ExpenseError::NoItems,
            method: "no_items",
            held: String::new(),
        },
        Case {
            error: ExpenseError::EmptyItemName,
            method: "empty_item_name",
            held: String::new(),
        },
        Case {
            error: ExpenseError::ZeroQuantity,
            method: "zero_quantity",
            held: String::new(),
        },
        Case {
            error: ExpenseError::ItemsDoNotAddUp {
                expected: expected_money,
                actual,
            },
            method: "items_do_not_add_up",
            held: format!("{expected_money} {actual}"),
        },
        Case {
            error: ExpenseError::NegativeTip,
            method: "negative_tip",
            held: String::new(),
        },
        Case {
            error: ExpenseError::Overflow,
            method: "overflow",
            held: String::new(),
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedExpenseError::from_expense_error(&case.error),
            ConvertedExpenseError {
                method: case.method,
                held: case.held,
                message: case.error.to_string(),
            }
        );
    }
}
