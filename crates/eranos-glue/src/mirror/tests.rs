//! A mirrored item keeps its definition and hands a glue crate its fields
//! alone. A mirror that kept an attribute would carry a derive or a serde
//! rule into a glue crate's copy, one that lost a field would make the copy
//! disagree with the item, and an item no copy can state would fail far
//! from its cause.

use proc_macro2::TokenStream;
use quote::quote;

use super::*;

/// An item `#[mirrored]` is given, with the attribute's arguments.
struct Marked {
    /// The attribute's arguments.
    attribute: TokenStream,
    /// The item.
    item: TokenStream,
}

/// The code `#[mirrored]` writes for `item` with `attribute`.
fn mirror(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    mirrored(attribute, item)
}

/// A record, an enum, and an error keep their definitions whole, and each
/// mirror macro, named after its item, hands the item's kind, name,
/// visibility, and fields without a single attribute or doc comment.
#[test]
fn an_item_keeps_its_definition_and_its_mirror_its_fields() {
    let record = quote! {
        /// An amount.
        #[derive(Debug, Clone)]
        pub struct Money {
            /// The currency.
            pub currency: String,
            /// The minor units.
            #[serde(default)]
            pub minor_units: i64,
        }
    };
    let record_doc = "Hands the mirror of [`Money`] to the macro whose path it is given: its kind, name, and \
                      fields, without its attributes and documentation, from which a glue crate stamps its own \
                      copy.";
    assert_eq!(
        mirror(quote!(), record.clone()).unwrap().to_string(),
        quote! {
            #record

            #[doc = #record_doc]
            #[macro_export]
            macro_rules! mirror_money {
                ($($callback:tt)+) => {
                    $($callback)+ ! { Record pub struct Money { pub currency: String, pub minor_units: i64, } }
                };
            }
        }
        .to_string()
    );
    let choice = quote! {
        /// A balance.
        #[non_exhaustive]
        pub enum Balance {
            /// Is owed money.
            GetsBack { amount: Money },
            /// Is settled.
            #[default]
            Settled,
        }
    };
    let written = mirror(quote!(), choice.clone()).unwrap().to_string();
    assert!(written.starts_with(&choice.to_string()));
    assert!(written.contains(
        &quote!(Enum pub enum Balance { GetsBack { amount: Money }, Settled, }).to_string()
    ));
    assert!(written.contains(&quote!(macro_rules! mirror_balance).to_string()));
    let error = quote! {
        #[derive(Debug)]
        pub enum EranosError {
            Cancelled { detail: String },
        }
    };
    assert!(mirror(quote!(error), error).unwrap().to_string().contains(
        &quote!(Error pub enum EranosError { Cancelled { detail: String }, }).to_string()
    ));
}

/// An item a glue crate's copy could not state, or an attribute the rules
/// do not know, would fail in every glue crate instead of at the item.
#[test]
fn only_records_enums_and_errors_without_generics_are_mirrored() {
    let refusal = |attribute: TokenStream, item: TokenStream| {
        mirror(attribute, item).unwrap_err().to_string()
    };
    assert_eq!(
        refusal(
            quote!(record),
            quote!(
                pub struct Money {
                    pub minor_units: i64,
                }
            )
        ),
        "a mirrored item is a record or an enum, or an error with `#[mirrored(error)]`"
    );
    for Marked { attribute, item } in [
        Marked {
            attribute: quote!(),
            item: quote!(
                pub struct Money(pub i64);
            ),
        },
        Marked {
            attribute: quote!(),
            item: quote!(
                pub struct Unit;
            ),
        },
        Marked {
            attribute: quote!(error),
            item: quote!(
                pub struct Failure {
                    pub detail: String,
                }
            ),
        },
    ] {
        assert_eq!(
            refusal(attribute, item.clone()),
            "a mirrored struct is a record with named fields, and an error is an enum",
            "{item}"
        );
    }
    for item in [
        quote!(
            pub struct Pair<Value> {
                pub first: Value,
            }
        ),
        quote!(
            pub enum Choice<Value> {
                One { value: Value },
            }
        ),
    ] {
        assert_eq!(
            refusal(quote!(), item.clone()),
            "a mirrored item has no generics",
            "{item}"
        );
    }
    for item in [
        quote!(
            pub enum Cost {
                Local(Money),
            }
        ),
        quote!(
            pub enum Strategy {
                Direct = 1,
            }
        ),
    ] {
        assert_eq!(
            refusal(quote!(), item.clone()),
            "a mirrored enum's variant is a unit or names its fields",
            "{item}"
        );
    }
    assert_eq!(
        refusal(
            quote!(),
            quote!(
                pub fn money() {}
            )
        ),
        "a mirrored item is a struct or an enum"
    );
    assert!(
        !refusal(
            quote!(error, twice),
            quote!(
                pub enum Failure {}
            )
        )
        .is_empty()
    );
    assert!(!refusal(quote!(), quote!(not an item)).is_empty());
}
