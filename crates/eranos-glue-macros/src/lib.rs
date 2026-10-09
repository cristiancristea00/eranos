//! The proc macros of eranos's glue: entry points over eranos-glue, which
//! holds the model of the binding core's group calls and records and the
//! code written from it, and whose tests measure that code. A macro's
//! failure becomes a compile error at the span it names.
#![forbid(unsafe_code)]

use proc_macro::TokenStream;

/// Writes the functions and forms of the group calls that the binding
/// core's trait lists.
///
/// The code is every call's function, the group handle's synchronous and
/// asynchronous form of every call, the read handle's form of every read,
/// the write handle's form of every call, and the exported table macro that
/// hands the trait to the glue crates, as `eranos_glue::calls` describes.
#[proc_macro_attribute]
pub fn calls(attribute: TokenStream, item: TokenStream) -> TokenStream {
    eranos_glue::calls(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Adds the mirror of a record, an enum, or an error of the binding core.
///
/// The item stays as it is, and the exported macro beside it imports the
/// item where it is invoked and hands its mirror to a glue crate, as
/// `eranos_glue::mirrored` describes; an error takes `#[mirrored(error)]`.
#[proc_macro_attribute]
pub fn mirrored(attribute: TokenStream, item: TokenStream) -> TokenStream {
    eranos_glue::mirrored(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Writes a `uniffi` glue crate's object methods from the core's table.
///
/// The macro is the table's callback, and writes the exported methods of
/// the crate's `ExpenseGroup`, `ExpenseGroupReader`, and
/// `ExpenseGroupWriter`, as `eranos_glue::uniffi_objects` describes.
#[proc_macro]
pub fn uniffi_objects(input: TokenStream) -> TokenStream {
    eranos_glue::uniffi_objects(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Writes a `uniffi` glue crate's copy of a mirrored item.
///
/// The macro is a mirror's callback, and writes the item's
/// `#[uniffi::remote]` copy, as `eranos_glue::uniffi_mirror` describes.
#[proc_macro]
pub fn uniffi_mirror(input: TokenStream) -> TokenStream {
    eranos_glue::uniffi_mirror(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
