//! The forms written for the core: every call's synchronous and
//! asynchronous form on the group handle, every read's form on the read
//! handle, and the table. A form under the wrong lock, an answer left
//! unwrapped, a future that borrows what it outlives, or a table that lost
//! part of the trait would break every binding at once.

use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemTrait;

use super::*;
use crate::docs::Docs;

/// The trait of four calls the tests write forms of: an infallible read
/// without parameters, a fallible change of optional text, an infallible
/// change answering nothing, and a read whose asynchronous form is written
/// by hand.
fn trait_item() -> ItemTrait {
    syn::parse2(quote! {
        trait ExpenseGroupCalls {
            /// The group's name.
            #[read]
            #[infallible]
            fn name(&self) -> Result<String, EranosError>;
            /// Sets the language.
            #[write]
            fn set_language(&self, language: Option<&str>) -> Result<(), EranosError>;
            /// Sets the description.
            #[write]
            #[infallible]
            fn set_description(&self, description: &str) -> Result<(), EranosError>;
            /// The payments that settle every balance.
            #[read]
            #[async_body = bodies::settlement]
            fn settlement(&self, strategy: SettlementStrategy) -> Result<Settlement, EranosError>;
        }
    })
    .unwrap()
}

/// The documentation of the call of `trait_item` with the summary
/// `summary`.
fn docs(summary: &str) -> Docs {
    let item: ItemTrait = syn::parse2(quote! {
        trait ExpenseGroupCalls {
            #[doc = #summary]
            #[read]
            fn call(&self) -> Result<(), EranosError>;
        }
    })
    .unwrap();
    let Some(syn::TraitItem::Fn(function)) = item.items.first() else {
        panic!("one call");
    };
    Docs::parse(&function.attrs, &function.sig.ident).unwrap()
}

/// The items `expand` writes for [`trait_item`]: the group handle's forms,
/// the read handle's, and the table.
fn written() -> Vec<syn::Item> {
    let item = trait_item();
    let file: syn::File = syn::parse2(expand(&Calls::parse(&item).unwrap(), &item)).unwrap();
    file.items
}

/// `tokens` as text, for comparing the code `expand` writes.
fn text(tokens: impl ToTokens) -> String {
    tokens.to_token_stream().to_string()
}

/// Every call's forms on the group handle read under the lock the call
/// names, wrap an infallible operation's answer, move what the future
/// outlives into it, and call the hand-written body where the trait names
/// one; a form under another lock, an unwrapped answer, or a future that
/// borrows would break every binding at once.
#[test]
fn every_call_writes_both_forms_on_the_group_handle() {
    let (name, language, description, settlement) = (
        docs("The group's name."),
        docs("Sets the language."),
        docs("Sets the description."),
        docs("The payments that settle every balance."),
    );
    let (name_doc, name_future) = (
        name.synchronous(Access::Read, true),
        name.asynchronous(true),
    );
    let (language_doc, language_future) = (
        language.synchronous(Access::Write, false),
        language.asynchronous(false),
    );
    let (description_doc, description_future) = (
        description.synchronous(Access::Write, true),
        description.asynchronous(true),
    );
    let (settlement_doc, settlement_future) = (
        settlement.synchronous(Access::Read, false),
        settlement.asynchronous(false),
    );
    let expected: TokenStream = quote! {
        impl ExpenseGroupHandle {
            #[doc = #name_doc]
            pub fn name(&self) -> ::std::result::Result<String, EranosError> {
                self.reading(|group| ::std::result::Result::Ok(operations::name(group)))
            }
            #[doc = #name_future]
            pub fn name_async(
                &self,
                cancellation: ::std::option::Option<::std::sync::Arc<Cancellation>>,
            ) -> impl ::std::future::Future<Output = ::std::result::Result<String, EranosError>>
                   + ::std::marker::Send
                   + 'static {
                self.reading_async(cancellation, |group| ::std::result::Result::Ok(operations::name(group)))
            }
            #[doc = #language_doc]
            pub fn set_language(&self, language: Option<&str>) -> ::std::result::Result<(), EranosError> {
                self.writing(|group| operations::set_language(group, language))
            }
            #[doc = #language_future]
            pub fn set_language_async(
                &self,
                language: ::std::option::Option<::std::string::String>,
                cancellation: ::std::option::Option<::std::sync::Arc<Cancellation>>,
            ) -> impl ::std::future::Future<Output = ::std::result::Result<(), EranosError>>
                   + ::std::marker::Send
                   + 'static {
                self.writing_async(cancellation, move |group| operations::set_language(group, language.as_deref()))
            }
            #[doc = #description_doc]
            pub fn set_description(&self, description: &str) -> ::std::result::Result<(), EranosError> {
                self.writing(|group| { operations::set_description(group, description); ::std::result::Result::Ok(()) })
            }
            #[doc = #description_future]
            pub fn set_description_async(
                &self,
                description: ::std::string::String,
                cancellation: ::std::option::Option<::std::sync::Arc<Cancellation>>,
            ) -> impl ::std::future::Future<Output = ::std::result::Result<(), EranosError>>
                   + ::std::marker::Send
                   + 'static {
                self.writing_async(cancellation, move |group| {
                    operations::set_description(group, &description);
                    ::std::result::Result::Ok(())
                })
            }
            #[doc = #settlement_doc]
            pub fn settlement(&self, strategy: SettlementStrategy) -> ::std::result::Result<Settlement, EranosError> {
                self.reading(|group| operations::settlement(group, strategy))
            }
            #[doc = #settlement_future]
            pub fn settlement_async(
                &self,
                strategy: SettlementStrategy,
                cancellation: ::std::option::Option<::std::sync::Arc<Cancellation>>,
            ) -> impl ::std::future::Future<Output = ::std::result::Result<Settlement, EranosError>>
                   + ::std::marker::Send
                   + 'static {
                bodies::settlement(self.shared.clone(), strategy, cancellation)
            }
        }
    };
    let items = written();
    assert_eq!(3, items.len());
    assert_eq!(
        text(&items[0]),
        text(syn::parse2::<syn::Item>(expected).unwrap())
    );
}

/// Every read's form on the read handle runs under the lock the handle
/// holds, and the table hands the whole trait to the macro it is given; a
/// read handle's form that took a lock of its own, or a table that lost
/// part of the trait, would break every binding at once.
#[test]
fn every_read_writes_its_form_on_the_read_handle_and_the_table_hands_the_trait() {
    let (name, settlement) = (
        docs("The group's name.").reader(true),
        docs("The payments that settle every balance.").reader(false),
    );
    let reader: TokenStream = quote! {
        impl ExpenseGroupReader {
            #[doc = #name]
            pub fn name(&self) -> ::std::result::Result<String, EranosError> {
                self.reading(|group| ::std::result::Result::Ok(operations::name(group)))
            }
            #[doc = #settlement]
            pub fn settlement(&self, strategy: SettlementStrategy) -> ::std::result::Result<Settlement, EranosError> {
                self.reading(|group| operations::settlement(group, strategy))
            }
        }
    };
    let item = trait_item();
    let table_doc = "Hands every group call to the macro whose path it is given, as the trait `ExpenseGroupCalls` lists \
                     them, attributes and documentation included: `expense_group_calls!(path::to::callback);` expands to \
                     `path::to::callback! { trait ExpenseGroupCalls { … } }`, from which a glue crate writes its \
                     objects' methods.";
    let table: TokenStream = quote! {
        #[doc = #table_doc]
        #[macro_export]
        macro_rules! expense_group_calls {
            ($($callback:tt)+) => {
                $($callback)+ ! { #item }
            };
        }
    };
    let items = written();
    assert_eq!(
        text(&items[1]),
        text(syn::parse2::<syn::Item>(reader).unwrap())
    );
    assert_eq!(
        text(&items[2]),
        text(syn::parse2::<syn::Item>(table).unwrap())
    );
}
