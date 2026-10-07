//! The trait of group calls is the one list the core's forms, the read
//! handle's, and every glue crate's methods are written from. A call read
//! with the wrong lock, a parameter owned or lent the wrong way, or a trait
//! the rules refuse let through would reach every binding at once.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{FnArg, ItemTrait, TraitItem};

use super::*;

/// The calls `item` lists.
fn calls(item: TokenStream) -> syn::Result<Calls> {
    Calls::parse(&syn::parse2(item)?)
}

/// Why `item` is refused.
fn refusal(item: TokenStream) -> String {
    calls(item).err().unwrap().to_string()
}

/// `tokens` as text, for comparing the code the rules write.
fn text(tokens: impl ToTokens) -> String {
    tokens.to_token_stream().to_string()
}

/// A call's parameters would reach its forms owned, lent, or typed the
/// wrong way, so an asynchronous call would borrow what its future outlives
/// or a foreign call would take a type `uniffi` cannot lower.
#[test]
fn every_kind_of_parameter_is_owned_lent_and_crossed_by_its_rule() {
    let calls = calls(quote! {
        trait ExpenseGroupCalls {
            /// Takes one parameter of each kind.
            #[write]
            fn change(
                &self,
                id: &str,
                language: Option<&str>,
                category: &Category,
                rates: &[CurrencyRate],
                strategy: SettlementStrategy,
            ) -> Result<Settlement, EranosError>;
        }
    })
    .unwrap();
    assert_eq!(text(&calls.name), "ExpenseGroupCalls");
    let [call] = calls.calls.as_slice() else {
        panic!("one call");
    };
    assert_eq!(Access::Write, call.access);
    assert_eq!(text(&call.name), "change");
    assert_eq!(text(&call.answer), text(quote!(Settlement)));
    assert_eq!(text(&call.error), text(quote!(EranosError)));
    assert!(!call.infallible);
    assert!(call.asynchronous.is_none());
    assert!(!call.answers_nothing());
    let forms = |form: fn(&Parameter) -> TokenStream| -> Vec<String> {
        call.parameters
            .iter()
            .map(|parameter| text(form(parameter)))
            .collect()
    };
    assert_eq!(
        forms(Parameter::owned),
        [
            text(quote!(::std::string::String)),
            text(quote!(::std::option::Option<::std::string::String>)),
            text(quote!(Category)),
            text(quote!(::std::vec::Vec<CurrencyRate>)),
            text(quote!(SettlementStrategy)),
        ]
    );
    assert_eq!(
        forms(Parameter::borrowed),
        [
            text(quote!(&id)),
            text(quote!(language.as_deref())),
            text(quote!(&category)),
            text(quote!(&rates)),
            text(quote!(strategy)),
        ]
    );
    assert_eq!(
        forms(Parameter::foreign),
        [
            text(quote!(&str)),
            text(quote!(::std::option::Option<::std::string::String>)),
            text(quote!(&Category)),
            text(quote!(&[CurrencyRate])),
            text(quote!(SettlementStrategy)),
        ]
    );
    assert_eq!(
        forms(Parameter::lent),
        [
            text(quote!(id)),
            text(quote!(language.as_deref())),
            text(quote!(category)),
            text(quote!(rates)),
            text(quote!(strategy)),
        ]
    );
}

/// A read taken for a change would run under the shared lock, an
/// infallible operation's answer would not be wrapped, and a call whose
/// asynchronous form is written by hand would get a derived one.
#[test]
fn a_call_s_attributes_set_its_lock_its_fallibility_and_its_asynchronous_body() {
    let calls = calls(quote! {
        trait ExpenseGroupCalls {
            /// The name.
            #[read]
            #[infallible]
            fn name(&self) -> Result<String, EranosError>;
            /// The settlement.
            #[read]
            #[async_body = bodies::settlement]
            fn settlement(&self) -> Result<Settlement, EranosError>;
            /// Renames the group.
            #[write]
            #[infallible]
            fn rename(&self) -> Result<(), EranosError>;
        }
    })
    .unwrap();
    let [name, settlement, rename] = calls.calls.as_slice() else {
        panic!("three calls");
    };
    assert_eq!(Access::Read, name.access);
    assert!(name.infallible);
    assert!(name.asynchronous.is_none());
    assert!(!settlement.infallible);
    assert_eq!(
        settlement.asynchronous.as_ref().map(text),
        Some(text(quote!(bodies::settlement)))
    );
    assert!(rename.answers_nothing());
    assert!(name.parameters.is_empty());
}

/// A call without its lock, or with an attribute the rules do not know,
/// would be written under a guessed lock or with an attribute silently
/// dropped.
#[test]
fn a_call_s_attributes_are_its_lock_and_the_known_ones() {
    let call = |attributes: TokenStream| {
        refusal(quote! {
            trait ExpenseGroupCalls {
                /// A call.
                #attributes
                fn call(&self) -> Result<(), EranosError>;
            }
        })
    };
    assert_eq!(
        call(quote!()),
        "a group call says whether it reads or writes the group, with `#[read]` or `#[write]`"
    );
    assert_eq!(
        call(quote!(#[read] #[write])),
        "a group call either reads or writes the group"
    );
    assert_eq!(
        call(quote!(#[read] #[cfg(test)])),
        "a group call takes `#[read]` or `#[write]`, `#[infallible]`, and `#[async_body = path]`"
    );
    for attributes in [
        quote!(#[read(shared)]),
        quote!(#[write] #[infallible(always)]),
        quote!(#[write] #[async_body]),
    ] {
        assert!(!call(attributes.clone()).is_empty(), "{attributes}");
    }
    assert_eq!(
        call(quote!(#[write] #[async_body = "bodies::change"])),
        "`#[async_body = path]` names the function the asynchronous form runs"
    );
}

/// A call that is not a plain `&self` method could not be forwarded by a
/// form that takes `&self` and returns what the operation returns.
#[test]
fn a_call_is_a_plain_method_of_self_by_reference() {
    let signature = |signature: TokenStream| {
        refusal(quote! {
            trait ExpenseGroupCalls {
                /// A call.
                #[read]
                #signature;
            }
        })
    };
    for plain in [
        quote!(const fn call(&self) -> Result<(), EranosError>),
        quote!(async fn call(&self) -> Result<(), EranosError>),
        quote!(unsafe fn call(&self) -> Result<(), EranosError>),
        quote!(extern "C" fn call(&self) -> Result<(), EranosError>),
        quote!(fn call<Answer>(&self) -> Result<Answer, EranosError>),
        quote!(fn call(&self) -> Result<(), EranosError> where Self: Sized),
    ] {
        assert_eq!(
            signature(plain.clone()),
            "a group call is a plain method, declared without a body",
            "{plain}"
        );
    }
    assert_eq!(
        refusal(quote! {
            trait ExpenseGroupCalls {
                /// A call.
                #[read]
                fn call(&self) -> Result<(), EranosError> {}
            }
        }),
        "a group call is a plain method, declared without a body"
    );
    for receiver in [
        quote!(fn call(self) -> Result<(), EranosError>),
        quote!(fn call(&mut self) -> Result<(), EranosError>),
        quote!(fn call(&'group self) -> Result<(), EranosError>),
        quote!(fn call() -> Result<(), EranosError>),
        quote!(fn call(id: &str) -> Result<(), EranosError>),
    ] {
        assert_eq!(
            signature(receiver.clone()),
            "a group call takes `&self`",
            "{receiver}"
        );
    }
}

/// A receiver after the first parameter, which syn builds but never parses,
/// would reach the forms as a parameter they cannot write.
#[test]
fn a_second_receiver_is_refused() {
    let mut item: ItemTrait = syn::parse2(quote! {
        trait ExpenseGroupCalls {
            /// A call.
            #[read]
            fn call(&self, id: &str) -> Result<(), EranosError>;
        }
    })
    .unwrap();
    let Some(TraitItem::Fn(function)) = item.items.first_mut() else {
        panic!("one call");
    };
    let receiver = function.sig.inputs.first().unwrap().clone();
    function.sig.inputs.push(receiver);
    assert!(matches!(
        function.sig.inputs.last(),
        Some(FnArg::Receiver(_))
    ));
    assert_eq!(
        Calls::parse(&item).err().unwrap().to_string(),
        "a group call takes `&self` first and once"
    );
}

/// A parameter with a pattern, or of a type no rule covers, would be
/// written into forms that cannot name, own, or lend it.
#[test]
fn a_parameter_is_a_plain_name_of_a_type_a_rule_covers() {
    let parameter = |parameter: TokenStream| {
        refusal(quote! {
            trait ExpenseGroupCalls {
                /// A call.
                #[read]
                fn call(&self, #parameter) -> Result<(), EranosError>;
            }
        })
    };
    for pattern in [
        quote!((first, second): (Member, Member)),
        quote!(mut id: String),
        quote!(ref id: String),
        quote!(id @ _: String),
    ] {
        assert_eq!(
            parameter(pattern.clone()),
            "a group call names each parameter plainly",
            "{pattern}"
        );
    }
    assert_eq!(
        parameter(quote!(member: &mut Member)),
        "a group call takes no parameter by mutable reference"
    );
    for unsupported in [
        quote!(pair: &(Member, Member)),
        quote!(pair: (Member, Member)),
    ] {
        assert_eq!(
            parameter(unsupported.clone()),
            "a group call takes text, optional text, a record by reference, a list by slice, or a value",
            "{unsupported}"
        );
    }
}

/// A call that answered with anything but a `Result` of an answer and an
/// error would leave its forms, which report a lock's failure, no error to
/// fail with.
#[test]
fn a_call_answers_with_a_result_of_its_answer_and_its_error() {
    for output in [
        quote!(),
        quote!(-> String),
        quote!(-> (String, EranosError)),
        quote!(-> Result<String>),
        quote!(-> Result<String, EranosError, EranosError>),
        quote!(-> Option<String, EranosError>),
        quote!(-> Result<'static, String>),
    ] {
        assert_eq!(
            refusal(quote! {
                trait ExpenseGroupCalls {
                    /// A call.
                    #[read]
                    fn call(&self) #output;
                }
            }),
            "a group call answers with a `Result` of its answer and its error",
            "{output}"
        );
    }
}

/// A call without documentation would reach every binding undocumented.
#[test]
fn a_call_is_documented() {
    assert_eq!(
        refusal(quote! {
            trait ExpenseGroupCalls {
                #[read]
                fn call(&self) -> Result<(), EranosError>;
            }
        }),
        "a group call is documented, its summary ending with a full stop"
    );
}

/// An answer's type of no path segment, which syn builds but never parses,
/// would reach the forms as a `Result` they cannot name.
#[test]
fn an_answer_of_no_path_is_refused() {
    let mut item: ItemTrait = syn::parse2(quote! {
        trait ExpenseGroupCalls {
            /// A call.
            #[read]
            fn call(&self) -> Result<(), EranosError>;
        }
    })
    .unwrap();
    let Some(TraitItem::Fn(function)) = item.items.first_mut() else {
        panic!("one call");
    };
    let ReturnType::Type(_, output) = &mut function.sig.output else {
        panic!("an answer");
    };
    let Type::Path(path) = &mut **output else {
        panic!("a path");
    };
    path.path.segments.clear();
    assert_eq!(
        Calls::parse(&item).err().unwrap().to_string(),
        "a group call answers with a `Result` of its answer and its error"
    );
}

/// A trait with generics, bounds, or an item that is not a call would be
/// written into forms that could not state them.
#[test]
fn the_trait_is_a_plain_trait_of_calls() {
    for heading in [
        quote!(trait ExpenseGroupCalls<Answer>),
        quote!(trait ExpenseGroupCalls: Send),
        quote!(unsafe trait ExpenseGroupCalls),
        quote!(trait ExpenseGroupCalls where Self: Sized),
    ] {
        assert_eq!(
            refusal(quote!(#heading {})),
            "the group calls' trait is a plain trait of methods",
            "{heading}"
        );
    }
    assert_eq!(
        refusal(quote! {
            trait ExpenseGroupCalls {
                const LIMIT: usize;
            }
        }),
        "the group calls' trait holds methods alone"
    );
    assert!(
        calls(quote!(
            trait ExpenseGroupCalls {}
        ))
        .unwrap()
        .calls
        .is_empty()
    );
}
