//! Every failure of this crate converts through `FromExpenseError`, the trait
//! through which a crate outside this one, such as the binding core, reports
//! it. These tests keep each variant reaching its own method with what it
//! holds and its message.

use super::*;

/// One variant in a table of conversions: the error, what the conversion
/// finds it holding, and the method it converts through.
#[derive(Debug)]
struct Case {
    error: ExpenseError,
    held: String,
    method: &'static str,
}

/// An [`ExpenseError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedExpenseError {
    held: String,
    message: String,
    method: &'static str,
}

impl FromExpenseError for ConvertedExpenseError {
    fn amounts_do_not_add_up(actual: Money, expected: Money, message: String) -> Self {
        Self {
            held: format!("{actual} {expected}"),
            message,
            method: "amounts_do_not_add_up",
        }
    }

    fn currency_mismatch(expected: Currency, found: Currency, message: String) -> Self {
        Self {
            held: format!("{expected} {found}"),
            message,
            method: "currency_mismatch",
        }
    }

    fn duplicate_member(member: MemberId, message: String) -> Self {
        Self {
            held: format!("{member}"),
            message,
            method: "duplicate_member",
        }
    }

    fn empty_item_name(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "empty_item_name",
        }
    }

    fn empty_name(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "empty_name",
        }
    }

    fn empty_split(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "empty_split",
        }
    }

    fn items_do_not_add_up(actual: Money, expected: Money, message: String) -> Self {
        Self {
            held: format!("{actual} {expected}"),
            message,
            method: "items_do_not_add_up",
        }
    }

    fn negative_tip(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "negative_tip",
        }
    }

    fn no_items(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "no_items",
        }
    }

    fn non_positive_amount(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "non_positive_amount",
        }
    }

    fn non_positive_share(member: MemberId, message: String) -> Self {
        Self {
            held: format!("{member}"),
            message,
            method: "non_positive_share",
        }
    }

    fn overflow(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "overflow",
        }
    }

    fn percentages_do_not_add_up(message: String, total_basis_points: u64) -> Self {
        Self {
            held: format!("{total_basis_points}"),
            message,
            method: "percentages_do_not_add_up",
        }
    }

    fn zero_quantity(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "zero_quantity",
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds or its message, would show the apps another failure than the
/// library's.
#[test]
fn every_failure_of_expense_error_converts_through_the_method_of_its_variant() {
    let actual = Money::from_minor_units(Currency::EUR, 100);
    let expected_money = Money::from_minor_units(Currency::EUR, 200);
    let expected_currency = Currency::USD;
    let found = Currency::EUR;
    let member = MemberId::from_bytes([5; 16]);
    let total_basis_points = 9005;
    let cases = [
        Case {
            error: ExpenseError::AmountsDoNotAddUp {
                actual,
                expected: expected_money,
            },
            held: format!("{actual} {expected_money}"),
            method: "amounts_do_not_add_up",
        },
        Case {
            error: ExpenseError::CurrencyMismatch {
                expected: expected_currency,
                found,
            },
            held: format!("{expected_currency} {found}"),
            method: "currency_mismatch",
        },
        Case {
            error: ExpenseError::DuplicateMember { member },
            held: format!("{member}"),
            method: "duplicate_member",
        },
        Case {
            error: ExpenseError::EmptyItemName,
            held: String::new(),
            method: "empty_item_name",
        },
        Case {
            error: ExpenseError::EmptyName,
            held: String::new(),
            method: "empty_name",
        },
        Case {
            error: ExpenseError::EmptySplit,
            held: String::new(),
            method: "empty_split",
        },
        Case {
            error: ExpenseError::ItemsDoNotAddUp {
                actual,
                expected: expected_money,
            },
            held: format!("{actual} {expected_money}"),
            method: "items_do_not_add_up",
        },
        Case {
            error: ExpenseError::NegativeTip,
            held: String::new(),
            method: "negative_tip",
        },
        Case {
            error: ExpenseError::NoItems,
            held: String::new(),
            method: "no_items",
        },
        Case {
            error: ExpenseError::NonPositiveAmount,
            held: String::new(),
            method: "non_positive_amount",
        },
        Case {
            error: ExpenseError::NonPositiveShare { member },
            held: format!("{member}"),
            method: "non_positive_share",
        },
        Case {
            error: ExpenseError::Overflow,
            held: String::new(),
            method: "overflow",
        },
        Case {
            error: ExpenseError::PercentagesDoNotAddUp { total_basis_points },
            held: format!("{total_basis_points}"),
            method: "percentages_do_not_add_up",
        },
        Case {
            error: ExpenseError::ZeroQuantity,
            held: String::new(),
            method: "zero_quantity",
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedExpenseError::from_expense_error(&case.error),
            ConvertedExpenseError {
                held: case.held,
                message: case.error.to_string(),
                method: case.method,
            }
        );
    }
}
