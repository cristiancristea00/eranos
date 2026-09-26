//! The bridge's records, lists, and enums, method by method. A getter that
//! handed out a borrowed value, a setter a record a call cannot take, a
//! constructor that asked for an optional field, or a `const` that clippy
//! does not ask for would build a C API other than the one the C and C++
//! packages call, or a bridge the gates refuse.

use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{ImplItem, Item};

use super::*;
use crate::capi::tests::{CALLS, RECORDS, sources};

/// The fixture's items, by name.
fn named() -> BTreeMap<String, TokenStream> {
    let core = Core::parse(&sources()).unwrap();
    items(&core, &Plan::of(&core).unwrap())
        .unwrap()
        .into_iter()
        .map(|named| (named.name, named.tokens))
        .collect()
}

/// The items of `tokens`, parsed.
fn parsed(tokens: &TokenStream) -> Vec<Item> {
    syn::parse2::<syn::File>(tokens.clone()).unwrap().items
}

/// The methods of the `impl` among `tokens`, in order: each one's name, its
/// documentation, and its code without the documentation, as it prints.
fn methods(tokens: &TokenStream) -> Vec<(String, String, String)> {
    parsed(tokens)
        .iter()
        .filter_map(|item| {
            if let Item::Impl(implementation) = item {
                Some(implementation.items.clone())
            } else {
                None
            }
        })
        .flatten()
        .map(|member| {
            let ImplItem::Fn(mut function) = member else {
                panic!("methods alone");
            };
            let docs = crate::docs::Docs::lines(&function.attrs)
                .unwrap()
                .join("\n\n");
            function
                .attrs
                .retain(|attribute| !attribute.path().is_ident("doc"));
            (
                function.sig.ident.to_string(),
                docs,
                function.to_token_stream().to_string(),
            )
        })
        .collect()
}

/// The code of `method` among `methods`.
fn code<'methods>(methods: &'methods [(String, String, String)], method: &str) -> &'methods str {
    &methods
        .iter()
        .find(|(name, _, _)| name == method)
        .unwrap_or_else(|| panic!("no method {method}"))
        .2
}

/// The documentation of `method` among `methods`.
fn docs<'methods>(methods: &'methods [(String, String, String)], method: &str) -> &'methods str {
    &methods
        .iter()
        .find(|(name, _, _)| name == method)
        .unwrap_or_else(|| panic!("no method {method}"))
        .1
}

/// Asserts that every method of `expected` is among `methods`, written as it
/// is there.
fn assert_written(methods: &[(String, String, String)], expected: &[TokenStream]) {
    for method in expected {
        let function: syn::ImplItemFn = syn::parse2(method.clone()).unwrap();
        assert_eq!(
            code(methods, &function.sig.ident.to_string()),
            function.to_token_stream().to_string()
        );
    }
}

/// The names of `methods`, in order.
fn names(methods: &[(String, String, String)]) -> Vec<&str> {
    methods.iter().map(|(name, _, _)| name.as_str()).collect()
}

/// A record a call takes is mutable: its constructor comes first, then a
/// getter and a setter of every field in order, and a `clear_` method of
/// every optional one; an expense has its extra methods last.
#[test]
fn a_taken_record_has_every_accessor_in_field_order() {
    let named = named();
    let expense = &named["EranosExpense"];
    assert!(expense.to_string().contains("# [diplomat :: opaque_mut]"));
    let methods = methods(expense);
    assert_eq!(
        names(&methods),
        [
            "create",
            "id",
            "set_id",
            "clear_id",
            "cost",
            "set_cost",
            "count",
            "set_count",
            "date",
            "set_date",
            "distribution",
            "set_distribution",
            "name",
            "set_name",
            "tip",
            "set_tip",
            "fallback",
            "set_fallback",
            "clear_fallback",
            "items",
            "set_items",
            "limit",
            "set_limit",
            "clear_limit",
            "members",
            "set_members",
            "split",
            "set_split",
            "clear_split",
            "time",
            "set_time",
            "clear_time",
            "tip_amount",
            "total",
            "participants",
        ]
    );
    assert_eq!(
        docs(&methods, "create"),
        "An expense of its required fields; every other field starts absent, empty, or at its default.\n\n# Errors\n\n[`EranosError`] of the kind `InvalidText` for text that is not UTF-8."
    );
    assert_eq!(
        docs(&methods, "id"),
        "The identifier. Writes it, if present; absent otherwise."
    );
    assert_eq!(docs(&methods, "set_distribution"), "Sets how it is shared.");
    assert_eq!(docs(&methods, "clear_time"), "Clears the time.");
    assert_eq!(
        docs(&methods, "set_name"),
        "Sets the name.\n\n# Errors\n\n[`EranosError`] of the kind `InvalidText` for text that is not UTF-8."
    );
}

/// A taken record's constructor takes every required field in order, text
/// checked, and starts every other at its default.
#[test]
fn a_taken_records_constructor_takes_its_required_fields() {
    let named = named();
    assert_written(
        &methods(&named["EranosExpense"]),
        &[quote! {
            pub fn create(
                cost: &EranosCost,
                count: i32,
                date: EranosCalendarDate,
                distribution: EranosTipDistribution,
                name: &DiplomatStr,
                tip: &EranosTip
            ) -> Result<Box<Self>, Box<EranosError>> {
                Ok(Box::new(Self(eranos_bindings::Expense {
                    id: None,
                    cost: cost.0.clone(),
                    count,
                    date: date.into(),
                    distribution: distribution.into(),
                    name: crate::text(name)?,
                    tip: tip.0.clone(),
                    fallback: None,
                    items: Vec::new(),
                    limit: None,
                    members: Vec::new(),
                    split: None,
                    time: None
                })))
            }
        }],
    );
}

/// A taken record writes its text into a `DiplomatWrite` and checks the text
/// it is given, absent text answered and cleared.
#[test]
fn a_taken_record_writes_and_checks_its_text() {
    let named = named();
    assert_written(
        &methods(&named["EranosExpense"]),
        &[
            quote! {
                pub fn id(&self, write: &mut DiplomatWrite) -> Option<()> {
                    crate::write_optional(write, self.0.id.as_deref())
                }
            },
            quote! {
                pub fn set_id(&mut self, id: &DiplomatStr) -> Result<(), Box<EranosError>> {
                    self.0.id = Some(crate::text(id)?);
                    Ok(())
                }
            },
            quote! {
                pub fn clear_id(&mut self) {
                    self.0.id = None;
                }
            },
            quote! {
                pub fn name(&self, write: &mut DiplomatWrite) {
                    crate::write_text(write, &self.0.name);
                }
            },
            quote! {
                pub fn set_name(&mut self, name: &DiplomatStr) -> Result<(), Box<EranosError>> {
                    self.0.name = crate::text(name)?;
                    Ok(())
                }
            },
        ],
    );
}

/// A taken record hands an integer over as it is, and a date, a time, or a
/// C-like enum converted, each set in a constant function where clippy asks,
/// and cleared in one when what it drops frees nothing.
#[test]
fn a_taken_record_copies_and_converts_its_values() {
    let named = named();
    assert_written(
        &methods(&named["EranosExpense"]),
        &[
            quote! {
                #[must_use]
                pub const fn count(&self) -> i32 {
                    self.0.count
                }
            },
            quote! {
                pub const fn set_count(&mut self, count: i32) {
                    self.0.count = count;
                }
            },
            quote! {
                #[must_use]
                pub fn date(&self) -> EranosCalendarDate {
                    self.0.date.into()
                }
            },
            quote! {
                pub fn set_date(&mut self, date: EranosCalendarDate) {
                    self.0.date = date.into();
                }
            },
            quote! {
                #[must_use]
                pub fn fallback(&self) -> Option<EranosTipDistribution> {
                    self.0.fallback.map(Into::into)
                }
            },
            quote! {
                pub fn set_fallback(&mut self, fallback: EranosTipDistribution) {
                    self.0.fallback = Some(fallback.into());
                }
            },
            quote! {
                pub const fn clear_fallback(&mut self) {
                    self.0.fallback = None;
                }
            },
            quote! {
                #[must_use]
                pub const fn limit(&self) -> Option<i64> {
                    self.0.limit
                }
            },
            quote! {
                pub const fn set_limit(&mut self, limit: i64) {
                    self.0.limit = Some(limit);
                }
            },
            quote! {
                pub const fn clear_limit(&mut self) {
                    self.0.limit = None;
                }
            },
            quote! {
                #[must_use]
                pub fn time(&self) -> Option<EranosTimeOfDay> {
                    self.0.time.map(Into::into)
                }
            },
            quote! {
                pub fn set_time(&mut self, time: EranosTimeOfDay) {
                    self.0.time = Some(time.into());
                }
            },
            quote! {
                pub const fn clear_time(&mut self) {
                    self.0.time = None;
                }
            },
        ],
    );
}

/// A taken record hands a record or a list over as a copy in an opaque
/// object, and copies one it is given into its own allocation where it
/// has one.
#[test]
fn a_taken_record_copies_its_records_and_lists() {
    let named = named();
    assert_written(
        &methods(&named["EranosExpense"]),
        &[
            quote! {
                #[must_use]
                pub fn cost(&self) -> Box<EranosCost> {
                    Box::new(EranosCost(self.0.cost.clone()))
                }
            },
            quote! {
                pub fn set_cost(&mut self, cost: &EranosCost) {
                    self.0.cost.clone_from(&cost.0);
                }
            },
            quote! {
                #[must_use]
                pub fn items(&self) -> Box<EranosItemList> {
                    Box::new(EranosItemList(self.0.items.clone()))
                }
            },
            quote! {
                pub fn set_items(&mut self, items: &EranosItemList) {
                    self.0.items.clone_from(&items.0);
                }
            },
            quote! {
                #[must_use]
                pub fn members(&self) -> Box<EranosTextList> {
                    Box::new(EranosTextList(self.0.members.clone()))
                }
            },
            quote! {
                #[must_use]
                pub fn split(&self) -> Option<Box<EranosSplit>> {
                    self.0.split.clone().map(|value| Box::new(EranosSplit(value)))
                }
            },
            quote! {
                pub fn set_split(&mut self, split: &EranosSplit) {
                    self.0.split = Some(split.0.clone());
                }
            },
            quote! {
                pub fn clear_split(&mut self) {
                    self.0.split = None;
                }
            },
        ],
    );
}

/// A constructor that takes no text cannot fail and is `#[must_use]`, and a
/// field with a default of its own starts at it.
#[test]
fn a_constructor_without_text_cannot_fail_and_a_default_holds() {
    let named = named();
    let tip = methods(&named["EranosTip"]);
    assert_eq!(
        code(&tip, "create"),
        syn::parse2::<syn::ImplItemFn>(quote! {
            #[must_use]
            pub fn create(distribution: EranosTipDistribution, value: &EranosMoney) -> Box<Self> {
                Box::new(Self(eranos_bindings::Tip {
                    distribution: distribution.into(),
                    value: value.0.clone()
                }))
            }
        })
        .unwrap()
        .to_token_stream()
        .to_string()
    );
    let item = methods(&named["EranosItem"]);
    assert!(code(&item, "create").contains("quantity : 1"));
    assert!(!code(&item, "create").contains("quantity : &"));
}

/// A record that only comes back is read-only, a getter of every field.
#[test]
fn an_answered_record_has_getters_alone() {
    let named = named();
    let summary = &named["EranosSummary"];
    assert!(summary.to_string().contains("# [diplomat :: opaque]"));
    assert_eq!(names(&methods(summary)), ["balances", "total"]);
}

/// An enum with data has a constructor per variant, texts checked and a
/// unit variant without braces, then `kind`, then a getter per field name,
/// by name, and its extra methods last.
#[test]
fn an_enum_with_data_has_constructors_a_kind_and_getters() {
    let named = named();
    let balance = methods(&named["EranosBalance"]);
    assert_eq!(
        names(&balance),
        [
            "create_gets_back",
            "create_owes",
            "create_settled",
            "create_waiting",
            "kind",
            "amount",
            "count",
            "distribution",
            "note",
            "since",
            "format",
        ]
    );
    assert_written(
        &balance,
        &[
            quote! {
                pub fn create_gets_back(
                    amount: &EranosMoney,
                    note: &DiplomatStr,
                    since: EranosCalendarDate
                ) -> Result<Box<Self>, Box<EranosError>> {
                    Ok(Box::new(Self(eranos_bindings::Balance::GetsBack {
                        amount: amount.0.clone(),
                        note: crate::text(note)?,
                        since: since.into()
                    })))
                }
            },
            quote! {
                #[must_use]
                pub fn create_owes(count: i32, distribution: EranosTipDistribution, amount: &EranosMoney) -> Box<Self> {
                    Box::new(Self(eranos_bindings::Balance::Owes {
                        count,
                        distribution: distribution.into(),
                        amount: amount.0.clone()
                    }))
                }
            },
            quote! {
                #[must_use]
                pub fn create_settled() -> Box<Self> {
                    Box::new(Self(eranos_bindings::Balance::Settled))
                }
            },
            quote! {
                #[must_use]
                pub const fn kind(&self) -> EranosBalanceKind {
                    match &self.0 {
                        eranos_bindings::Balance::GetsBack { .. } => EranosBalanceKind::GetsBack,
                        eranos_bindings::Balance::Owes { .. } => EranosBalanceKind::Owes,
                        eranos_bindings::Balance::Settled => EranosBalanceKind::Settled,
                        eranos_bindings::Balance::Waiting { .. } => EranosBalanceKind::Waiting
                    }
                }
            },
        ],
    );
    assert_eq!(docs(&balance, "kind"), "Which kind of balance it is.");
}

/// A getter of a field that some variants hold is absent in the others and
/// says which hold it, each variant's documentation when they document it
/// apart.
#[test]
fn a_getter_is_absent_in_the_variants_without_its_field() {
    let named = named();
    let balance = methods(&named["EranosBalance"]);
    assert_written(
        &balance,
        &[
            quote! {
                #[must_use]
                pub fn amount(&self) -> Option<Box<EranosMoney>> {
                    match &self.0 {
                        eranos_bindings::Balance::GetsBack { amount, .. }
                        | eranos_bindings::Balance::Owes { amount, .. }
                        | eranos_bindings::Balance::Waiting { amount, .. } => Some(Box::new(EranosMoney(amount.clone()))),
                        eranos_bindings::Balance::Settled => None,
                    }
                }
            },
            quote! {
                #[must_use]
                pub const fn count(&self) -> Option<i32> {
                    match &self.0 {
                        eranos_bindings::Balance::Owes { count, .. } => Some(*count),
                        eranos_bindings::Balance::GetsBack { .. }
                        | eranos_bindings::Balance::Settled
                        | eranos_bindings::Balance::Waiting { .. } => None,
                    }
                }
            },
            quote! {
                #[must_use]
                pub fn distribution(&self) -> Option<EranosTipDistribution> {
                    match &self.0 {
                        eranos_bindings::Balance::Owes { distribution, .. } => Some((*distribution).into()),
                        eranos_bindings::Balance::GetsBack { .. }
                        | eranos_bindings::Balance::Settled
                        | eranos_bindings::Balance::Waiting { .. } => None,
                    }
                }
            },
            quote! {
                pub fn note(&self, write: &mut DiplomatWrite) -> Option<()> {
                    match &self.0 {
                        eranos_bindings::Balance::GetsBack { note, .. }
                        | eranos_bindings::Balance::Waiting { note, .. } => crate::write_optional(write, Some(note)),
                        eranos_bindings::Balance::Owes { .. } | eranos_bindings::Balance::Settled => None,
                    }
                }
            },
        ],
    );
    assert_eq!(
        docs(&balance, "amount"),
        "In `GetsBack`, what the member gets back; in `Owes`, what the member owes; in `Waiting`, what the member waits for. Present in the kinds `GetsBack`, `Owes`, and `Waiting` alone."
    );
    assert_eq!(
        docs(&balance, "note"),
        "A note. Present in the kinds `GetsBack` and `Waiting` alone. Writes it, if present; absent otherwise."
    );
    assert_eq!(
        docs(&balance, "since"),
        "Since when. Present in the kind `GetsBack` alone."
    );
}

/// A getter of a field that every variant holds is always there: text
/// written, an integer as it is in a constant function, a date converted,
/// and a record copied.
#[test]
fn a_getter_of_a_field_every_variant_holds_is_always_there() {
    let named = named();
    let cost = methods(&named["EranosCost"]);
    assert_written(
        &cost,
        &[
            quote! {
                #[must_use]
                pub fn amount(&self) -> Box<EranosMoney> {
                    match &self.0 {
                        eranos_bindings::Cost::Converted { amount, .. }
                        | eranos_bindings::Cost::Local { amount, .. } => Box::new(EranosMoney(amount.clone())),
                    }
                }
            },
            quote! {
                #[must_use]
                pub fn date(&self) -> EranosCalendarDate {
                    match &self.0 {
                        eranos_bindings::Cost::Converted { date, .. }
                        | eranos_bindings::Cost::Local { date, .. } => (*date).into(),
                    }
                }
            },
            quote! {
                #[must_use]
                pub const fn places(&self) -> i32 {
                    match &self.0 {
                        eranos_bindings::Cost::Converted { places, .. }
                        | eranos_bindings::Cost::Local { places, .. } => *places,
                    }
                }
            },
            quote! {
                pub fn rate(&self, write: &mut DiplomatWrite) {
                    match &self.0 {
                        eranos_bindings::Cost::Converted { rate, .. }
                        | eranos_bindings::Cost::Local { rate, .. } => crate::write_text(write, rate),
                    }
                }
            },
        ],
    );
    assert_eq!(docs(&cost, "rate"), "The rate.");
}

/// A field name the variants hold with different types gives a getter per
/// variant, named after it, and the kinds are a C-like enum of the
/// variants.
#[test]
fn a_field_name_of_two_types_gives_a_getter_per_variant() {
    let named = named();
    let split = methods(&named["EranosSplit"]);
    assert_eq!(
        names(&split),
        [
            "create_amounts",
            "create_equal",
            "create_itemised",
            "kind",
            "amounts_shares",
            "itemised_shares",
            "members",
            "participants",
        ]
    );
    assert!(code(&split, "amounts_shares").contains("EranosMoneyList (shares . clone ())"));
    let kinds = parsed(&named["EranosBalanceKind"]);
    assert_eq!(
        kinds.first().unwrap().to_token_stream().to_string(),
        quote! {
            #[doc = "The kinds of balance."]
            #[derive(Debug)]
            pub enum EranosBalanceKind {
                #[doc = "The others owe the member."]
                GetsBack,
                #[doc = "The member owes the others."]
                Owes,
                #[doc = "The member owes nothing."]
                Settled,
                #[doc = "The member waits."]
                Waiting
            }
        }
        .to_string()
    );
}

/// An enum without data is a C-like enum that Diplomat converts to and from
/// the core's.
#[test]
fn an_enum_without_data_is_a_c_like_enum() {
    let named = named();
    assert_eq!(
        named["EranosTipDistribution"].to_string(),
        quote! {
            #[doc = "How a tip is shared."]
            #[derive(Debug)]
            #[diplomat::enum_convert(eranos_bindings::TipDistribution)]
            pub enum EranosTipDistribution {
                #[doc = "The same for everyone."]
                Equal,
                #[doc = "In proportion to each share."]
                Proportional
            }
        }
        .to_string()
    );
}

/// A list a call takes is mutable, with `create` and `push`; one that only
/// comes back is not; and the list of currencies lists every one.
#[test]
fn a_taken_list_is_mutable_and_an_answered_one_is_not() {
    let named = named();
    let items = &named["EranosItemList"];
    assert!(items.to_string().contains("# [diplomat :: opaque_mut]"));
    assert_eq!(
        names(&methods(items)),
        ["create", "push", "len", "is_empty", "get"]
    );
    let expenses = &named["EranosExpenseList"];
    assert!(expenses.to_string().contains("# [diplomat :: opaque]"));
    assert_eq!(names(&methods(expenses)), ["len", "is_empty", "get"]);
    assert_eq!(
        names(&methods(&named["EranosCurrencyDetailsList"])),
        ["len", "is_empty", "get", "all"]
    );
    assert_eq!(
        names(&methods(&named["EranosCurrencyDetails"])),
        ["code", "of"]
    );
}

/// `items` writes what a plan names alone, and refuses a record or an enum
/// of a type the bridge cannot carry, or an enum's optional field, even when
/// a plan names it without having read it.
#[test]
fn items_write_what_the_plan_names_and_refuse_what_they_cannot_carry() {
    let core = Core::parse(&sources()).unwrap();
    assert!(items(&core, &Plan::default()).unwrap().is_empty());
    let broken = Core::parse(&[
        "
        /// A record.
        #[mirrored]
        pub struct Record {
            /// A float.
            pub value: f64,
        }

        /// An enum.
        #[mirrored]
        pub enum Choice {
            /// One.
            One {
                /// A float.
                value: f64,
            },
        }

        /// An optional one.
        #[mirrored]
        pub enum Optional {
            /// One.
            One {
                /// Some text.
                value: Option<String>,
            },
        }
        "
        .to_owned(),
        CALLS.to_owned(),
    ])
    .unwrap();
    for name in ["Record", "Choice", "Optional"] {
        let plan = Plan {
            objects: [(name.to_owned(), true)].into(),
            ..Plan::default()
        };
        assert!(items(&broken, &plan).is_err(), "{name}");
    }
    assert!(RECORDS.contains("pub struct Handle"));
}
