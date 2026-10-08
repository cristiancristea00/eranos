//! How each type of the binding core crosses the C bridge, and which of the
//! core's records, lists, and enums the bridge writes: those the group
//! calls, the group's constructor, and the templates reach. A record a call
//! takes, or a list one takes, is mutable, with a constructor and setters;
//! one that only comes back is read-only.

use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{GenericArgument, PathArguments, Type};

use crate::calls::Kind;
use crate::capi::items::Core;

/// The integers the core's records hold, which cross as themselves: the
/// signed ones of every count and amount, and the `u8` of a currency's
/// decimal places.
const INTEGERS: [&str; 3] = ["i32", "i64", "u8"];

/// The core's plain structs, which cross by value as the C glue's
/// hand-written structs of the same fields.
const PLAIN: [&str; 2] = ["CalendarDate", "TimeOfDay"];

/// The core's records whose opaque objects the C glue writes by hand, with
/// methods of their own.
const WRITTEN: [&str; 1] = ["Money"];

/// The records the group's constructor takes, which no call does.
const CONSTRUCTED: [&str; 1] = ["NewExpenseGroup"];

/// The records whose lists the templates answer with: the list of every
/// currency.
const LISTED: [&str; 1] = ["CurrencyDetails"];

/// How a value crosses the bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Crossing {
    /// An enum without data, as a C-like enum converted with `into`.
    Choice(Ident),
    /// An integer, as itself: an `i32`, an `i64`, or a `u8`.
    Integer(Ident),
    /// A list of records, as an opaque list of copies.
    List(Ident),
    /// A record or an enum with data, as an opaque object holding a copy.
    Object(Ident),
    /// A date or a time of day, as a plain struct converted with `into`.
    Plain(Ident),
    /// Text, taken as bytes and written into a `DiplomatWrite`.
    Text,
    /// A list of text, as the C glue's list of text.
    TextList,
}

impl Crossing {
    /// How the core's type `written` crosses.
    ///
    /// # Errors
    ///
    /// For a type that is not text, an `i32`, an `i64`, or a `u8`, a
    /// mirrored record or enum, or a list of text or of records.
    pub(crate) fn of(written: &Type, core: &Core) -> syn::Result<Self> {
        if let Some(element) = argument(written, "Vec") {
            return match Self::of(element, core)? {
                Self::Object(name) => Ok(Self::List(name)),
                Self::Text => Ok(Self::TextList),
                Self::Choice(_)
                | Self::Integer(_)
                | Self::List(_)
                | Self::Plain(_)
                | Self::TextList => Err(unsupported(written)),
            };
        }
        let Type::Path(path) = written else {
            return Err(unsupported(written));
        };
        let name = path.path.get_ident().ok_or_else(|| unsupported(written))?;
        let text = name.to_string();
        if "String" == text {
            Ok(Self::Text)
        } else if INTEGERS.contains(&text.as_str()) {
            Ok(Self::Integer(name.clone()))
        } else if PLAIN.contains(&text.as_str()) {
            Ok(Self::Plain(name.clone()))
        } else {
            let item = core.item(&text).ok_or_else(|| unsupported(written))?;
            if item.is_plain_enum() {
                Ok(Self::Choice(name.clone()))
            } else {
                Ok(Self::Object(name.clone()))
            }
        }
    }

    /// The name of the bridge's opaque type that holds the value: a
    /// record's, a list's, or the list of text's; none for a value that
    /// crosses by value or as text.
    pub(crate) fn opaque(&self) -> Option<Ident> {
        match self {
            Self::List(element) => Some(super::list(element)),
            Self::Object(name) => Some(super::eranos(name)),
            Self::TextList => Some(format_ident!("EranosTextList")),
            Self::Choice(_) | Self::Integer(_) | Self::Plain(_) | Self::Text => None,
        }
    }

    /// The bridge's parameter `name` for the value: text as bytes, a record
    /// or a list by reference, and anything else by value.
    pub(crate) fn parameter(&self, name: &Ident) -> TokenStream {
        match self {
            Self::Choice(value) | Self::Plain(value) => {
                let bridge = super::eranos(value);
                quote!(#name: #bridge)
            }
            Self::Integer(integer) => quote!(#name: #integer),
            Self::List(_) | Self::Object(_) | Self::TextList => {
                let opaque = self.opaque();
                quote!(#name: &#opaque)
            }
            Self::Text => quote!(#name: &DiplomatStr),
        }
    }

    /// The core's value of the bridge's parameter `name`, owned: text
    /// checked, a value converted, and a record or a list copied.
    pub(crate) fn owned(&self, name: &Ident) -> TokenStream {
        match self {
            Self::Choice(_) | Self::Plain(_) => quote!(#name.into()),
            Self::Integer(_) => quote!(#name),
            Self::List(_) | Self::Object(_) | Self::TextList => quote!(#name.0.clone()),
            Self::Text => quote!(crate::text(#name)?),
        }
    }
}

/// A value of a field: how it crosses, and whether it may be absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Value {
    /// How it crosses.
    pub(crate) crossing: Crossing,
    /// Whether it is an `Option`.
    pub(crate) optional: bool,
}

impl Value {
    /// How the core's type `written` crosses, and whether it may be absent.
    ///
    /// # Errors
    ///
    /// As [`Crossing::of`], and for an optional list, which the core never
    /// holds, since an empty list says the same.
    pub(crate) fn of(written: &Type, core: &Core) -> syn::Result<Self> {
        let Some(inner) = argument(written, "Option") else {
            return Ok(Self {
                crossing: Crossing::of(written, core)?,
                optional: false,
            });
        };
        let crossing = Crossing::of(inner, core)?;
        if let Crossing::List(_) | Crossing::TextList = crossing {
            return Err(syn::Error::new_spanned(
                written,
                "the C bridge carries no optional list: an empty list says the same",
            ));
        }
        Ok(Self {
            crossing,
            optional: true,
        })
    }
}

/// What a group call answers, which decides how each form hands it over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Answer {
    /// Whether something holds: `bool`.
    Flag,
    /// A record, or none: `Option<T>`.
    Found(Ident),
    /// Whether something holds, or none for what the call does not find:
    /// `Option<bool>`.
    FoundFlag,
    /// A list of records: `Vec<T>`.
    List(Ident),
    /// Nothing: `()`.
    Nothing,
    /// Text, or none: `Option<String>`.
    OptionalText,
    /// A record: `T`.
    Record(Ident),
    /// Text: `String`.
    Text,
}

impl Answer {
    /// What the core's answer `written` is.
    ///
    /// # Errors
    ///
    /// As [`Value::of`], and for an answer that is not nothing, a flag or
    /// none, optional text, a record or none, or a list of records.
    pub(crate) fn of(written: &Type, core: &Core) -> syn::Result<Self> {
        if let Type::Tuple(tuple) = written
            && tuple.elems.is_empty()
        {
            return Ok(Self::Nothing);
        }
        if flag(written) {
            return Ok(Self::Flag);
        }
        if argument(written, "Option").is_some_and(flag) {
            return Ok(Self::FoundFlag);
        }
        let Value { crossing, optional } = Value::of(written, core)?;
        match (crossing, optional) {
            (Crossing::List(element), false) => Ok(Self::List(element)),
            (Crossing::Object(name), false) => Ok(Self::Record(name)),
            (Crossing::Object(name), true) => Ok(Self::Found(name)),
            (Crossing::Text, false) => Ok(Self::Text),
            (Crossing::Text, true) => Ok(Self::OptionalText),
            (
                Crossing::Choice(_)
                | Crossing::Integer(_)
                | Crossing::List(_)
                | Crossing::Plain(_)
                | Crossing::TextList,
                _,
            ) => Err(syn::Error::new_spanned(
                written,
                "a group call answers with nothing, a flag, an optional flag, text, optional text, a record, an optional record, or a list of records",
            )),
        }
    }

    /// What the plan reaches through the answer: its record, or its list.
    fn crossing(&self) -> Option<Crossing> {
        match self {
            Self::Found(name) | Self::Record(name) => Some(Crossing::Object(name.clone())),
            Self::List(element) => Some(Crossing::List(element.clone())),
            Self::Flag | Self::FoundFlag | Self::Nothing | Self::OptionalText | Self::Text => None,
        }
    }
}

/// How a call's parameter crosses: text, optional text, a record, a list, or
/// a value.
///
/// # Errors
///
/// As [`Crossing::of`], and for a record or a list of another kind, and a
/// value that is a record, which a call takes by reference.
pub(crate) fn parameter_crossing(kind: &Kind, core: &Core) -> syn::Result<Crossing> {
    match kind {
        Kind::List(element) => match Crossing::of(element, core)? {
            Crossing::Object(name) => Ok(Crossing::List(name)),
            Crossing::Text => Ok(Crossing::TextList),
            Crossing::Choice(_)
            | Crossing::Integer(_)
            | Crossing::List(_)
            | Crossing::Plain(_)
            | Crossing::TextList => Err(syn::Error::new_spanned(
                element,
                "a group call takes a list of text or of records",
            )),
        },
        Kind::OptionalText | Kind::Text => Ok(Crossing::Text),
        Kind::Record(record) => {
            let crossing = Crossing::of(record, core)?;
            if let Crossing::Object(_) = crossing {
                Ok(crossing)
            } else {
                Err(syn::Error::new_spanned(
                    record,
                    "a group call takes a record by reference, and anything else by value",
                ))
            }
        }
        Kind::Value(value) => {
            let crossing = Crossing::of(value, core)?;
            if let Crossing::Object(_) | Crossing::List(_) | Crossing::TextList = crossing {
                Err(syn::Error::new_spanned(
                    value,
                    "a group call takes a record by reference, and anything else by value",
                ))
            } else {
                Ok(crossing)
            }
        }
    }
}

/// What the bridge writes of the core's records, lists, and enums.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Plan {
    /// The enums without data, each a C-like enum.
    pub(crate) choices: BTreeSet<String>,
    /// The lists of records, by their element's name, each `true` when a
    /// call takes one.
    pub(crate) lists: BTreeMap<String, bool>,
    /// The records and enums with data, each `true` when a call takes one.
    pub(crate) objects: BTreeMap<String, bool>,
}

impl Plan {
    /// Every record, list, and enum that `core`'s calls reach through what
    /// they take and answer, and through the group's constructor and the
    /// templates, but for the hand-written ones.
    ///
    /// # Errors
    ///
    /// For a field, a parameter, or an answer whose type the bridge cannot
    /// carry.
    pub(crate) fn of(core: &Core) -> syn::Result<Self> {
        let mut plan = Self::default();
        for call in &core.calls.calls {
            for parameter in &call.parameters {
                plan.reach(&parameter_crossing(&parameter.kind, core)?, true, core)?;
            }
            if let Some(crossing) = Answer::of(&call.answer, core)?.crossing() {
                plan.reach(&crossing, false, core)?;
            }
        }
        for name in CONSTRUCTED {
            plan.reach(&Crossing::Object(format_ident!("{name}")), true, core)?;
        }
        for element in LISTED {
            plan.reach(&Crossing::List(format_ident!("{element}")), false, core)?;
        }
        Ok(plan)
    }

    /// Adds what `crossing` reaches, taken by a call if `taken`, and what its
    /// fields reach in turn.
    ///
    /// # Errors
    ///
    /// For a field whose type the bridge cannot carry.
    fn reach(&mut self, crossing: &Crossing, taken: bool, core: &Core) -> syn::Result<()> {
        match crossing {
            Crossing::Choice(name) => {
                self.choices.insert(name.to_string());
                Ok(())
            }
            Crossing::List(element) => {
                mark(&mut self.lists, element, taken);
                self.reach(&Crossing::Object(element.clone()), taken, core)
            }
            Crossing::Object(name) => {
                let text = name.to_string();
                if WRITTEN.contains(&text.as_str()) || !mark(&mut self.objects, name, taken) {
                    return Ok(());
                }
                // `Crossing::of` made an object only of an item the core
                // holds, and the constructor and the templates name items
                // of the core.
                let Some(item) = core.item(&text) else {
                    return Err(syn::Error::new_spanned(name, "the core holds no such item"));
                };
                for field in item.fields() {
                    self.reach(&Value::of(&field.written, core)?.crossing, taken, core)?;
                }
                Ok(())
            }
            Crossing::Integer(_) | Crossing::Plain(_) | Crossing::Text | Crossing::TextList => {
                Ok(())
            }
        }
    }
}

/// Marks `name` in `known` as reached, and as taken by a call if `taken`,
/// and returns whether that changed anything, so a record reached again is
/// followed only once more, when a call turns out to take it.
fn mark(known: &mut BTreeMap<String, bool>, name: &Ident, taken: bool) -> bool {
    match known.get_mut(&name.to_string()) {
        Some(entry) if *entry || !taken => false,
        Some(entry) => {
            *entry = true;
            true
        }
        None => {
            known.insert(name.to_string(), taken);
            true
        }
    }
}

/// The type argument of `written` when it is the generic `name` of one
/// argument, such as `Option<T>` or `Vec<T>`.
fn argument<'written>(written: &'written Type, name: &str) -> Option<&'written Type> {
    if let Type::Path(path) = written
        && let Some(segment) = path.path.segments.last()
        && segment.ident == name
        && let PathArguments::AngleBracketed(arguments) = &segment.arguments
        && 1 == arguments.args.len()
        && let Some(GenericArgument::Type(argument)) = arguments.args.first()
    {
        Some(argument)
    } else {
        None
    }
}

/// Whether `written` is `bool`.
fn flag(written: &Type) -> bool {
    "bool" == written.to_token_stream().to_string()
}

/// The failure of a type the bridge cannot carry.
fn unsupported(written: &Type) -> syn::Error {
    syn::Error::new_spanned(
        written,
        "the C bridge carries text, an `i32`, an `i64`, or a `u8`, a mirrored record or enum, and a list of text or of records",
    )
}

#[cfg(test)]
mod tests;
