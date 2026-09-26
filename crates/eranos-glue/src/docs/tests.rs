//! Each form of a call carries the call's documentation with the failures of
//! its own: the lock on wasm, the cancellation, the released handle. A form
//! documented with another's failures, a summary cut short, or a Rust link
//! left in a foreign binding's documentation would mislead every caller of
//! that form.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ItemTrait;
use syn::TraitItem;

use super::*;

/// The documentation of the one call of `item`, a trait.
fn docs(item: TokenStream) -> syn::Result<Docs> {
    let item: ItemTrait = syn::parse2(item)?;
    let Some(TraitItem::Fn(function)) = item.items.first() else {
        panic!("one call");
    };
    Docs::parse(&function.attrs, &function.sig.ident)
}

/// A documentation of two paragraphs, the first over two lines.
fn two_paragraphs() -> Docs {
    docs(quote! {
        trait GroupCalls {
            /// Renames the category that `category` names by its
            /// identifier, and returns it as it was.
            ///
            /// Its expenses keep it.
            #[write]
            fn rename(&self) -> Result<(), EranosError>;
        }
    })
    .unwrap()
}

/// A doc comment's lines would reach the forms broken where the source
/// wraps them, or its paragraphs run together.
#[test]
fn a_doc_comment_reads_as_its_paragraphs() {
    assert_eq!(
        two_paragraphs(),
        Docs {
            summary: "Renames the category that `category` names by its identifier, and returns it as it was."
                .to_owned(),
            details: vec!["Its expenses keep it.".to_owned()],
        }
    );
    let attributes = docs(quote! {
        trait GroupCalls {
            #[doc = "The first.\nstill the first."]
            #[doc = ""]
            #[doc = "  The second.  "]
            #[doc = "The third.\n\nThe fourth."]
            #[read]
            fn call(&self) -> Result<(), EranosError>;
        }
    })
    .unwrap();
    assert_eq!(attributes.summary, "The first. still the first.");
    assert_eq!(
        attributes.details,
        ["The second. The third.", "The fourth."]
    );
}

/// A form would promise the failures of another: a read's lock taken by a
/// change, a change's by a read, or the operation's refusal where the
/// operation cannot fail.
#[test]
fn each_form_names_its_own_failures() {
    let docs = two_paragraphs();
    let paragraphs = "Renames the category that `category` names by its identifier, and returns it as it was.\n\nIts expenses keep it.";
    assert_eq!(
        docs.synchronous(Access::Write, false),
        format!(
            "{paragraphs}\n\n# Errors\n\nThe library's refusal, and [`GroupFailure::Busy`](crate::GroupFailure::Busy) on wasm while another call holds the lock."
        )
    );
    assert_eq!(
        docs.synchronous(Access::Read, true),
        format!(
            "{paragraphs}\n\n# Errors\n\n[`GroupFailure::Busy`](crate::GroupFailure::Busy) on wasm while a change holds the lock or waits for it."
        )
    );
    assert_eq!(
        docs.reader(false),
        format!(
            "{paragraphs}\n\n# Errors\n\nThe library's refusal, and [`EranosError::Released`] after [`release`](Self::release)."
        )
    );
    assert_eq!(
        docs.reader(true),
        format!(
            "{paragraphs}\n\n# Errors\n\n[`EranosError::Released`] after [`release`](Self::release)."
        )
    );
    assert_eq!(
        docs.asynchronous(false),
        "Renames the category that `category` names by its identifier, and returns it as it was, as a future that waits for the lock.\n\nIts expenses keep it.\n\n# Errors\n\nThe library's refusal, and [`EranosError::Cancelled`]."
    );
    assert!(
        docs.asynchronous(true)
            .ends_with("# Errors\n\n[`EranosError::Cancelled`].")
    );
}

/// A call without documentation, or with a summary the asynchronous form
/// could not extend, would reach every binding undocumented or with a
/// summary cut mid-word; documentation that is not text cannot be read.
#[test]
fn a_call_is_documented_and_its_summary_ends_with_a_full_stop() {
    let refusal = |attributes: TokenStream| {
        docs(quote! {
            trait GroupCalls {
                #attributes
                #[read]
                fn call(&self) -> Result<(), EranosError>;
            }
        })
        .unwrap_err()
        .to_string()
    };
    for attributes in [
        quote!(),
        quote!(#[doc = ""]),
        quote!(#[doc = "The group's name"]),
    ] {
        assert_eq!(
            refusal(attributes.clone()),
            "a group call is documented, its summary ending with a full stop",
            "{attributes}"
        );
    }
    for attributes in [quote!(#[doc = 1]), quote!(#[doc = concat!("The name.")])] {
        assert_eq!(
            refusal(attributes.clone()),
            "a group call's documentation is text",
            "{attributes}"
        );
    }
    assert!(!refusal(quote!(#[doc(hidden)])).is_empty());
    assert!(Docs::parse(&[], &format_ident!("call")).is_err());
}

/// A Rust link in a foreign binding's documentation would read there as
/// raw markdown; text without links, or with a bracket that opens no link,
/// would be changed.
#[test]
fn a_foreign_binding_reads_every_link_as_its_code_span() {
    assert_eq!(
        foreign(
            "[`GroupFailure::Busy`](crate::GroupFailure::Busy) on wasm, or [`EranosError::Cancelled`]."
        ),
        "`GroupFailure::Busy` on wasm, or `EranosError::Cancelled`."
    );
    assert_eq!(
        foreign("[`EranosError::Released`] after [`release`](Self::release)."),
        "`EranosError::Released` after `release`."
    );
    assert_eq!(
        foreign("Plain `code` [text](link)."),
        "Plain `code` [text](link)."
    );
    assert_eq!(
        foreign("Open [`bracket, never closed"),
        "Open [`bracket, never closed"
    );
    assert_eq!(foreign("A [`link`](unclosed"), "A `link`(unclosed");
}
