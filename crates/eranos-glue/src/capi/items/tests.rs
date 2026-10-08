//! The core as the C bridge reads it: its marked records and enums, with
//! their documentation on one line, and its one trait of calls. An item
//! read without its mark, a field without its name, or a second trait of
//! calls would write a bridge of other types than the core's.

use quote::ToTokens;

use super::*;
use crate::capi::tests::{CALLS, RECORDS, sources};

/// The fixture's core holds its marked items in their order, not the
/// unmarked one, and its one trait of calls.
#[test]
fn the_core_is_its_marked_items_and_its_one_trait_of_calls() {
    let core = Core::parse(&sources()).unwrap();
    assert_eq!(
        core.items
            .iter()
            .map(|item| item.name.to_string())
            .collect::<Vec<_>>(),
        [
            "Money",
            "CalendarDate",
            "TimeOfDay",
            "TipDistribution",
            "ExpenseItem",
            "TipRule",
            "Split",
            "Cost",
            "Balance",
            "Expense",
            "CurrencyDetails",
            "NewExpenseGroup",
            "Summary",
        ]
    );
    assert_eq!(core.calls.calls.len(), 15);
    assert!(core.item("Expense").is_some());
    assert!(core.item("Handle").is_none());
}

/// A record reads as its fields, and an enum as its variants, each with its
/// documentation on one line and its links as code spans; an enum whose
/// variants hold nothing is a plain one.
#[test]
fn an_item_reads_as_its_fields_or_its_variants() {
    let core = Core::parse(&[
        "
        /// A share, in [`Money`](crate::Money),
        /// of a member.
        #[eranos_glue_macros::mirrored]
        pub struct Share {
            /// The amount.
            pub amount: Money,
            /// The member.
            pub member: String,
        }

        /// A choice.
        #[mirrored]
        pub enum Choice {
            /// A value.
            Some {
                /// The value.
                value: i32,
            },
            /// None.
            None,
        }

        /// A plain choice.
        #[mirrored]
        pub enum Plain {
            /// One.
            One,
        }

        #[mirrored]
        pub struct Unit;
        "
        .to_owned(),
        CALLS.to_owned(),
    ])
    .unwrap();
    let share = core.item("Share").unwrap();
    assert_eq!(share.doc, "A share, in `Money`, of a member.");
    let Shape::Record(fields) = &share.shape else {
        panic!("a record");
    };
    assert_eq!(
        fields
            .iter()
            .map(|field| (
                field.doc.as_str(),
                field.name.to_string(),
                field.written.to_token_stream().to_string()
            ))
            .collect::<Vec<_>>(),
        [
            ("The amount.", "amount".to_owned(), "Money".to_owned()),
            ("The member.", "member".to_owned(), "String".to_owned()),
        ]
    );
    assert_eq!(share.fields().len(), 2);
    assert!(!share.is_plain_enum());
    let choice = core.item("Choice").unwrap();
    let Shape::Choice(variants) = &choice.shape else {
        panic!("an enum");
    };
    assert_eq!(
        variants
            .iter()
            .map(|variant| (
                variant.doc.as_str(),
                variant.name.to_string(),
                variant.fields.len()
            ))
            .collect::<Vec<_>>(),
        [
            ("A value.", "Some".to_owned(), 1),
            ("None.", "None".to_owned(), 0)
        ]
    );
    assert_eq!(choice.fields().len(), 1);
    assert!(!choice.is_plain_enum());
    assert!(core.item("Plain").unwrap().is_plain_enum());
    assert!(core.item("Unit").unwrap().fields().is_empty());
}

/// A core without its trait of calls, or with two, sources that are not
/// Rust, a mirrored field without a name, and documentation that is not
/// text are refused.
#[test]
fn a_core_the_rules_refuse_is_refused() {
    let refusal = |sources: &[&str]| {
        Core::parse(
            &sources
                .iter()
                .map(|&source| source.to_owned())
                .collect::<Vec<_>>(),
        )
        .err()
        .map(|error| error.to_string())
        .unwrap_or_default()
    };
    assert_eq!(
        refusal(&[RECORDS]),
        "the core holds one trait of group calls, marked `#[calls]`"
    );
    assert_eq!(
        refusal(&[RECORDS, CALLS, CALLS]),
        "the core holds one trait of group calls, marked `#[calls]`"
    );
    assert_ne!(refusal(&["fn ("]), "");
    for refused in [
        "#[mirrored] pub struct Pair(i32, i32);",
        "#[mirrored] pub enum Pair { Both(i32, i32) }",
    ] {
        assert_eq!(
            refusal(&[refused, CALLS]),
            "a mirrored item names its fields"
        );
    }
    for refused in [
        "#[doc = 1] #[mirrored] pub struct Pair { pub left: i32 }",
        "#[mirrored] pub struct Pair { #[doc = 1] pub left: i32 }",
        "#[mirrored] pub enum Pair { #[doc = 1] Left }",
    ] {
        assert_eq!(refusal(&[refused, CALLS]), "documentation is text");
    }
    assert_ne!(
        refusal(&[
            RECORDS,
            "#[calls] trait ExpenseGroupCalls { const LIMIT: usize; }"
        ]),
        ""
    );
}
