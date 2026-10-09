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
            fn call(group: &ExpenseGroup) -> Result<(), EranosError> {}
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
/// read's form forwards to the read handle's and every call's to the write
/// handle's; and every text is the core's documentation with its links as
/// code spans.
#[test]
fn the_objects_forward_every_form_to_the_core() {
    let calls = Calls::parse(
        &syn::parse2(quote! {
            trait ExpenseGroupCalls {
                /// The category with the identifier `id`.
                fn category(group: &ExpenseGroup, id: &str) -> Result<Option<Category>, EranosError> {}
                /// Sets the language.
                fn set_language(group: &mut ExpenseGroup, language: Option<&str>) -> Result<(), EranosError> {}
                /// The group's name.
                fn name(group: &ExpenseGroup) -> String {}
            }
        })
        .unwrap(),
    )
    .unwrap();
    assert_eq!(objects(&calls).to_string(), expected_objects().to_string());
}

/// The objects the test above expects for its trait.
fn expected_objects() -> TokenStream {
    let (group, handles) = (expected_group_objects(), expected_handle_objects());
    quote! {
        #group
        #handles
    }
}

/// The group object the test above expects for its trait: every call's
/// forms, the infallible read's two synchronous forms in blocks of their
/// own.
fn expected_group_objects() -> TokenStream {
    let (category, language, name) = (
        docs("The category with the identifier `id`."),
        docs("Sets the language."),
        docs("The group's name."),
    );
    let category_doc = foreign(&category.synchronous(Access::Read, false));
    let category_future = foreign(&category.asynchronous(false));
    let language_doc = foreign(&language.synchronous(Access::Write, false));
    let language_future = foreign(&language.asynchronous(false));
    let name_unfailing_doc = foreign(&name.unfailing());
    let name_doc = foreign(&name.synchronous(Access::Read, true));
    let name_future = foreign(&name.asynchronous(true));
    assert!(!category_doc.contains("]("));
    quote! {
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

            #[doc = #name_future]
            pub(crate) async fn name_async(
                &self,
                cancellation: ::std::option::Option<::std::sync::Arc<Cancellation>>,
            ) -> ::std::result::Result<String, EranosError> {
                self.handle.name_async(shared(cancellation)).await
            }
        }

        #[cfg(not(target_family = "wasm"))]
        #[::uniffi::export]
        impl ExpenseGroup {
            #[doc = #name_unfailing_doc]
            pub(crate) fn name(&self) -> String {
                self.handle.name()
            }
        }

        #[cfg(target_family = "wasm")]
        #[::uniffi::export]
        impl ExpenseGroup {
            #[doc = #name_doc]
            pub(crate) fn name(&self) -> ::std::result::Result<String, EranosError> {
                self.handle.name()
            }
        }

    }
}

/// The read and write handles the test above expects: every read's form
/// on the one, every call's form on the other.
fn expected_handle_objects() -> TokenStream {
    let (category, language, name) = (
        docs("The category with the identifier `id`."),
        docs("Sets the language."),
        docs("The group's name."),
    );
    let category_read = foreign(&category.reader(false));
    let name_read = foreign(&name.reader(true));
    let (category_written, language_written, name_written) = (
        foreign(&category.writer(false)),
        foreign(&language.writer(false)),
        foreign(&name.writer(true)),
    );
    quote! {
        #[::uniffi::export]
        impl ExpenseGroupReader {
            #[doc = #category_read]
            pub(crate) fn category(&self, id: &str) -> ::std::result::Result<Option<Category>, EranosError> {
                self.reader.category(id)
            }

            #[doc = #name_read]
            pub(crate) fn name(&self) -> ::std::result::Result<String, EranosError> {
                self.reader.name()
            }
        }

        #[::uniffi::export]
        impl ExpenseGroupWriter {
            #[doc = #category_written]
            pub(crate) fn category(&self, id: &str) -> ::std::result::Result<Option<Category>, EranosError> {
                self.writer.category(id)
            }

            #[doc = #language_written]
            pub(crate) fn set_language(
                &self,
                language: ::std::option::Option<::std::string::String>
            ) -> ::std::result::Result<(), EranosError> {
                self.writer.set_language(language.as_deref())
            }

            #[doc = #name_written]
            pub(crate) fn name(&self) -> ::std::result::Result<String, EranosError> {
                self.writer.name()
            }
        }
    }
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
