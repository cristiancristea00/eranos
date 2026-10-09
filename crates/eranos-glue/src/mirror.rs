//! The mirrors of the binding core's records, enums, and errors. An item
//! marked `#[mirrored]` keeps its definition and gains an exported macro
//! that imports the item where it is invoked and hands its kind, name, and
//! fields, without its attributes and documentation, to a glue crate's
//! macro, which writes its own copy of the item from them: a crate holds
//! one `uniffi` release, so the core's items cannot carry `uniffi`'s
//! derives themselves.

use proc_macro2::{Ident, Punct, Spacing, TokenStream};
use quote::{format_ident, quote};
use syn::{Fields, Item};

/// The item `item` with its mirror, the kind of which `attribute` names:
/// nothing for a record or an enum, `error` for an error.
///
/// # Errors
///
/// For an attribute other than `error`, an item that is neither a struct
/// with named fields nor an enum of unit variants and variants with named
/// fields, an item with generics, and an error that is not an enum.
pub(crate) fn mirrored(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let error = if attribute.is_empty() {
        false
    } else {
        let word: Ident = syn::parse2(attribute)?;
        match word.to_string().as_str() {
            "error" => true,
            _ => {
                return Err(syn::Error::new_spanned(
                    word,
                    "a mirrored item is a record or an enum, or an error with `#[mirrored(error)]`",
                ));
            }
        }
    };
    let item: Item = syn::parse2(item)?;
    let mut stripped = item.clone();
    let (name, kind) = if let Item::Struct(record) = &mut stripped {
        if error || !matches!(record.fields, Fields::Named(_)) {
            return Err(syn::Error::new_spanned(
                &record.ident,
                "a mirrored struct is a record with named fields, and an error is an enum",
            ));
        }
        if !record.generics.params.is_empty() {
            return Err(generic(&record.ident));
        }
        record.attrs.clear();
        for field in &mut record.fields {
            field.attrs.clear();
        }
        (record.ident.clone(), quote!(Record))
    } else if let Item::Enum(choice) = &mut stripped {
        if !choice.generics.params.is_empty() {
            return Err(generic(&choice.ident));
        }
        choice.attrs.clear();
        for variant in &mut choice.variants {
            if matches!(variant.fields, Fields::Unnamed(_)) || variant.discriminant.is_some() {
                return Err(syn::Error::new_spanned(
                    &variant.ident,
                    "a mirrored enum's variant is a unit or names its fields",
                ));
            }
            variant.attrs.clear();
            for field in &mut variant.fields {
                field.attrs.clear();
            }
        }
        let kind = if error { quote!(Error) } else { quote!(Enum) };
        (choice.ident.clone(), kind)
    } else {
        return Err(syn::Error::new_spanned(
            &stripped,
            "a mirrored item is a struct or an enum",
        ));
    };
    let macro_name = format_ident!("mirror_{}", crate::snake_case(&name.to_string()));
    let doc = format!(
        "Imports [`{name}`] where it is invoked, and hands its mirror to the macro whose path it is \
         given: its kind, name, and fields, without its attributes and documentation, from which a \
         glue crate writes its own copy, which names the item."
    );
    let dollar = Punct::new('$', Spacing::Alone);
    Ok(quote! {
        #item

        #[doc = #doc]
        #[macro_export]
        macro_rules! #macro_name {
            (#dollar (#dollar callback:tt)+) => {
                use #dollar crate::#name;
                #dollar (#dollar callback)+ ! { #kind #stripped }
            };
        }
    })
}

/// The failure of a mirrored item `name` that has generics, which no glue
/// crate's copy could name.
fn generic(name: &Ident) -> syn::Error {
    syn::Error::new_spanned(name, "a mirrored item has no generics")
}

#[cfg(test)]
mod tests;
