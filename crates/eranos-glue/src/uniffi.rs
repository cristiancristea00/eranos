//! The code the `uniffi` glue writes from the binding core's tables: the
//! methods of its `ExpenseGroup`, `ExpenseGroupReader`, and `ExpenseGroupWriter`
//! objects, which forward to the core's handles, and the `#[uniffi::remote]`
//! copies of the core's records, enums, and error.
//!
//! The code names `ExpenseGroup`, `ExpenseGroupReader`, `ExpenseGroupWriter`,
//! `Cancellation`, `shared`, and every type a call takes or answers as the
//! module that invokes the table imports them, and `::uniffi` as the
//! invoking crate's own release, so the three glue crates, each on its
//! release, share one source.

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::Item;
use syn::parse::{Parse, ParseStream};

use crate::calls::{Access, Call, Calls, Parameter};
use crate::docs::foreign;

/// The exported methods of the objects: each call's synchronous and
/// asynchronous forms on `ExpenseGroup`, each read's form on `ExpenseGroupReader`,
/// and each call's form on `ExpenseGroupWriter`, with the core's documentation
/// in the form a foreign binding shows.
///
/// The synchronous form of a call that cannot fail is exported twice, in a
/// block of its own for each target, since `uniffi` reads an attribute on a
/// method before `cfg` strips it: outside wasm it answers the value itself,
/// as the core's form does, and on wasm a `Result`.
pub(crate) fn objects(calls: &Calls) -> TokenStream {
    let group = calls.calls.iter().map(|call| {
        let (method, answer, error) = (&call.name, &call.answer, &call.error);
        let asynchronous = format_ident!("{}_async", call.name);
        let doc = foreign(&call.docs.synchronous(call.access, call.infallible));
        let asynchronous_doc = foreign(&call.docs.asynchronous(call.infallible));
        let parameters = foreign_parameters(call);
        let lent = call.parameters.iter().map(Parameter::lent);
        let owned = call.parameters.iter().map(|parameter| {
            let name = &parameter.name;
            let owned = parameter.owned();
            quote!(#name: #owned)
        });
        let names = call.parameters.iter().map(|parameter| &parameter.name);
        let synchronous = (!call.infallible).then(|| {
            quote! {
                #[doc = #doc]
                pub(crate) fn #method(&self #(, #parameters)*) -> ::std::result::Result<#answer, #error> {
                    self.handle.#method(#(#lent),*)
                }
            }
        });
        quote! {
            #synchronous

            #[doc = #asynchronous_doc]
            pub(crate) async fn #asynchronous(
                &self,
                #(#owned,)*
                cancellation: ::std::option::Option<::std::sync::Arc<Cancellation>>,
            ) -> ::std::result::Result<#answer, #error> {
                self.handle.#asynchronous(#(#names,)* shared(cancellation)).await
            }
        }
    });
    let unfailing_blocks = unfailing_blocks(calls);
    let reads = calls
        .calls
        .iter()
        .filter(|call| Access::Read == call.access)
        .map(|call| {
            let (method, answer, error) = (&call.name, &call.answer, &call.error);
            let doc = foreign(&call.docs.reader(call.infallible));
            let parameters = foreign_parameters(call);
            let lent = call.parameters.iter().map(Parameter::lent);
            quote! {
                #[doc = #doc]
                pub(crate) fn #method(&self #(, #parameters)*) -> ::std::result::Result<#answer, #error> {
                    self.reader.#method(#(#lent),*)
                }
            }
        });
    let writes = calls.calls.iter().map(|call| {
        let (method, answer, error) = (&call.name, &call.answer, &call.error);
        let doc = foreign(&call.docs.writer(call.infallible));
        let parameters = foreign_parameters(call);
        let lent = call.parameters.iter().map(Parameter::lent);
        quote! {
            #[doc = #doc]
            pub(crate) fn #method(&self #(, #parameters)*) -> ::std::result::Result<#answer, #error> {
                self.writer.#method(#(#lent),*)
            }
        }
    });
    quote! {
        #[::uniffi::export]
        impl ExpenseGroup {
            #(#group)*
        }

        #unfailing_blocks

        #[::uniffi::export]
        impl ExpenseGroupReader {
            #(#reads)*
        }

        #[::uniffi::export]
        impl ExpenseGroupWriter {
            #(#writes)*
        }
    }
}

/// The two export blocks of the synchronous forms of the calls that cannot
/// fail, one for each target, or none when no call is one.
fn unfailing_blocks(calls: &Calls) -> Option<TokenStream> {
    let unfailing: Vec<&Call> = calls.calls.iter().filter(|call| call.infallible).collect();
    let outside_wasm = unfailing.iter().map(|call| {
        let method = &call.name;
        let doc = foreign(&call.docs.unfailing());
        let parameters = foreign_parameters(call);
        let lent = call.parameters.iter().map(Parameter::lent);
        let answer = (!call.answers_nothing()).then(|| {
            let answer = &call.answer;
            quote!(-> #answer)
        });
        quote! {
            #[doc = #doc]
            pub(crate) fn #method(&self #(, #parameters)*) #answer {
                self.handle.#method(#(#lent),*)
            }
        }
    });
    let on_wasm = unfailing.iter().map(|call| {
        let (method, answer, error) = (&call.name, &call.answer, &call.error);
        let doc = foreign(&call.docs.synchronous(call.access, call.infallible));
        let parameters = foreign_parameters(call);
        let lent = call.parameters.iter().map(Parameter::lent);
        quote! {
            #[doc = #doc]
            pub(crate) fn #method(&self #(, #parameters)*) -> ::std::result::Result<#answer, #error> {
                self.handle.#method(#(#lent),*)
            }
        }
    });
    (!unfailing.is_empty()).then(|| {
        quote! {
            #[cfg(not(target_family = "wasm"))]
            #[::uniffi::export]
            impl ExpenseGroup {
                #(#outside_wasm)*
            }

            #[cfg(target_family = "wasm")]
            #[::uniffi::export]
            impl ExpenseGroup {
                #(#on_wasm)*
            }
        }
    })
}

/// `call`'s parameters as a foreign binding's synchronous form takes them.
fn foreign_parameters(call: &Call) -> Vec<TokenStream> {
    call.parameters
        .iter()
        .map(|parameter| {
            let name = &parameter.name;
            let foreign = parameter.foreign();
            quote!(#name: #foreign)
        })
        .collect()
}

/// A mirror as the core's table hands it: the kind of `uniffi` item it
/// becomes, then the item.
struct Mirror {
    /// `Record`, `Enum`, or `Error`.
    kind: Ident,
    /// The item, without its attributes and documentation.
    item: Item,
}

impl Parse for Mirror {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let kind: Ident = input.parse()?;
        match kind.to_string().as_str() {
            "Enum" | "Error" | "Record" => Ok(Self {
                kind,
                item: input.parse()?,
            }),
            _ => Err(syn::Error::new_spanned(
                kind,
                "a mirror is a `Record`, an `Enum`, or an `Error`",
            )),
        }
    }
}

/// The `#[uniffi::remote]` copy of the mirror `input`, from which `uniffi`
/// derives the conversions of the core's item it names.
///
/// # Errors
///
/// For input that is not a kind of mirror followed by an item.
pub(crate) fn mirror(input: TokenStream) -> syn::Result<TokenStream> {
    let Mirror { kind, item } = syn::parse2(input)?;
    Ok(quote! {
        #[::uniffi::remote(#kind)]
        #item
    })
}

#[cfg(test)]
mod tests;
