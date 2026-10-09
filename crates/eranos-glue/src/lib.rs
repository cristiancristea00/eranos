//! The model of the binding core's group calls and records, and the code
//! the glue writes from it, so every call and every record is written once,
//! in the core.
//!
//! The core lists its group calls, each with its body, in one trait, which
//! [`calls`] turns into the calls' functions, the group handle's
//! synchronous and asynchronous forms, the read and write handles' forms,
//! and a table macro that hands the trait to the glue crates; each record,
//! enum, and error the bindings expose is marked with [`mirrored`], which
//! adds a macro that imports it and hands over its fields. The `uniffi`
//! glue crates write their objects' methods from the table with
//! [`uniffi_objects`], and their `#[uniffi::remote]` copies from the mirrors
//! with [`uniffi_mirror`]. The
//! proc macros of eranos-glue-macros call these functions inside the
//! compiler, where no coverage run reaches; this crate's tests measure them.
//!
//! The C glue's bridge cannot come from a macro, since `diplomat-tool` reads
//! its source files: [`capi_bridge`] writes it from the core's source files
//! instead, as the files that `cargo xtask generate` formats and the
//! repository holds.
#![forbid(unsafe_code)]

mod calls;
mod capi;
mod docs;
mod handle;
mod mirror;
mod uniffi;

use proc_macro2::TokenStream;
use syn::ItemTrait;

use crate::calls::Calls;
pub use crate::capi::Generated;

/// The code of the trait `item`, which lists the group calls.
///
/// With `attribute` empty, the code is every call's function, private to
/// the module, the group handle's synchronous and asynchronous form of every
/// call, the read handle's form of every read, the write handle's form of
/// every call, and the exported macro that hands the trait to a glue
/// crate's macro; the trait itself is not kept.
///
/// # Errors
///
/// For an attribute with arguments, an item that is not a plain trait of
/// functions, and a function that is not a documented call with a body
/// that takes the group first, or whose `Result` is not of an answer and
/// `EranosError`.
pub fn calls(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    if !attribute.is_empty() {
        return Err(syn::Error::new_spanned(
            attribute,
            "`#[calls]` takes no arguments",
        ));
    }
    let item: ItemTrait = syn::parse2(item)?;
    Ok(handle::expand(&Calls::parse(&item)?, &item))
}

/// The item `item`, a record, an enum, or an error, with the exported
/// macro that hands its mirror to a glue crate's macro; `attribute` is
/// empty for a record or an enum and `error` for an error.
///
/// # Errors
///
/// For an attribute other than `error`, and an item that is not a struct
/// with named fields or an enum whose variants are units or name their
/// fields, or that has generics.
pub fn mirrored(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    mirror::mirrored(attribute, item)
}

/// The exported methods of the `uniffi` glue's `ExpenseGroup`,
/// `ExpenseGroupReader`, and `ExpenseGroupWriter` objects, from `input`, the
/// trait of group calls that the core's table hands over.
///
/// # Errors
///
/// As for [`calls`].
pub fn uniffi_objects(input: TokenStream) -> syn::Result<TokenStream> {
    Ok(uniffi::objects(&Calls::parse(&syn::parse2::<ItemTrait>(
        input,
    )?)?))
}

/// The `#[uniffi::remote]` copy of the mirror `input`, as an item's mirror
/// macro hands it over.
///
/// # Errors
///
/// For input that is not `Record`, `Enum`, or `Error` followed by an item.
pub fn uniffi_mirror(input: TokenStream) -> syn::Result<TokenStream> {
    uniffi::mirror(input)
}

/// The files of the C glue's bridge, from `sources`, the text of each of the
/// binding core's source files.
///
/// The bridge holds every record, list, and enum the core's calls reach,
/// and the group, its read handle, and the outcomes of its calls.
///
/// # Errors
///
/// For a source that is not Rust, a core without its trait of calls or with
/// two, and a record, an enum, or a call whose types the bridge cannot
/// carry.
pub fn capi_bridge(sources: &[String]) -> syn::Result<Vec<Generated>> {
    capi::bridge(sources)
}

/// `name`, written in `UpperCamelCase`, in `snake_case`: `ExpenseGroupCalls` is
/// `expense_group_calls`.
fn snake_case(name: &str) -> String {
    let mut snake = String::with_capacity(name.len() * 2);
    for (index, character) in name.char_indices() {
        if character.is_uppercase() && 0 < index {
            snake.push('_');
        }
        snake.extend(character.to_lowercase());
    }
    snake
}

#[cfg(test)]
mod tests;
