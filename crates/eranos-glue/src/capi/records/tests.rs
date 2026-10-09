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

/// A method of an `impl`, as it prints.
struct Method {
    /// Its code without its documentation.
    code: String,
    /// Its documentation, its paragraphs a blank line apart.
    docs: String,
    /// Its name.
    name: String,
}

/// The methods of the `impl` among `tokens`, in order.
fn methods(tokens: &TokenStream) -> Vec<Method> {
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
            Method {
                code: function.to_token_stream().to_string(),
                docs,
                name: function.sig.ident.to_string(),
            }
        })
        .collect()
}

/// The code of `method` among `methods`.
fn code<'methods>(methods: &'methods [Method], method: &str) -> &'methods str {
    &methods
        .iter()
        .find(|found| found.name == method)
        .unwrap_or_else(|| panic!("no method {method}"))
        .code
}

/// The documentation of `method` among `methods`.
fn docs<'methods>(methods: &'methods [Method], method: &str) -> &'methods str {
    &methods
        .iter()
        .find(|found| found.name == method)
        .unwrap_or_else(|| panic!("no method {method}"))
        .docs
}

/// Asserts that every method of `expected` is among `methods`, written as it
/// is there.
fn assert_written(methods: &[Method], expected: &[TokenStream]) {
    for method in expected {
        let function: syn::ImplItemFn = syn::parse2(method.clone()).unwrap();
        assert_eq!(
            code(methods, &function.sig.ident.to_string()),
            function.to_token_stream().to_string()
        );
    }
}

/// The names of `methods`, in order.
fn names(methods: &[Method]) -> Vec<&str> {
    methods.iter().map(|method| method.name.as_str()).collect()
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
        "A new `EranosExpense` from its required fields; every other field starts absent, empty, or at its default.\n\n# Errors\n\n[`EranosError`] of the kind `InvalidText` for text that is not UTF-8."
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
/// checked, and starts every other at its default: an expense's tip at what
/// the core's no-tip helper returns.
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
                name: &DiplomatStr
            ) -> Result<Box<Self>, Box<EranosError>> {
                Ok(Box::new(Self(eranos_bindings::Expense {
                    id: None,
                    cost: cost.0.clone(),
                    count,
                    date: date.into(),
                    distribution: distribution.into(),
                    name: crate::text(name)?,
                    tip: eranos_bindings::tip_rule_none(),
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
                pub fn items(&self) -> Box<EranosExpenseItemList> {
                    Box::new(EranosExpenseItemList(self.0.items.clone()))
                }
            },
            quote! {
                pub fn set_items(&mut self, items: &EranosExpenseItemList) {
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

/// A constructor that takes no text cannot fail and is `#[must_use]`, a
/// field with a default of its own starts at it, the no-tip rule has a
/// constructor of its own, and an item reads its line total before its
/// discount, its discount's amount, and its line total from the core.
#[test]
fn a_constructor_without_text_cannot_fail_and_a_default_holds() {
    let named = named();
    let tip = methods(&named["EranosTipRule"]);
    assert_eq!(
        names(&tip),
        [
            "create",
            "distribution",
            "set_distribution",
            "value",
            "set_value",
            "none"
        ]
    );
    assert_written(
        &tip,
        &[quote! {
            #[must_use]
            pub fn none() -> Box<Self> {
                Box::new(Self(eranos_bindings::tip_rule_none()))
            }
        }],
    );
    assert_eq!(
        code(&tip, "create"),
        syn::parse2::<syn::ImplItemFn>(quote! {
            #[must_use]
            pub fn create(distribution: EranosTipDistribution, value: &EranosMoney) -> Box<Self> {
                Box::new(Self(eranos_bindings::TipRule {
                    distribution: distribution.into(),
                    value: value.0.clone()
                }))
            }
        })
        .unwrap()
        .to_token_stream()
        .to_string()
    );
    let item = methods(&named["EranosExpenseItem"]);
    assert!(code(&item, "create").contains("quantity : 10000"));
    assert!(!code(&item, "create").contains("quantity : &"));
    // An item's totals come from the core, after the accessors, in the
    // library's order.
    let item_names = names(&item);
    assert_eq!(
        item_names.get(item_names.len().saturating_sub(3)..),
        Some(
            &[
                "line_total_before_discount",
                "discount_amount",
                "line_total"
            ][..]
        )
    );
    assert_written(
        &item,
        &[
            quote! {
                pub fn line_total_before_discount(&self) -> Result<Box<EranosMoney>, Box<EranosError>> {
                    eranos_bindings::expense_item_line_total_before_discount(&self.0)
                        .map(|money| Box::new(EranosMoney(money)))
                        .map_err(crate::failed)
                }
            },
            quote! {
                pub fn discount_amount(&self) -> Result<Box<EranosMoney>, Box<EranosError>> {
                    eranos_bindings::expense_item_discount_amount(&self.0)
                        .map(|money| Box::new(EranosMoney(money)))
                        .map_err(crate::failed)
                }
            },
            quote! {
                pub fn line_total(&self) -> Result<Box<EranosMoney>, Box<EranosError>> {
                    eranos_bindings::expense_item_line_total(&self.0)
                        .map(|money| Box::new(EranosMoney(money)))
                        .map_err(crate::failed)
                }
            },
        ],
    );
}

/// A record that only comes back is read-only: a getter of every field, then
/// the methods its template adds, and no constructor or setter.
#[test]
fn an_answered_record_has_no_constructor_or_setter() {
    let named = named();
    let summary = &named["EranosSummary"];
    assert!(summary.to_string().contains("# [diplomat :: opaque]"));
    assert_eq!(
        names(&methods(summary)),
        [
            "balances",
            "total",
            "member",
            "category_total",
            "member_category_total"
        ]
    );
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
    assert_eq!(
        docs(&balance, "kind"),
        "Which kind of `EranosBalance` it is."
    );
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
            #[doc = "The kinds of `EranosBalance`."]
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
    let items = &named["EranosExpenseItemList"];
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
        ["code", "places", "of"]
    );
}

/// A record the library parses is read-only, though a taken record holds
/// it: C makes one only through `parse`, which the core's helper fills, and
/// reads every field. A writable one would let C pair a value with text the
/// library did not derive from it.
#[test]
fn a_parsed_record_is_read_only_and_made_by_parse() {
    let named = named();
    let phone = &named["EranosPhoneNumber"];
    assert!(phone.to_string().contains("# [diplomat :: opaque]"));
    let number = methods(phone);
    assert_eq!(names(&number), ["parse", "e164", "international"]);
    assert_written(
        &number,
        &[quote! {
            pub fn parse(text: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
                eranos_bindings::phone_number_parse(&crate::text(text)?)
                    .map(|value| Box::new(Self(value)))
                    .map_err(crate::failed)
            }
        }],
    );
    assert_eq!(
        docs(&number, "parse"),
        "The phone number written `text`, in E.164 form and in international form.\n\n# Errors\n\n[`EranosError`] of the kind `Contact` for text that is not a phone number, and of the kind `InvalidText` for text that is not UTF-8."
    );
    let iban = methods(&named["EranosIban"]);
    assert_eq!(names(&iban), ["parse", "iban", "country_code"]);
    assert!(code(&iban, "parse").contains("eranos_bindings :: iban_parse"));
}

/// A field that holds a record the library parses is set from the value's
/// text, which the library parses, filling the rest, and refuses as its own
/// helper does, and is read as a copy.
#[test]
fn a_field_of_a_parsed_record_is_set_from_its_text() {
    let named = named();
    let item = methods(&named["EranosExpenseItem"]);
    assert_written(
        &item,
        &[
            quote! {
                #[must_use]
                pub fn phone(&self) -> Option<Box<EranosPhoneNumber>> {
                    self.0.phone.clone().map(|value| Box::new(EranosPhoneNumber(value)))
                }
            },
            quote! {
                pub fn set_phone(&mut self, phone: &DiplomatStr) -> Result<(), Box<EranosError>> {
                    self.0.phone = Some(
                        eranos_bindings::phone_number_parse(&crate::text(phone)?)
                            .map_err(crate::failed)?
                    );
                    Ok(())
                }
            },
            quote! {
                pub fn clear_phone(&mut self) {
                    self.0.phone = None;
                }
            },
        ],
    );
    assert_eq!(
        docs(&item, "set_phone"),
        "Sets whom to call.\n\nThe library parses the number, which it writes in E.164 form, and fills its international form.\n\n# Errors\n\n[`EranosError`] of the kind `Contact` for text that is not a phone number, and of the kind `InvalidText` for text that is not UTF-8."
    );
}

/// A variant named after a record the library parses, which holds its
/// fields, is made from the value's text alone, which the core's helper
/// parses into every field; its fields read as any variant's do.
#[test]
fn a_variant_named_after_a_parsed_record_is_made_from_its_text() {
    let named = named();
    let account = methods(&named["EranosAccount"]);
    assert_eq!(
        names(&account),
        [
            "create_iban",
            "create_username",
            "kind",
            "country_code",
            "iban",
            "username"
        ]
    );
    assert_written(
        &account,
        &[quote! {
            pub fn create_iban(iban: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
                eranos_bindings::iban_parse(&crate::text(iban)?)
                    .map(|eranos_bindings::Iban { iban, country_code }| {
                        Box::new(Self(eranos_bindings::Account::Iban { iban, country_code }))
                    })
                    .map_err(crate::failed)
            }
        }],
    );
    assert_eq!(
        docs(&account, "create_iban"),
        "A bank account.\n\nThe library parses the IBAN, which it writes without spaces and in capitals, and fills its country code.\n\n# Errors\n\n[`EranosError`] of the kind `PaymentAccount` for text that is not an IBAN, and of the kind `InvalidText` for text that is not UTF-8."
    );
}

/// `items` writes what a plan names alone, and refuses a record or an enum
/// of a type the bridge cannot carry, an enum's optional field, or a variant
/// named after a record the library parses that does not hold its fields,
/// even when a plan names it without having read it.
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

        /// An IBAN.
        #[mirrored]
        pub struct Iban {
            /// The IBAN.
            pub iban: String,
            /// The country code.
            pub country_code: String,
        }

        /// An account named after the IBAN without its country code.
        #[mirrored]
        pub enum Account {
            /// A bank account.
            Iban {
                /// The IBAN.
                iban: String,
            },
        }
        "
        .to_owned(),
        CALLS.to_owned(),
    ])
    .unwrap();
    for name in ["Record", "Choice", "Optional", "Account"] {
        let plan = Plan {
            objects: [(name.to_owned(), true)].into(),
            ..Plan::default()
        };
        assert!(items(&broken, &plan).is_err(), "{name}");
    }
    assert!(RECORDS.contains("pub struct Handle"));
}
