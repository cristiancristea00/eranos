//! The bridge's group, form by form. A synchronous form that lent the core
//! another value, an asynchronous one that read its caller's objects after
//! it returned, a read handle's form of a change, or an outcome that handed
//! out another answer would reach every C and C++ caller of that call.

use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{ImplItem, ImplItemFn, Item};

use super::*;
use crate::capi::tests::{CALLS, RECORDS, sources};

/// The fixture's items, by name.
fn named() -> BTreeMap<String, TokenStream> {
    let core = Core::parse(&sources()).unwrap();
    items(&core)
        .unwrap()
        .into_iter()
        .map(|named| (named.name, named.tokens))
        .collect()
}

/// The methods of the `impl` among `tokens`, in order.
fn methods(tokens: &TokenStream) -> Vec<ImplItemFn> {
    syn::parse2::<syn::File>(tokens.clone())
        .unwrap()
        .items
        .into_iter()
        .filter_map(|item| {
            if let Item::Impl(implementation) = item {
                Some(implementation.items)
            } else {
                None
            }
        })
        .flatten()
        .map(|member| {
            let ImplItem::Fn(function) = member else {
                panic!("methods alone");
            };
            function
        })
        .collect()
}

/// The names of `methods`, in order.
fn names(methods: &[ImplItemFn]) -> Vec<String> {
    methods
        .iter()
        .map(|method| method.sig.ident.to_string())
        .collect()
}

/// The method `name` of `methods`: its documentation, and its code without
/// it, as it prints.
fn method(methods: &[ImplItemFn], name: &str) -> (String, String) {
    let mut method = methods
        .iter()
        .find(|method| method.sig.ident == name)
        .unwrap_or_else(|| panic!("no method {name}"))
        .clone();
    let docs = crate::docs::Docs::lines(&method.attrs)
        .unwrap()
        .join("\n\n");
    method
        .attrs
        .retain(|attribute| !attribute.path().is_ident("doc"));
    (docs, method.to_token_stream().to_string())
}

/// `code` as a method prints.
fn printed(code: TokenStream) -> String {
    syn::parse2::<ImplItemFn>(code)
        .unwrap()
        .to_token_stream()
        .to_string()
}

/// Asserts that every method of `expected` is among `methods`, written as it
/// is there.
fn assert_written(methods: &[ImplItemFn], expected: &[TokenStream]) {
    for code in expected {
        let function = syn::parse2::<ImplItemFn>(code.clone()).unwrap();
        assert_eq!(
            method(methods, &function.sig.ident.to_string()).1,
            function.to_token_stream().to_string()
        );
    }
}

/// The group has its constructors first and its read handle last, and
/// between them every call's synchronous forms then its asynchronous ones,
/// a call that takes optional text with the twin that passes none.
#[test]
fn the_group_has_every_form_of_every_call_in_order() {
    let named = named();
    assert_eq!(
        names(&methods(&named["EranosGroup"])),
        [
            "create",
            "load",
            "load_async",
            "expense",
            "expense_async",
            "expenses",
            "expenses_async",
            "name",
            "name_async",
            "description",
            "description_async",
            "is_archived",
            "is_archived_async",
            "summary",
            "summary_async",
            "add_expense",
            "add_expense_async",
            "set_members",
            "set_members_async",
            "set_language",
            "clear_language",
            "set_language_async",
            "clear_language_async",
            "move_to",
            "move_to_async",
            "read",
            "read_async",
        ]
    );
}

/// A synchronous form checks its text, lends a record or a list, converts a
/// value, and passes an integer as it is; it writes text into a
/// `DiplomatWrite`, and hands a record or a list over in an opaque object.
#[test]
fn a_synchronous_form_lends_the_core_its_parameters() {
    let named = named();
    assert_written(
        &methods(&named["EranosGroup"]),
        &[
            quote! {
                pub fn expense(&self, id: &DiplomatStr) -> Result<Option<Box<EranosExpense>>, Box<EranosError>> {
                    self.handle
                        .expense(&crate::text(id)?)
                        .map(|value| value.map(|value| Box::new(EranosExpense(value))))
                        .map_err(crate::failed)
                }
            },
            quote! {
                pub fn expenses(&self) -> Result<Box<EranosExpenseList>, Box<EranosError>> {
                    self.handle
                        .expenses()
                        .map(|list| Box::new(EranosExpenseList(list)))
                        .map_err(crate::failed)
                }
            },
            quote! {
                pub fn name(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
                    let text = self.handle.name().map_err(crate::failed)?;
                    crate::write_text(write, &text);
                    Ok(())
                }
            },
            quote! {
                pub fn description(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
                    let text = self.handle.description().map_err(crate::failed)?;
                    crate::write_optional(write, text.as_deref());
                    Ok(())
                }
            },
            quote! {
                pub fn is_archived(&self, id: &DiplomatStr) -> Result<bool, Box<EranosError>> {
                    self.handle.is_archived(&crate::text(id)?).map_err(crate::failed)
                }
            },
            quote! {
                pub fn summary(&self) -> Result<Box<EranosSummary>, Box<EranosError>> {
                    self.handle
                        .summary()
                        .map(|value| Box::new(EranosSummary(value)))
                        .map_err(crate::failed)
                }
            },
            quote! {
                pub fn add_expense(&self, expense: &EranosExpense, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
                    let text = self.handle.add_expense(&expense.0).map_err(crate::failed)?;
                    crate::write_text(write, &text);
                    Ok(())
                }
            },
            quote! {
                pub fn set_members(&self, members: &EranosTextList, items: &EranosItemList) -> Result<(), Box<EranosError>> {
                    self.handle.set_members(&members.0, &items.0).map_err(crate::failed)
                }
            },
            quote! {
                pub fn set_language(&self, language: &DiplomatStr, count: i32) -> Result<(), Box<EranosError>> {
                    self.handle
                        .set_language(Some(&crate::text(language)?), count)
                        .map_err(crate::failed)
                }
            },
            quote! {
                pub fn clear_language(&self, count: i32) -> Result<(), Box<EranosError>> {
                    self.handle.set_language(None, count).map_err(crate::failed)
                }
            },
            quote! {
                pub fn move_to(&self, date: EranosCalendarDate, distribution: EranosTipDistribution) -> Result<(), Box<EranosError>> {
                    self.handle.move_to(date.into(), distribution.into()).map_err(crate::failed)
                }
            },
        ],
    );
}

/// An asynchronous form checks its text before it returns and reports its
/// outcome to its callback on the worker thread, a record in an `Option`.
#[test]
fn an_asynchronous_read_checks_its_text_before_it_returns() {
    let named = named();
    assert_written(
        &methods(&named["EranosGroup"]),
        &[
            quote! {
                pub fn expense_async(
                    &self,
                    id: &DiplomatStr,
                    cancellation: Option<&EranosCancellation>,
                    callback: impl Fn(Box<EranosExpenseOutcome>) + 'static,
                ) {
                    let handle = self.handle.clone();
                    let cancellation = cancellation.map(EranosCancellation::shared);
                    let id = crate::decoded(id);
                    crate::spawn(
                        &self.jobs,
                        async move {
                            handle
                                .expense_async(id?, cancellation)
                                .await
                                .map_err(crate::Failure::Library)
                        },
                        callback,
                        EranosExpenseOutcome,
                    );
                }
            },
            quote! {
                pub fn summary_async(
                    &self,
                    cancellation: Option<&EranosCancellation>,
                    callback: impl Fn(Box<EranosSummaryOutcome>) + 'static,
                ) {
                    let handle = self.handle.clone();
                    let cancellation = cancellation.map(EranosCancellation::shared);
                    crate::spawn(
                        &self.jobs,
                        async move {
                            handle
                                .summary_async(cancellation)
                                .await
                                .map_err(crate::Failure::Library)
                                .map(Some)
                        },
                        callback,
                        EranosSummaryOutcome,
                    );
                }
            },
        ],
    );
}

/// An asynchronous form copies or converts every other parameter before it
/// returns, so it reads none of its caller's objects after.
#[test]
fn an_asynchronous_change_prepares_its_parameters_before_it_returns() {
    let named = named();
    assert_written(
        &methods(&named["EranosGroup"]),
        &[
            quote! {
                pub fn set_members_async(
                    &self,
                    members: &EranosTextList,
                    items: &EranosItemList,
                    cancellation: Option<&EranosCancellation>,
                    callback: impl Fn(Box<EranosUnitOutcome>) + 'static,
                ) {
                    let handle = self.handle.clone();
                    let cancellation = cancellation.map(EranosCancellation::shared);
                    let members = Ok(members.0.clone());
                    let items = Ok(items.0.clone());
                    crate::spawn(
                        &self.jobs,
                        async move {
                            handle
                                .set_members_async(members?, items?, cancellation)
                                .await
                                .map_err(crate::Failure::Library)
                        },
                        callback,
                        EranosUnitOutcome,
                    );
                }
            },
            quote! {
                pub fn move_to_async(
                    &self,
                    date: EranosCalendarDate,
                    distribution: EranosTipDistribution,
                    cancellation: Option<&EranosCancellation>,
                    callback: impl Fn(Box<EranosUnitOutcome>) + 'static,
                ) {
                    let handle = self.handle.clone();
                    let cancellation = cancellation.map(EranosCancellation::shared);
                    let date = Ok(date.into());
                    let distribution = Ok(distribution.into());
                    crate::spawn(
                        &self.jobs,
                        async move {
                            handle
                                .move_to_async(date?, distribution?, cancellation)
                                .await
                                .map_err(crate::Failure::Library)
                        },
                        callback,
                        EranosUnitOutcome,
                    );
                }
            },
        ],
    );
}

/// The asynchronous twin of a call that takes optional text passes none,
/// and the call itself passes its text checked, in `Some`.
#[test]
fn an_asynchronous_twin_passes_no_optional_text() {
    let named = named();
    assert_written(
        &methods(&named["EranosGroup"]),
        &[
            quote! {
                pub fn set_language_async(
                    &self,
                    language: &DiplomatStr,
                    count: i32,
                    cancellation: Option<&EranosCancellation>,
                    callback: impl Fn(Box<EranosUnitOutcome>) + 'static,
                ) {
                    let handle = self.handle.clone();
                    let cancellation = cancellation.map(EranosCancellation::shared);
                    let language = crate::decoded(language);
                    let count = Ok(count);
                    crate::spawn(
                        &self.jobs,
                        async move {
                            handle
                                .set_language_async(Some(language?), count?, cancellation)
                                .await
                                .map_err(crate::Failure::Library)
                        },
                        callback,
                        EranosUnitOutcome,
                    );
                }
            },
            quote! {
                pub fn clear_language_async(
                    &self,
                    count: i32,
                    cancellation: Option<&EranosCancellation>,
                    callback: impl Fn(Box<EranosUnitOutcome>) + 'static,
                ) {
                    let handle = self.handle.clone();
                    let cancellation = cancellation.map(EranosCancellation::shared);
                    let count = Ok(count);
                    crate::spawn(
                        &self.jobs,
                        async move {
                            handle
                                .set_language_async(None, count?, cancellation)
                                .await
                                .map_err(crate::Failure::Library)
                        },
                        callback,
                        EranosUnitOutcome,
                    );
                }
            },
        ],
    );
}

/// Every form carries the call's documentation and its own failures: the
/// call's refusal unless it cannot fail, the lock on wasm by what the call
/// takes, text that is not UTF-8 for a form that takes text, and the
/// released handle for the read handle's; the twin says what it passes.
#[test]
fn every_form_says_how_it_fails() {
    let named = named();
    let group = methods(&named["EranosGroup"]);
    assert_eq!(
        method(&group, "expense").0,
        format!(
            "The expense with the identifier `id`.\n\n# Errors\n\nThe library's refusal. {READ_BUSY} {INVALID_TEXT}"
        )
    );
    assert_eq!(
        method(&group, "name").0,
        format!(
            "The group's name. Writes it into `write`.\n\nIt is never blank.\n\n# Errors\n\n{READ_BUSY}"
        )
    );
    assert_eq!(
        method(&group, "description").0,
        format!(
            "The description. Writes it into `write` when there is one, and nothing otherwise.\n\n# Errors\n\n{READ_BUSY}"
        )
    );
    assert_eq!(
        method(&group, "add_expense").0,
        format!(
            "Adds `expense` and returns its identifier. Writes it into `write`.\n\n# Errors\n\nThe library's refusal. {WRITE_BUSY}"
        )
    );
    assert_eq!(
        method(&group, "clear_language").0,
        format!(
            "The form of `set_language` that passes no `language`.\n\n# Errors\n\nThe library's refusal. {WRITE_BUSY}"
        )
    );
    assert!(
        method(&group, "expense_async").0.starts_with(
            "The asynchronous form of `expense`: it runs on the group's worker thread"
        )
    );
    let reader = methods(&named["EranosGroupReader"]);
    assert_eq!(
        method(&reader, "is_archived").0,
        format!(
            "Whether the member with the identifier `id` is archived.\n\n# Errors\n\nThe library's refusal. {RELEASED} {INVALID_TEXT}"
        )
    );
}

/// The read handle has its release first, then every read's synchronous
/// form over the handle, and no change.
#[test]
fn the_read_handle_reads_through_itself() {
    let named = named();
    let reader = methods(&named["EranosGroupReader"]);
    assert_eq!(
        names(&reader),
        [
            "release",
            "expense",
            "expenses",
            "name",
            "description",
            "is_archived",
            "summary",
        ]
    );
    assert_eq!(
        method(&reader, "is_archived").1,
        printed(quote! {
            pub fn is_archived(&self, id: &DiplomatStr) -> Result<bool, Box<EranosError>> {
                self.0.is_archived(&crate::text(id)?).map_err(crate::failed)
            }
        })
    );
}

/// Each kind of answer has its outcome, which holds the call's failure or a
/// copy of its answer, a record's in an `Option` whether the call may find
/// none or not.
#[test]
fn each_kind_of_answer_has_its_outcome() {
    let named = named();
    for name in [
        "EranosExpenseListOutcome",
        "EranosExpenseOutcome",
        "EranosFlagOutcome",
        "EranosGroupOutcome",
        "EranosGroupReaderOutcome",
        "EranosOptionalTextOutcome",
        "EranosSummaryOutcome",
        "EranosTextOutcome",
        "EranosUnitOutcome",
    ] {
        assert!(named.contains_key(name), "{name}");
    }
    let held = |name: &str| {
        syn::parse2::<syn::File>(named[name].clone())
            .unwrap()
            .items
            .into_iter()
            .find_map(|item| {
                if let Item::Struct(mut record) = item {
                    record.attrs.clear();
                    Some(record.to_token_stream().to_string())
                } else {
                    None
                }
            })
            .unwrap()
    };
    assert_eq!(
        held("EranosSummaryOutcome"),
        quote! {
            pub struct EranosSummaryOutcome(pub(crate) Result<Option<eranos_bindings::Summary>, crate::Failure>);
        }
        .to_string()
    );
    assert_eq!(
        held("EranosExpenseListOutcome"),
        quote! {
            pub struct EranosExpenseListOutcome(pub(crate) Result<Vec<eranos_bindings::Expense>, crate::Failure>);
        }
        .to_string()
    );
}

/// An outcome hands its answer over as the synchronous form does: a flag,
/// text written into a `DiplomatWrite`, or a copy in an opaque object, and
/// an outcome of nothing has no value.
#[test]
fn an_outcome_hands_its_answer_over_as_the_synchronous_form_does() {
    let named = named();
    let value = |name: &str| method(&methods(&named[name]), "value").1;
    assert_eq!(
        value("EranosFlagOutcome"),
        printed(quote! {
            #[must_use]
            pub const fn value(&self) -> bool {
                matches!(self.0, Ok(true))
            }
        })
    );
    assert_eq!(
        value("EranosTextOutcome"),
        printed(quote! {
            pub fn value(&self, write: &mut DiplomatWrite) -> Option<()> {
                crate::write_optional(write, self.0.as_deref().ok())
            }
        })
    );
    assert_eq!(
        value("EranosOptionalTextOutcome"),
        printed(quote! {
            pub fn value(&self, write: &mut DiplomatWrite) -> Option<()> {
                crate::write_optional(write, self.0.as_ref().ok().and_then(Option::as_deref))
            }
        })
    );
    assert_eq!(
        value("EranosExpenseOutcome"),
        printed(quote! {
            #[must_use]
            pub fn value(&self) -> Option<Box<EranosExpense>> {
                self.0
                    .as_ref()
                    .ok()
                    .and_then(Option::as_ref)
                    .map(|value| Box::new(EranosExpense(value.clone())))
            }
        })
    );
    assert_eq!(
        value("EranosExpenseListOutcome"),
        printed(quote! {
            #[must_use]
            pub fn value(&self) -> Option<Box<EranosExpenseList>> {
                self.0
                    .as_ref()
                    .ok()
                    .map(|list| Box::new(EranosExpenseList(list.clone())))
            }
        })
    );
    assert_eq!(
        names(&methods(&named["EranosUnitOutcome"])),
        ["is_ok", "error"]
    );
    assert_eq!(
        names(&methods(&named["EranosGroupOutcome"])),
        ["is_ok", "error", "take"]
    );
    assert!(
        named["EranosFlagOutcome"]
            .to_string()
            .contains("an asynchronous call that returns an answer")
    );
}

/// A call that takes optional text twice, or without setting it, and a
/// call whose parameter or answer the bridge cannot carry, are refused.
#[test]
fn calls_the_bridge_cannot_carry_are_refused() {
    let refusal = |calls: &str| {
        let core = Core::parse(&[RECORDS.to_owned(), calls.to_owned()]).unwrap();
        items(&core)
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default()
    };
    assert_eq!(
        refusal(&CALLS.replace(
            "count: i32) -> Result<(), EranosError>",
            "count: Option<&str>) -> Result<(), EranosError>"
        )),
        "a group call takes optional text once at most"
    );
    assert_eq!(
        refusal(&CALLS.replace("fn set_language", "fn language")),
        "a group call that takes optional text sets it, and its name starts with `set_`"
    );
    assert!(
        !refusal(&CALLS.replace(
            "-> Result<bool, EranosError>",
            "-> Result<i32, EranosError>"
        ))
        .is_empty()
    );
    assert!(!refusal(&CALLS.replace("items: &[Item]", "items: &[f64]")).is_empty());
}
