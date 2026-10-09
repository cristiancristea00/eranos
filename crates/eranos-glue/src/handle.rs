//! The code the binding core's calls write: the group handle's synchronous
//! and asynchronous form of every call, the read handle's form of every
//! read, and the table that hands the calls to the glue crates.
//!
//! The code names `ExpenseGroupHandle`, `ExpenseGroupReader`, `Cancellation`, and
//! `operations` as the module that lists the calls imports them, and calls
//! the handles' `reading`, `writing`, `reading_unfailing`,
//! `writing_unfailing`, `reading_async`, and `writing_async` and the group
//! handle's `shared`, which that module can reach.
//!
//! The synchronous form of a call that cannot fail is written twice: outside
//! wasm it waits for the lock and returns the call's answer itself, and on
//! wasm, whose one thread must not wait, it tries the lock and returns a
//! `Result`, as every other form does.

use proc_macro2::{Punct, Spacing, TokenStream};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::ItemTrait;

use crate::calls::{Access, Call, Calls, Parameter};

/// The forms of every call of `calls`, and the table that hands `item`, the
/// trait that lists them, to a glue crate's macro.
pub(crate) fn expand(calls: &Calls, item: &ItemTrait) -> TokenStream {
    let forms = calls.calls.iter().map(|call| {
        let synchronous = synchronous(call);
        let asynchronous = asynchronous(call);
        quote! { #synchronous #asynchronous }
    });
    let reads = calls
        .calls
        .iter()
        .filter(|call| Access::Read == call.access)
        .map(reader);
    let table = table(calls, item);
    quote! {
        impl ExpenseGroupHandle {
            #(#forms)*
        }

        impl ExpenseGroupReader {
            #(#reads)*
        }

        #table
    }
}

/// The run of `call`'s operation, which shares its name, on `group` with
/// `arguments`, its answer wrapped in `Ok` when the operation cannot fail.
fn operation(call: &Call, arguments: &[TokenStream]) -> TokenStream {
    let name = &call.name;
    let run = quote_spanned! { call.span => operations::#name(group #(, #arguments)*) };
    if !call.infallible {
        run
    } else if call.answers_nothing() {
        quote_spanned! { call.span => { #run; ::std::result::Result::Ok(()) } }
    } else {
        quote_spanned! { call.span => ::std::result::Result::Ok(#run) }
    }
}

/// The group handle's synchronous form of `call`, which takes its lock for
/// the call: one form, or, for a call that cannot fail, its form outside
/// wasm, which answers the call's value itself, and its form on wasm, which
/// tries the lock and answers a `Result`.
fn synchronous(call: &Call) -> TokenStream {
    let doc = call.docs.synchronous(call.access, call.infallible);
    let lock = match call.access {
        Access::Read => quote!(reading),
        Access::Write => quote!(writing),
    };
    let arguments: Vec<TokenStream> = call
        .parameters
        .iter()
        .map(|parameter| parameter.name.to_token_stream())
        .collect();
    let operation = operation(call, &arguments);
    let signature = signature(call);
    if !call.infallible {
        return quote_spanned! { call.span =>
            #[doc = #doc]
            #signature {
                self.#lock(|group| #operation)
            }
        };
    }
    let unfailing_doc = call.docs.unfailing();
    let unfailing_lock = match call.access {
        Access::Read => quote!(reading_unfailing),
        Access::Write => quote!(writing_unfailing),
    };
    let unfailing_signature = unfailing_signature(call);
    let name = &call.name;
    let must_use = (!call.answers_nothing()).then(|| quote!(#[must_use]));
    let unfailing_operation = if call.parameters.is_empty() {
        quote_spanned! { call.span => operations::#name }
    } else {
        quote_spanned! { call.span => |group| operations::#name(group #(, #arguments)*) }
    };
    quote_spanned! { call.span =>
        #[doc = #unfailing_doc]
        #must_use
        #[cfg(not(target_family = "wasm"))]
        #unfailing_signature {
            self.#unfailing_lock(#unfailing_operation)
        }
        #[doc = #doc]
        #[cfg(target_family = "wasm")]
        #signature {
            self.#lock(|group| #operation)
        }
    }
}

/// The group handle's asynchronous form of `call`: a future of its owned
/// parameters that waits for the lock, runs the operation, and stops when
/// its cancellation is cancelled first, or the function that
/// `#[async_body]` names, which takes the handle's shared group.
fn asynchronous(call: &Call) -> TokenStream {
    let doc = call.docs.asynchronous(call.infallible);
    let method = format_ident!("{}_async", call.name, span = call.span);
    let parameters = call.parameters.iter().map(|parameter| {
        let name = &parameter.name;
        let owned = parameter.owned();
        quote!(#name: #owned)
    });
    let body = call.asynchronous.as_ref().map_or_else(
        || {
            let borrowed: Vec<TokenStream> =
                call.parameters.iter().map(Parameter::borrowed).collect();
            let operation = operation(call, &borrowed);
            let lock = match call.access {
                Access::Read => quote!(reading_async),
                Access::Write => quote!(writing_async),
            };
            let capture = if call.parameters.is_empty() {
                quote!()
            } else {
                quote!(move)
            };
            quote_spanned! { call.span => self.#lock(cancellation, #capture |group| #operation) }
        },
        |path| {
            let names = call.parameters.iter().map(|parameter| &parameter.name);
            quote_spanned! { call.span => #path(self.shared.clone(), #(#names,)* cancellation) }
        },
    );
    let (answer, error) = (&call.answer, &call.error);
    quote_spanned! { call.span =>
        #[doc = #doc]
        pub fn #method(
            &self,
            #(#parameters,)*
            cancellation: ::std::option::Option<::std::sync::Arc<Cancellation>>,
        ) -> impl ::std::future::Future<Output = ::std::result::Result<#answer, #error>>
               + ::std::marker::Send
               + 'static {
            #body
        }
    }
}

/// The read handle's form of `call`, a read, which runs under the lock the
/// handle holds.
fn reader(call: &Call) -> TokenStream {
    let doc = call.docs.reader(call.infallible);
    let arguments: Vec<TokenStream> = call
        .parameters
        .iter()
        .map(|parameter| parameter.name.to_token_stream())
        .collect();
    let operation = operation(call, &arguments);
    let signature = signature(call);
    quote_spanned! { call.span =>
        #[doc = #doc]
        #signature {
            self.reading(|group| #operation)
        }
    }
}

/// `call`'s signature outside wasm for a call that cannot fail, public:
/// the trait's, answering the value itself, or nothing.
fn unfailing_signature(call: &Call) -> TokenStream {
    let method = &call.name;
    let parameters = call.parameters.iter().map(|parameter| {
        let name = &parameter.name;
        let written = &parameter.written;
        quote!(#name: #written)
    });
    if call.answers_nothing() {
        quote_spanned! { call.span => pub fn #method(&self #(, #parameters)*) }
    } else {
        let answer = &call.answer;
        quote_spanned! { call.span => pub fn #method(&self #(, #parameters)*) -> #answer }
    }
}

/// `call`'s signature as the trait declares it, public.
fn signature(call: &Call) -> TokenStream {
    let method = &call.name;
    let parameters = call.parameters.iter().map(|parameter| {
        let name = &parameter.name;
        let written = &parameter.written;
        quote!(#name: #written)
    });
    let (answer, error) = (&call.answer, &call.error);
    quote_spanned! { call.span =>
        pub fn #method(&self #(, #parameters)*) -> ::std::result::Result<#answer, #error>
    }
}

/// The exported macro, named after the trait in snake case, that hands
/// `item`, the trait, to a glue crate's macro, whose path it takes.
fn table(calls: &Calls, item: &ItemTrait) -> TokenStream {
    let name = format_ident!("{}", crate::snake_case(&calls.name.to_string()));
    let doc = format!(
        "Hands every group call to the macro whose path it is given, as the trait `{trait}` lists \
         them, attributes and documentation included: `{name}!(path::to::callback);` expands to \
         `path::to::callback! {{ trait {trait} {{ … }} }}`, from which a glue crate writes its \
         objects' methods.",
        trait = calls.name,
    );
    let dollar = Punct::new('$', Spacing::Alone);
    quote! {
        #[doc = #doc]
        #[macro_export]
        macro_rules! #name {
            (#dollar (#dollar callback:tt)+) => {
                #dollar (#dollar callback)+ ! { #item }
            };
        }
    }
}

#[cfg(test)]
mod tests;
