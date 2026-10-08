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

/// A method of an `impl`, as it prints.
struct Method {
    /// Its code without its documentation.
    code: String,
    /// Its documentation, its paragraphs a blank line apart.
    docs: String,
}

/// The method `name` of `methods`, as it prints.
fn method(methods: &[ImplItemFn], name: &str) -> Method {
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
    Method {
        code: method.to_token_stream().to_string(),
        docs,
    }
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
            method(methods, &function.sig.ident.to_string()).code,
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
        names(&methods(&named["EranosExpenseGroup"])),
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
            "is_open",
            "is_open_async",
            "has_expense_named",
            "has_expense_named_async",
            "latest_expense",
            "latest_expense_async",
            "is_latest_itemised",
            "is_latest_itemised_async",
            "summary",
            "summary_async",
            "add_expense",
            "add_expense_async",
            "remove_expense",
            "remove_expense_async",
            "set_members",
            "set_members_async",
            "set_note",
            "set_note_async",
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
/// `DiplomatWrite`, hands a record or a list over in an opaque object, and a
/// lookup's flag in a `DiplomatOption`, which Diplomat lets a `Result` hold.
/// The group's form of a call that cannot fail returns that answer itself,
/// which `crate::waited` takes out of the core's `Result`.
#[test]
fn a_synchronous_form_lends_the_core_its_parameters() {
    let named = named();
    assert_written(
        &methods(&named["EranosExpenseGroup"]),
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
                #[must_use]
                pub fn expenses(&self) -> Box<EranosExpenseList> {
                    Box::new(EranosExpenseList(crate::waited(self.handle.expenses())))
                }
            },
            quote! {
                pub fn name(&self, write: &mut DiplomatWrite) {
                    crate::write_text(write, &crate::waited(self.handle.name()));
                }
            },
            quote! {
                pub fn description(&self, write: &mut DiplomatWrite) -> Option<()> {
                    crate::write_optional(write, crate::waited(self.handle.description()).as_deref())
                }
            },
            quote! {
                pub fn is_archived(&self, id: &DiplomatStr) -> Result<DiplomatOption<bool>, Box<EranosError>> {
                    self.handle
                        .is_archived(&crate::text(id)?)
                        .map(DiplomatOption::from)
                        .map_err(crate::failed)
                }
            },
            quote! {
                #[must_use]
                pub fn is_open(&self) -> bool {
                    crate::waited(self.handle.is_open())
                }
            },
            quote! {
                #[must_use]
                pub fn latest_expense(&self) -> Option<Box<EranosExpense>> {
                    crate::waited(self.handle.latest_expense()).map(|value| Box::new(EranosExpense(value)))
                }
            },
            quote! {
                #[must_use]
                pub fn is_latest_itemised(&self) -> DiplomatOption<bool> {
                    DiplomatOption::from(crate::waited(self.handle.is_latest_itemised()))
                }
            },
            quote! {
                #[must_use]
                pub fn summary(&self) -> Box<EranosSummary> {
                    Box::new(EranosSummary(crate::waited(self.handle.summary())))
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
                pub fn set_members(&self, members: &EranosTextList, items: &EranosExpenseItemList) -> Result<(), Box<EranosError>> {
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

/// The group's form of a call that cannot fail returns a `Result` all the
/// same when it takes text, which it checks, with its answer or with
/// nothing; one that returned the answer alone would pass bytes that are not
/// UTF-8 on to the core.
#[test]
fn a_form_that_cannot_fail_but_takes_text_checks_it() {
    let named = named();
    assert_written(
        &methods(&named["EranosExpenseGroup"]),
        &[
            quote! {
                pub fn has_expense_named(&self, name: &DiplomatStr) -> Result<bool, Box<EranosError>> {
                    Ok(crate::waited(self.handle.has_expense_named(&crate::text(name)?)))
                }
            },
            quote! {
                pub fn set_note(&self, note: &DiplomatStr) -> Result<(), Box<EranosError>> {
                    crate::waited(self.handle.set_note(&crate::text(note)?));
                    Ok(())
                }
            },
        ],
    );
}

/// An asynchronous form checks its text before it returns and reports its
/// outcome to its callback on the worker thread: a lookup's to an outcome of
/// its own, and a record the call always returns as the core returns it.
#[test]
fn an_asynchronous_read_checks_its_text_before_it_returns() {
    let named = named();
    assert_written(
        &methods(&named["EranosExpenseGroup"]),
        &[
            quote! {
                pub fn expense_async(
                    &self,
                    id: &DiplomatStr,
                    cancellation: Option<&EranosCancellation>,
                    callback: impl Fn(Box<EranosFoundExpenseOutcome>) + 'static,
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
                        EranosFoundExpenseOutcome,
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
/// returns, so it reads none of its caller's objects after, and passes it
/// without `?`: only the check of text can fail, and a `?` on any other
/// parameter would be a branch of the bridge that never runs.
#[test]
fn an_asynchronous_change_prepares_its_parameters_before_it_returns() {
    let named = named();
    assert_written(
        &methods(&named["EranosExpenseGroup"]),
        &[
            quote! {
                pub fn set_members_async(
                    &self,
                    members: &EranosTextList,
                    items: &EranosExpenseItemList,
                    cancellation: Option<&EranosCancellation>,
                    callback: impl Fn(Box<EranosUnitOutcome>) + 'static,
                ) {
                    let handle = self.handle.clone();
                    let cancellation = cancellation.map(EranosCancellation::shared);
                    let members = members.0.clone();
                    let items = items.0.clone();
                    crate::spawn(
                        &self.jobs,
                        async move {
                            handle
                                .set_members_async(members, items, cancellation)
                                .await
                                .map_err(crate::Failure::Library)
                        },
                        callback,
                        EranosUnitOutcome,
                    );
                }
            },
            quote! {
                pub fn remove_expense_async(
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
                                .remove_expense_async(id?, cancellation)
                                .await
                                .map_err(crate::Failure::Library)
                        },
                        callback,
                        EranosExpenseOutcome,
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
                    let date = date.into();
                    let distribution = distribution.into();
                    crate::spawn(
                        &self.jobs,
                        async move {
                            handle
                                .move_to_async(date, distribution, cancellation)
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
        &methods(&named["EranosExpenseGroup"]),
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
                    crate::spawn(
                        &self.jobs,
                        async move {
                            handle
                                .set_language_async(Some(language?), count, cancellation)
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
                    crate::spawn(
                        &self.jobs,
                        async move {
                            handle
                                .set_language_async(None, count, cancellation)
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

/// Every form carries the call's documentation and its own failures, if it
/// has any: the call's refusal and the lock on wasm by what the call takes,
/// unless the call cannot fail, text that is not UTF-8 for a form that
/// takes text, and the released handle for the read handle's; the twin says
/// what it passes.
#[test]
fn every_form_says_how_it_fails() {
    let named = named();
    let group = methods(&named["EranosExpenseGroup"]);
    assert_eq!(
        method(&group, "expense").docs,
        format!(
            "The expense with the identifier `id`. `None` when it finds none.\n\n# Errors\n\nThe library's refusal. {READ_BUSY} {INVALID_TEXT}"
        )
    );
    assert_eq!(
        method(&group, "name").docs,
        "The group's name. Writes it into `write`.\n\nIt is never blank."
    );
    assert_eq!(
        method(&group, "description").docs,
        "The description. Writes it into `write` when there is one, and nothing otherwise."
    );
    assert_eq!(
        method(&group, "set_note").docs,
        format!("Sets the note; blank text clears it.\n\n# Errors\n\n{INVALID_TEXT}")
    );
    assert_eq!(
        method(&group, "add_expense").docs,
        format!(
            "Adds `expense` and returns its identifier. Writes it into `write`.\n\n# Errors\n\nThe library's refusal. {WRITE_BUSY}"
        )
    );
    assert_eq!(
        method(&group, "clear_language").docs,
        format!(
            "The form of `set_language` that passes no `language`.\n\n# Errors\n\nThe library's refusal. {WRITE_BUSY}"
        )
    );
    assert!(
        method(&group, "expense_async").docs.starts_with(
            "The asynchronous form of `expense`: it runs on the group's worker thread"
        )
    );
    let reader = methods(&named["EranosExpenseGroupReader"]);
    assert_eq!(
        method(&reader, "is_archived").docs,
        format!(
            "Whether the member with the identifier `id` is archived. `None` when it finds none.\n\n# Errors\n\nThe library's refusal. {RELEASED} {INVALID_TEXT}"
        )
    );
}

/// The read handle has its release first, then every read's synchronous
/// form over the handle, and no change.
#[test]
fn the_read_handle_reads_through_itself() {
    let named = named();
    let reader = methods(&named["EranosExpenseGroupReader"]);
    assert_eq!(
        names(&reader),
        [
            "release",
            "expense",
            "expenses",
            "name",
            "description",
            "is_archived",
            "is_open",
            "has_expense_named",
            "latest_expense",
            "is_latest_itemised",
            "summary",
        ]
    );
    assert_written(
        &reader,
        &[
            quote! {
                pub fn is_archived(&self, id: &DiplomatStr) -> Result<DiplomatOption<bool>, Box<EranosError>> {
                    self.0
                        .is_archived(&crate::text(id)?)
                        .map(DiplomatOption::from)
                        .map_err(crate::failed)
                }
            },
            quote! {
                pub fn is_open(&self) -> Result<bool, Box<EranosError>> {
                    self.0.is_open().map_err(crate::failed)
                }
            },
        ],
    );
}

/// Each kind of answer has its outcome, which holds the call's failure or a
/// copy of its answer, in an `Option` only where the answer may be absent:
/// a lookup's record has an outcome apart from the record a call always
/// returns, so that the latter holds it as the core returns it.
#[test]
fn each_kind_of_answer_has_its_outcome() {
    let named = named();
    for name in [
        "EranosExpenseListOutcome",
        "EranosExpenseOutcome",
        "EranosFoundExpenseOutcome",
        "EranosFlagOutcome",
        "EranosFoundFlagOutcome",
        "EranosExpenseGroupOutcome",
        "EranosExpenseGroupReaderOutcome",
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
            pub struct EranosSummaryOutcome(pub(crate) Result<eranos_bindings::Summary, crate::Failure>);
        }
        .to_string()
    );
    assert_eq!(
        held("EranosExpenseOutcome"),
        quote! {
            pub struct EranosExpenseOutcome(pub(crate) Result<eranos_bindings::Expense, crate::Failure>);
        }
        .to_string()
    );
    assert_eq!(
        held("EranosFoundExpenseOutcome"),
        quote! {
            pub struct EranosFoundExpenseOutcome(pub(crate) Result<Option<eranos_bindings::Expense>, crate::Failure>);
        }
        .to_string()
    );
    assert_eq!(
        held("EranosFlagOutcome"),
        quote! {
            pub struct EranosFlagOutcome(pub(crate) Result<bool, crate::Failure>);
        }
        .to_string()
    );
    assert_eq!(
        held("EranosFoundFlagOutcome"),
        quote! {
            pub struct EranosFoundFlagOutcome(pub(crate) Result<Option<bool>, crate::Failure>);
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

/// An outcome hands its answer over as the synchronous form does, absent
/// when the call failed: text written into a `DiplomatWrite`, or a copy in
/// an opaque object, absent too when a lookup found none; an outcome of
/// nothing has no value.
#[test]
fn an_outcome_hands_its_answer_over_as_the_synchronous_form_does() {
    let named = named();
    let value = |name: &str| method(&methods(&named[name]), "value").code;
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
        value("EranosFoundExpenseOutcome"),
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
        value("EranosExpenseOutcome"),
        printed(quote! {
            #[must_use]
            pub fn value(&self) -> Option<Box<EranosExpense>> {
                self.0
                    .as_ref()
                    .ok()
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
        names(&methods(&named["EranosExpenseGroupOutcome"])),
        ["is_ok", "error", "take"]
    );
    assert!(
        named["EranosFoundExpenseOutcome"]
            .to_string()
            .contains("an asynchronous call that returns an expense it may not find")
    );
}

/// A flag's outcome hands the answer over when the call succeeded, and a
/// lookup's flag has an outcome of its own, which hands it over only when
/// the call also found what it answers about; one outcome for both would
/// tell a C caller that a member the group does not hold is active.
#[test]
fn a_lookup_s_flag_has_an_outcome_of_its_own() {
    let named = named();
    let value = |name: &str| method(&methods(&named[name]), "value").code;
    assert_eq!(
        value("EranosFlagOutcome"),
        printed(quote! {
            #[must_use]
            pub const fn value(&self) -> Option<bool> {
                match &self.0 {
                    Ok(answer) => Some(*answer),
                    Err(_) => None,
                }
            }
        })
    );
    assert_eq!(
        value("EranosFoundFlagOutcome"),
        printed(quote! {
            #[must_use]
            pub const fn value(&self) -> Option<bool> {
                match &self.0 {
                    Ok(answer) => *answer,
                    Err(_) => None,
                }
            }
        })
    );
    assert!(
        named["EranosFlagOutcome"]
            .to_string()
            .contains("an asynchronous call that returns an answer,")
    );
    assert!(
        named["EranosFoundFlagOutcome"].to_string().contains(
            "an asynchronous call that returns an answer about something it may not find"
        )
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
    assert_ne!(
        refusal(&CALLS.replace(
            "-> Result<bool, EranosError>",
            "-> Result<i32, EranosError>"
        )),
        ""
    );
    assert_ne!(
        refusal(&CALLS.replace("items: &[ExpenseItem]", "items: &[f64]")),
        ""
    );
}
