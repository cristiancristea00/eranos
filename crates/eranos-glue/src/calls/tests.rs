//! The trait of group calls is the one list the core's functions and forms,
//! the read and write handles', and every glue crate's methods are written
//! from. A call read with the wrong lock, an answer taken for a failure, a
//! parameter owned or lent the wrong way, or a trait the rules refuse let
//! through would reach every binding at once.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{FnArg, ItemTrait, TraitItem, parse_quote};

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
/// or a foreign call would take a type `uniffi` cannot lower; its function
/// would run another body than the trait's.
#[test]
fn every_kind_of_parameter_is_owned_lent_and_crossed_by_its_rule() {
    let calls = calls(quote! {
        trait ExpenseGroupCalls {
            /// Takes one parameter of each kind.
            fn change(
                group: &mut ExpenseGroup,
                id: &str,
                language: Option<&str>,
                category: &Category,
                rates: &[CurrencyRate],
                strategy: SettlementStrategy,
                note: Option<String>,
            ) -> Result<SettlementPlan, EranosError> {
                group.settle(strategy)
            }
        }
    })
    .unwrap();
    assert_eq!(text(&calls.name), "ExpenseGroupCalls");
    let [call] = calls.calls.as_slice() else {
        panic!("one call");
    };
    assert_eq!(Access::Write, call.access);
    assert_eq!(text(&call.name), "change");
    assert_eq!(text(&call.answer), text(quote!(SettlementPlan)));
    assert!(!call.infallible);
    assert!(call.asynchronous.is_none());
    assert!(!call.answers_nothing());
    assert_eq!(
        text(&call.function),
        text(quote! {
            #[doc = r" Takes one parameter of each kind."]
            fn change(
                group: &mut ExpenseGroup,
                id: &str,
                language: Option<&str>,
                category: &Category,
                rates: &[CurrencyRate],
                strategy: SettlementStrategy,
                note: Option<String>,
            ) -> Result<SettlementPlan, EranosError> {
                group.settle(strategy)
            }
        })
    );
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
            text(quote!(Option<String>)),
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
            text(quote!(note)),
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
            text(quote!(Option<String>)),
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
            text(quote!(note)),
        ]
    );
}

/// A read taken for a change would run under the shared lock, an answer
/// taken for a failure, or a failure for an answer, would be wrapped or
/// left unwrapped the wrong way, and a call whose asynchronous form is
/// written by hand would get a derived one.
#[test]
fn a_call_s_group_answer_and_attribute_set_its_lock_its_fallibility_and_its_asynchronous_body() {
    let calls = calls(quote! {
        trait ExpenseGroupCalls {
            /// The name.
            fn name(group: &ExpenseGroup) -> String {}
            /// The note.
            fn note(group: &eranos::ExpenseGroup) -> Option<String> {}
            /// The settlement.
            #[async_body = bodies::settlement]
            fn settlement(group: &ExpenseGroup) -> std::result::Result<SettlementPlan, crate::EranosError> {}
            /// Renames the group.
            fn rename(group: &mut ExpenseGroup) {}
            /// Whether the group is open, or the failure.
            fn open(group: &ExpenseGroup) -> Option<Result<bool, EranosError>> {}
            /// The group's label.
            fn label(group: &ExpenseGroup) -> &'static str {}
        }
    })
    .unwrap();
    let [name, note, settlement, rename, open, label] = calls.calls.as_slice() else {
        panic!("six calls");
    };
    assert_eq!(Access::Read, name.access);
    assert!(name.infallible);
    assert_eq!(text(&name.answer), text(quote!(String)));
    assert!(name.asynchronous.is_none());
    assert!(name.parameters.is_empty());
    assert_eq!(Access::Read, note.access);
    assert!(note.infallible);
    assert_eq!(text(&note.answer), text(quote!(Option<String>)));
    assert!(!settlement.infallible);
    assert_eq!(text(&settlement.answer), text(quote!(SettlementPlan)));
    assert_eq!(
        settlement.asynchronous.as_ref().map(text),
        Some(text(quote!(bodies::settlement)))
    );
    assert_eq!(Access::Write, rename.access);
    assert!(rename.infallible);
    assert!(rename.answers_nothing());
    assert!(open.infallible);
    let found: Type = parse_quote!(Option<Result<bool, EranosError>>);
    assert_eq!(text(&open.answer), text(found));
    assert!(label.infallible);
    assert_eq!(text(&label.answer), text(quote!(&'static str)));
}

/// A call with an attribute the rules do not know, such as the lock or the
/// fallibility a call states by its group and its answer, would be written
/// with the attribute silently dropped.
#[test]
fn a_call_takes_its_asynchronous_body_alone_besides_its_documentation() {
    let call = |attributes: TokenStream| {
        refusal(quote! {
            trait ExpenseGroupCalls {
                /// A call.
                #attributes
                fn call(group: &ExpenseGroup) {}
            }
        })
    };
    for attributes in [quote!(#[read]), quote!(#[infallible]), quote!(#[cfg(test)])] {
        assert_eq!(
            call(attributes.clone()),
            "a group call takes `#[async_body = path]` alone besides its documentation",
            "{attributes}"
        );
    }
    assert_ne!(call(quote!(#[async_body])), "");
    assert_eq!(
        call(quote!(#[async_body = "bodies::change"])),
        "`#[async_body = path]` names the function the asynchronous form runs"
    );
}

/// A call that is not a plain function with a body, or that does not take
/// the group first by reference, could not be run under a lock by a form
/// that hands it the group.
#[test]
fn a_call_is_a_plain_function_with_a_body_that_takes_the_group_first() {
    let signature = |signature: TokenStream| {
        refusal(quote! {
            trait ExpenseGroupCalls {
                /// A call.
                #signature
            }
        })
    };
    for plain in [
        quote!(
            const fn call(group: &ExpenseGroup) {}
        ),
        quote!(
            async fn call(group: &ExpenseGroup) {}
        ),
        quote!(
            unsafe fn call(group: &ExpenseGroup) {}
        ),
        quote!(
            extern "C" fn call(group: &ExpenseGroup) {}
        ),
        quote!(
            fn call<Answer>(group: &ExpenseGroup) -> Answer {}
        ),
        quote!(
            fn call(group: &ExpenseGroup)
            where
                Self: Sized,
            {
            }
        ),
    ] {
        assert_eq!(
            signature(plain.clone()),
            "a group call is a plain function",
            "{plain}"
        );
    }
    assert_eq!(
        signature(quote!(
            fn call(group: &ExpenseGroup);
        )),
        "a group call has a body, which its forms run under the lock"
    );
    for first in [
        quote!(
            fn call(&self) {}
        ),
        quote!(
            fn call(self) {}
        ),
        quote!(
            fn call() {}
        ),
        quote!(
            fn call(id: &str) {}
        ),
        quote!(
            fn call(group: ExpenseGroup) {}
        ),
        quote!(
            fn call(group: &Member) {}
        ),
        quote!(
            fn call(groups: &[ExpenseGroup]) {}
        ),
    ] {
        assert_eq!(
            signature(first.clone()),
            "a group call takes the group first, as `&ExpenseGroup` to read it or `&mut ExpenseGroup` to change it",
            "{first}"
        );
    }
}

/// A receiver after the group, which syn builds but never parses, would
/// reach the forms as a parameter they cannot write.
#[test]
fn a_receiver_after_the_group_is_refused() {
    let mut item: ItemTrait = syn::parse2(quote! {
        trait ExpenseGroupCalls {
            /// A call.
            fn call(group: &ExpenseGroup, id: &str) {}
        }
    })
    .unwrap();
    let Some(TraitItem::Fn(function)) = item.items.first_mut() else {
        panic!("one call");
    };
    let receiver: FnArg = parse_quote!(&self);
    function.sig.inputs.push(receiver);
    assert!(matches!(
        function.sig.inputs.last(),
        Some(FnArg::Receiver(_))
    ));
    assert_eq!(
        Calls::parse(&item).err().unwrap().to_string(),
        "a group call takes no `self`"
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
                fn call(group: &ExpenseGroup, #parameter) {}
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

/// A call that can fail with another error than the bindings' one, or whose
/// `Result` does not name its answer and its error, would leave its forms an
/// error they cannot report or an answer they cannot name.
#[test]
fn a_call_that_can_fail_answers_with_a_result_of_its_answer_and_eranos_error() {
    for output in [
        quote!(-> Result),
        quote!(-> Result<String>),
        quote!(-> Result<String, EranosError, EranosError>),
        quote!(-> Result<'static, String>),
        quote!(-> Result<String, MoneyError>),
        quote!(-> Result<String, &EranosError>),
    ] {
        assert_eq!(
            refusal(quote! {
                trait ExpenseGroupCalls {
                    /// A call.
                    fn call(group: &ExpenseGroup) #output {}
                }
            }),
            "a group call that can fail answers with a `Result` of its answer and `EranosError`",
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
                fn call(group: &ExpenseGroup) {}
            }
        }),
        "a group call is documented, its summary ending with a full stop"
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
            "the group calls' trait is a plain trait of functions",
            "{heading}"
        );
    }
    assert_eq!(
        refusal(quote! {
            trait ExpenseGroupCalls {
                const LIMIT: usize;
            }
        }),
        "the group calls' trait holds functions alone"
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
