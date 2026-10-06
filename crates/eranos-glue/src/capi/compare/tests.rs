//! The comparison of a committed file with the generator's code. A
//! comparison that saw rustfmt's commas, a `match` arm's braces, or a doc
//! comment's wrapping would fail every commit that ran rustfmt, and one that
//! missed a changed token or a changed sentence would let a hand edit pass.

use proc_macro2::TokenStream;
use quote::quote;

use super::*;

/// Whether `committed`, as text, holds the code `generated`.
fn holds(committed: &str, generated: TokenStream) -> bool {
    same(committed, &syn::parse2(generated).unwrap()).unwrap()
}

/// Trailing commas, a `match` arm's expression in braces, and documentation
/// wrapped or written as attributes compare as the same code.
#[test]
fn layout_alone_never_differs() {
    assert!(holds(
        "fn call() { f(a, b,); g::<A, B,>(); }",
        quote! { fn call() { f(a, b); g::<A, B>(); } }
    ));
    assert!(holds(
        "fn call() { match x { A => { f(a) } B => g(), } }",
        quote! { fn call() { match x { A => f(a), B => g() } } }
    ));
    assert!(holds(
        "fn call() { match x { A => { f(a) }, B => { g() } } }",
        quote! { fn call() { match x { A => f(a), B => g(), } } }
    ));
    assert!(holds(
        "//! A file.\n\n/// One  two\n/// three.\n///\n/// Four.\nfn call() {}",
        quote! {
            #![doc = "A file."]
            #[doc = "One two three.\n\nFour."]
            fn call() {}
        }
    ));
}

/// A changed token, a removed comma between two arguments, a sentence of
/// documentation, an inner comment made outer, and a block of statements
/// unwrapped all differ.
#[test]
fn code_or_documentation_that_differs_differs() {
    assert!(!holds(
        "fn call() { f(a, c) }",
        quote! { fn call() { f(a, b) } }
    ));
    assert!(!holds(
        "fn call() { f(a, &b) }",
        quote! { fn call() { f(a &b) } }
    ));
    assert!(!holds(
        "/// One.\nfn call() {}",
        quote! { #[doc = "Two."] fn call() {} }
    ));
    assert!(!holds(
        "//! One.\nfn call() {}",
        quote! { #[doc = "One."] fn call() {} }
    ));
    assert!(!holds(
        "fn call() { match x { A => { f(); } } }",
        quote! { fn call() { match x { A => f(), } } }
    ));
}

/// Committed text that is not Rust is refused.
#[test]
fn text_that_is_not_rust_is_refused() {
    let generated = syn::parse2(quote! { fn call() {} }).unwrap();
    assert!(same("fn (", &generated).is_err());
}

/// Only `doc = "…"` in brackets after `#` reads as documentation; any other
/// attribute, or `#` before another group, reads as the tokens it is.
#[test]
fn only_a_doc_of_text_reads_as_documentation() {
    assert_eq!(
        normalised(
            quote! { # (doc = "No.") #[doc = 1] #[doc(hidden)] #[derive(Debug)] #[doc = "Yes."] }
        ),
        [
            Piece::Token("#".to_owned()),
            Piece::Open(Delimiter::Parenthesis),
            Piece::Token("doc".to_owned()),
            Piece::Token("=".to_owned()),
            Piece::Token("\"No.\"".to_owned()),
            Piece::Close,
            Piece::Token("#".to_owned()),
            Piece::Open(Delimiter::Bracket),
            Piece::Token("doc".to_owned()),
            Piece::Token("=".to_owned()),
            Piece::Token("1".to_owned()),
            Piece::Close,
            Piece::Token("#".to_owned()),
            Piece::Open(Delimiter::Bracket),
            Piece::Token("doc".to_owned()),
            Piece::Open(Delimiter::Parenthesis),
            Piece::Token("hidden".to_owned()),
            Piece::Close,
            Piece::Close,
            Piece::Token("#".to_owned()),
            Piece::Open(Delimiter::Bracket),
            Piece::Token("derive".to_owned()),
            Piece::Open(Delimiter::Parenthesis),
            Piece::Token("Debug".to_owned()),
            Piece::Close,
            Piece::Close,
            Piece::Docs {
                inner: false,
                texts: vec!["Yes.".to_owned()],
            },
        ]
    );
    assert_eq!(
        normalised(quote!(#[doc = "One."] #![doc = "Two."])),
        [
            Piece::Docs {
                inner: false,
                texts: vec!["One.".to_owned()],
            },
            Piece::Docs {
                inner: true,
                texts: vec!["Two.".to_owned()],
            },
        ]
    );
}
