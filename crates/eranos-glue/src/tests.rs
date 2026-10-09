//! The entry points the proc macros call. An argument `#[calls]` does not
//! take, or input that is not the item a macro expects, would be dropped or
//! misread instead of refused where it was written; a name cased otherwise
//! than a table's callers write it would name a macro no glue crate finds.

use quote::quote;

use super::*;

/// `#[calls]` takes no arguments and a trait; `uniffi_objects!` takes the
/// trait its table hands over; `#[mirrored]` and `uniffi_mirror!` take a
/// mirror's item and its kind.
#[test]
fn every_entry_point_refuses_what_it_does_not_take() {
    let item = quote! {
        trait ExpenseGroupCalls {
            /// The group's name.
            fn name(group: &ExpenseGroup) -> String {}
        }
    };
    assert!(calls(quote!(), item.clone()).is_ok());
    assert_eq!(
        calls(quote!(verbose), item.clone())
            .unwrap_err()
            .to_string(),
        "`#[calls]` takes no arguments"
    );
    assert!(
        calls(
            quote!(),
            quote!(
                struct ExpenseGroupCalls;
            )
        )
        .is_err()
    );
    assert!(uniffi_objects(item).is_ok());
    let refused = quote! {
        trait ExpenseGroupCalls {
            const LIMIT: usize;
        }
    };
    assert!(calls(quote!(), refused.clone()).is_err());
    assert!(uniffi_objects(refused).is_err());
    assert!(
        uniffi_objects(quote!(
            struct ExpenseGroupCalls;
        ))
        .is_err()
    );
    assert!(
        mirrored(
            quote!(),
            quote!(
                pub struct Money {
                    pub minor_units: i64,
                }
            )
        )
        .is_ok()
    );
    assert!(uniffi_mirror(quote!(Record pub struct Money { pub minor_units: i64 })).is_ok());
}

/// A table or a mirror macro would get a name no glue crate calls it by.
#[test]
fn names_turn_from_upper_camel_case_into_snake_case() {
    assert_eq!(snake_case("ExpenseGroupCalls"), "expense_group_calls");
    assert_eq!(snake_case("MemberCategoryTotals"), "member_category_totals");
    assert_eq!(snake_case("Money"), "money");
    assert_eq!(snake_case(""), "");
}
