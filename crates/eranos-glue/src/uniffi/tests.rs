//! The `uniffi` glue's methods and copies, written from the core's tables.
//! A method that forwarded to another form, took a type `uniffi` cannot
//! lower, or showed a Rust link in a foreign binding's documentation, or a
//! copy of another kind than its item, would reach Swift, Kotlin, Java,
//! Kotlin Multiplatform, and TypeScript alike.

use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemTrait;

use super::*;

/// The documentation of a call with the summary `summary`.
fn docs(summary: &str) -> crate::docs::Docs {
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
    crate::docs::Docs::parse(&function.attrs, &function.sig.ident).unwrap()
}

/// Every call's synchronous form forwards to the core's, its optional text
/// taken owned and lent; its asynchronous form owns every parameter, hands
/// the core the cancellation it holds, and awaits the core's future; every
/// read's form forwards to the read handle's; and every text is the core's
/// documentation with its links as code spans.
#[test]
fn the_objects_forward_every_form_to_the_core() {
    let calls = Calls::parse(
        &syn::parse2(quote! {
            trait ExpenseGroupCalls {
                /// The category with the identifier `id`.
                #[read]
                fn category(&self, id: &str) -> Result<Option<Category>, EranosError>;
                /// Sets the language.
                #[write]
                fn set_language(&self, language: Option<&str>) -> Result<(), EranosError>;
            }
        })
        .unwrap(),
    )
    .unwrap();
    let (category, language) = (
        docs("The category with the identifier `id`."),
        docs("Sets the language."),
    );
    let category_doc = foreign(&category.synchronous(Access::Read, false));
    let category_future = foreign(&category.asynchronous(false));
    let category_read = foreign(&category.reader(false));
    let language_doc = foreign(&language.synchronous(Access::Write, false));
    let language_future = foreign(&language.asynchronous(false));
    assert!(!category_doc.contains("]("));
    let expected: TokenStream = quote! {
        #[::uniffi::export]
        impl ExpenseGroup {
            #[doc = #category_doc]
            pub(crate) fn category(&self, id: &str) -> ::std::result::Result<Option<Category>, EranosError> {
                self.handle.category(id)
            }

            #[doc = #category_future]
            pub(crate) async fn category_async(
                &self,
                id: ::std::string::String,
                cancellation: ::std::option::Option<::std::sync::Arc<Cancellation>>,
            ) -> ::std::result::Result<Option<Category>, EranosError> {
                self.handle.category_async(id, shared(cancellation)).await
            }

            #[doc = #language_doc]
            pub(crate) fn set_language(
                &self,
                language: ::std::option::Option<::std::string::String>
            ) -> ::std::result::Result<(), EranosError> {
                self.handle.set_language(language.as_deref())
            }

            #[doc = #language_future]
            pub(crate) async fn set_language_async(
                &self,
                language: ::std::option::Option<::std::string::String>,
                cancellation: ::std::option::Option<::std::sync::Arc<Cancellation>>,
            ) -> ::std::result::Result<(), EranosError> {
                self.handle.set_language_async(language, shared(cancellation)).await
            }
        }

        #[::uniffi::export]
        impl ExpenseGroupReader {
            #[doc = #category_read]
            pub(crate) fn category(&self, id: &str) -> ::std::result::Result<Option<Category>, EranosError> {
                self.reader.category(id)
            }
        }
    };
    assert_eq!(objects(&calls).to_string(), expected.to_string());
}

/// Each mirror becomes the `uniffi` copy of its kind, its item as the
/// mirror hands it; a mirror of another kind, or without an item, is
/// refused rather than copied as something `uniffi` would misread.
#[test]
fn a_mirror_becomes_the_remote_copy_of_its_kind() {
    let item = quote!(
        pub struct Money {
            pub currency: String,
            pub minor_units: i64,
        }
    );
    assert_eq!(
        mirror(quote!(Record #item)).unwrap().to_string(),
        quote!(#[::uniffi::remote(Record)] #item).to_string()
    );
    for kind in ["Enum", "Error"] {
        let kind = Ident::new(kind, proc_macro2::Span::call_site());
        let choice = quote!(
            pub enum Balance {
                Settled,
            }
        );
        assert_eq!(
            mirror(quote!(#kind #choice)).unwrap().to_string(),
            quote!(#[::uniffi::remote(#kind)] #choice).to_string()
        );
    }
    assert_eq!(
        mirror(quote!(Object #item)).unwrap_err().to_string(),
        "a mirror is a `Record`, an `Enum`, or an `Error`"
    );
    assert!(mirror(quote!(Record)).is_err());
    assert!(mirror(quote!(#item)).is_err());
}
