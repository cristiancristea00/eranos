//! Every failure of this crate converts through `FromExpenseError`, the trait
//! through which a crate outside this one, such as the binding core, reports
//! it. These tests keep each variant reaching its own method with what it
//! holds and its message.

use super::*;

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
        (
            ExpenseError::AmountsDoNotAddUp {
                actual,
                expected: expected_money,
            },
            "amounts_do_not_add_up",
            format!("{actual} {expected_money}"),
        ),
        (
            ExpenseError::CurrencyMismatch {
                expected: expected_currency,
                found,
            },
            "currency_mismatch",
            format!("{expected_currency} {found}"),
        ),
        (
            ExpenseError::DuplicateMember { member },
            "duplicate_member",
            format!("{member}"),
        ),
        (
            ExpenseError::EmptyItemName,
            "empty_item_name",
            String::new(),
        ),
        (ExpenseError::EmptyName, "empty_name", String::new()),
        (ExpenseError::EmptySplit, "empty_split", String::new()),
        (
            ExpenseError::ItemsDoNotAddUp {
                actual,
                expected: expected_money,
            },
            "items_do_not_add_up",
            format!("{actual} {expected_money}"),
        ),
        (ExpenseError::NegativeTip, "negative_tip", String::new()),
        (ExpenseError::NoItems, "no_items", String::new()),
        (
            ExpenseError::NonPositiveAmount,
            "non_positive_amount",
            String::new(),
        ),
        (
            ExpenseError::NonPositiveShare { member },
            "non_positive_share",
            format!("{member}"),
        ),
        (ExpenseError::Overflow, "overflow", String::new()),
        (
            ExpenseError::PercentagesDoNotAddUp { total_basis_points },
            "percentages_do_not_add_up",
            format!("{total_basis_points}"),
        ),
        (ExpenseError::ZeroQuantity, "zero_quantity", String::new()),
    ];
    for (error, method, held) in cases {
        assert_eq!(
            ConvertedExpenseError::from_expense_error(&error),
            ConvertedExpenseError {
                held,
                message: error.to_string(),
                method,
            }
        );
    }
}
