//! The bridge's records, lists, and enums, as opaque objects over copies of
//! the core's values, since Diplomat carries text and lists only behind
//! them.
//!
//! A record a call takes is mutable: a constructor of its required fields,
//! and a getter and a setter of every field in its order, with a `clear_`
//! method for an optional one. A record that only comes back has its
//! getters alone. A record the library parses has `parse` and its getters:
//! the setter of a field that holds one takes the value's text, which the
//! library parses, and a variant named after one, which holds its fields,
//! is made from that text too. An enum with data has a constructor per
//! variant, `kind`, which answers a C-like enum of its variants, and a
//! getter per field name, absent in the variants without it; an enum
//! without data is a C-like enum. A list has `len`, `is_empty`, and `get`,
//! and, when a call takes one, `create` and `push`.

use std::collections::BTreeSet;

use proc_macro2::{Ident, Literal, TokenStream};
use quote::{ToTokens, format_ident, quote};

use crate::capi::crossing::{Crossing, Parsed, Plan, Value, parsed};
use crate::capi::items::{Core, Field, Mirrored, Shape, Variant};
use crate::capi::{
    INVALID_TEXT, Named, article, capitalised, doc, doc_with_errors, eranos, inside, list, plural,
    words,
};

/// A field of a record that starts at a value of its own when the record is
/// created without it.
struct FieldDefault {
    /// The field's name.
    field: &'static str,
    /// The record's name.
    record: &'static str,
    /// What the field starts at.
    value: Start,
}

/// What a field that a record's constructor leaves out starts at.
enum Start {
    /// The integer, which fits the widest integer field a record has, an
    /// `i64`.
    Integer(i64),
    /// What the core's value helper of this name returns.
    Helper(&'static str),
}

impl Start {
    /// The expression of the value.
    fn tokens(&self) -> TokenStream {
        match self {
            Self::Integer(value) => Literal::i64_unsuffixed(*value).into_token_stream(),
            Self::Helper(name) => {
                let helper = format_ident!("{name}");
                quote!(eranos_bindings::#helper())
            }
        }
    }
}

/// The fields of a record that start at a value of their own when the
/// record is created without them: an expense has no tip and is in the
/// built-in category, and a line of a bill is of one unit, 10 000
/// ten-thousandths.
const DEFAULTS: [FieldDefault; 3] = [
    FieldDefault {
        field: "tip",
        record: "Expense",
        value: Start::Helper("tip_rule_none"),
    },
    FieldDefault {
        field: "category",
        record: "Expense",
        value: Start::Helper("category_id_uncategorised"),
    },
    FieldDefault {
        field: "quantity",
        record: "ExpenseItem",
        value: Start::Integer(10_000),
    },
];

/// A field of a record and how it crosses.
struct Typed<'item> {
    /// The field.
    field: &'item Field,
    /// How it crosses.
    value: &'item Value,
}

/// The items of every record, list, and enum that `plan` names, of the
/// core's `core`.
///
/// # Errors
///
/// For a field whose type the bridge cannot carry, and an optional field of
/// an enum's variant.
pub(crate) fn items(core: &Core, plan: &Plan) -> syn::Result<Vec<Named>> {
    let mut items = Vec::new();
    for item in &core.items {
        let name = item.name.to_string();
        match &item.shape {
            Shape::Choice(variants) => {
                if plan.choices.contains(&name) {
                    items.push(choice(item, variants));
                } else if plan.objects.contains_key(&name) {
                    items.extend(data_enum(item, variants, core)?);
                }
            }
            Shape::Record(fields) => {
                if let Some(&taken) = plan.objects.get(&name) {
                    items.push(record(item, fields, taken, core)?);
                }
            }
        }
    }
    for (element, &taken) in &plan.lists {
        items.push(list_of(&format_ident!("{element}"), taken));
    }
    Ok(items)
}

/// The opaque object of the record `item` of `fields`: mutable, with a
/// constructor and setters, when a call takes it, `taken`.
///
/// # Errors
///
/// For a field whose type the bridge cannot carry.
fn record(item: &Mirrored, fields: &[Field], taken: bool, core: &Core) -> syn::Result<Named> {
    let (name, object) = (&item.name, eranos(&item.name));
    let values = fields
        .iter()
        .map(|field| Value::of(&field.written, core))
        .collect::<syn::Result<Vec<_>>>()?;
    let fields = fields
        .iter()
        .zip(&values)
        .map(|(field, value)| Typed { field, value })
        .collect::<Vec<_>>();
    let (opaque, create) = if taken {
        (quote!(#[diplomat::opaque_mut]), Some(create(item, &fields)))
    } else {
        (quote!(#[diplomat::opaque]), None)
    };
    let parse = parsed(&name.to_string()).map(parse);
    let accessors = fields
        .iter()
        .map(|typed| accessors(typed.field, typed.value, taken));
    let summary = doc(&[&item.doc]);
    let extras = extras(&object);
    Ok(Named {
        name: object.to_string(),
        tokens: quote! {
            #summary
            #opaque
            #[derive(Debug)]
            pub struct #object(pub(crate) eranos_bindings::#name);

            impl #object {
                #create
                #parse
                #(#accessors)*
                #extras
            }
        },
    })
}

/// The `parse` of the record the library parses, `parsed`: the record of
/// the value written `text`, with the text the library derives from it.
fn parse(parsed: &Parsed) -> TokenStream {
    let helper = format_ident!("{}", parsed.helper);
    let documentation = doc_with_errors(parsed.summary, &parsed.errors());
    quote! {
        #documentation
        pub fn parse(text: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
            eranos_bindings::#helper(&crate::text(text)?)
                .map(|value| Box::new(Self(value)))
                .map_err(crate::failed)
        }
    }
}

/// The constructor of the record `item` of `fields`, which takes every
/// required field in order and starts every other one at its default.
fn create(item: &Mirrored, fields: &[Typed<'_>]) -> TokenStream {
    let name = &item.name;
    let required = fields
        .iter()
        .filter(|typed| default(item, typed.field, typed.value).is_none())
        .collect::<Vec<_>>();
    let parameters = required
        .iter()
        .map(|typed| typed.value.crossing.parameter(&typed.field.name));
    let initialisers = fields.iter().map(|typed| {
        let field_name = &typed.field.name;
        default(item, typed.field, typed.value).map_or_else(
            || initialiser(field_name, &typed.value.crossing),
            |default| quote!(#field_name: #default),
        )
    });
    let fallible = required
        .iter()
        .any(|typed| Crossing::Text == typed.value.crossing);
    let words = words(name);
    let possessive = if plural(&words) { "their" } else { "its" };
    let summary = format!(
        "{} of {possessive} required fields; every other field starts absent, empty, or at its default.",
        capitalised(&article(&words))
    );
    constructor(
        &format_ident!("create"),
        &summary,
        parameters,
        &quote!(Box::new(Self(eranos_bindings::#name { #(#initialisers),* }))),
        fallible,
    )
}

/// What `field` of the record `item` starts at when a constructor leaves it
/// out: nothing for an optional field, an empty list, or its default; none
/// for a field the constructor takes.
fn default(item: &Mirrored, field: &Field, value: &Value) -> Option<TokenStream> {
    if value.optional {
        return Some(quote!(None));
    }
    if let Crossing::List(_) | Crossing::TextList = value.crossing {
        return Some(quote!(Vec::new()));
    }
    DEFAULTS
        .iter()
        .find(|default| item.name == default.record && field.name == default.field)
        .map(|default| default.value.tokens())
}

/// The field `name` of a struct literal, from the parameter of the same
/// name: shorthand for an integer, which needs no conversion.
fn initialiser(name: &Ident, crossing: &Crossing) -> TokenStream {
    if let Crossing::Integer(_) = crossing {
        quote!(#name)
    } else {
        let owned = crossing.owned(name);
        quote!(#name: #owned)
    }
}

/// A constructor `name` of `parameters` that makes its object of `body`:
/// one that can only succeed is `#[must_use]`, and one that takes text,
/// `fallible`, fails for text that is not UTF-8.
fn constructor(
    name: &Ident,
    summary: &str,
    parameters: impl Iterator<Item = TokenStream>,
    body: &TokenStream,
    fallible: bool,
) -> TokenStream {
    if fallible {
        let documentation = doc_with_errors(summary, INVALID_TEXT);
        quote! {
            #documentation
            pub fn #name(#(#parameters),*) -> Result<Box<Self>, Box<EranosError>> {
                Ok(#body)
            }
        }
    } else {
        let documentation = doc(&[summary]);
        quote! {
            #documentation
            #[must_use]
            pub fn #name(#(#parameters),*) -> Box<Self> {
                #body
            }
        }
    }
}

/// The getter of `field` of a record, and when a call takes the record,
/// `taken`, its setter, and for an optional field its `clear_` method.
fn accessors(field: &Field, value: &Value, taken: bool) -> TokenStream {
    let getter = getter(field, value);
    if !taken {
        return getter;
    }
    let setter = setter(field, value);
    let clear = value.optional.then(|| clear(field, value));
    quote!(#getter #setter #clear)
}

/// The getter of `field` of a record: text written into a `DiplomatWrite`,
/// an integer as itself, a date, a time, or a C-like enum converted, and a
/// record or a list as a copy in an opaque object of its own.
fn getter(field: &Field, value: &Value) -> TokenStream {
    let name = &field.name;
    let summary = doc(&[&field.doc]);
    match (&value.crossing, value.optional) {
        (Crossing::Text, false) => quote! {
            #summary
            pub fn #name(&self, write: &mut DiplomatWrite) {
                crate::write_text(write, &self.0.#name);
            }
        },
        (Crossing::Text, true) => {
            let written = doc(&[&format!(
                "{} Writes it, if present; absent otherwise.",
                field.doc
            )]);
            quote! {
                #written
                pub fn #name(&self, write: &mut DiplomatWrite) -> Option<()> {
                    crate::write_optional(write, self.0.#name.as_deref())
                }
            }
        }
        (Crossing::Integer(integer), optional) => {
            let answer = if optional {
                quote!(Option<#integer>)
            } else {
                quote!(#integer)
            };
            quote! {
                #summary
                #[must_use]
                pub const fn #name(&self) -> #answer {
                    self.0.#name
                }
            }
        }
        (Crossing::Choice(converted) | Crossing::Plain(converted), false) => {
            let bridge = eranos(converted);
            quote! {
                #summary
                #[must_use]
                pub fn #name(&self) -> #bridge {
                    self.0.#name.into()
                }
            }
        }
        (Crossing::Choice(converted) | Crossing::Plain(converted), true) => {
            let bridge = eranos(converted);
            quote! {
                #summary
                #[must_use]
                pub fn #name(&self) -> Option<#bridge> {
                    self.0.#name.map(Into::into)
                }
            }
        }
        (Crossing::List(_) | Crossing::Object(_) | Crossing::TextList, false) => {
            let opaque = value.crossing.opaque();
            quote! {
                #summary
                #[must_use]
                pub fn #name(&self) -> Box<#opaque> {
                    Box::new(#opaque(self.0.#name.clone()))
                }
            }
        }
        (Crossing::List(_) | Crossing::Object(_) | Crossing::TextList, true) => {
            let opaque = value.crossing.opaque();
            quote! {
                #summary
                #[must_use]
                pub fn #name(&self) -> Option<Box<#opaque>> {
                    self.0.#name.clone().map(|value| Box::new(#opaque(value)))
                }
            }
        }
    }
}

/// The setter of `field` of a record: text fails when it is not UTF-8, a
/// record the library parses is set from the value's text, which fails also
/// when the library refuses it, an integer is set in a constant function,
/// and any other record or a list is copied into the field's own allocation
/// where it has one.
fn setter(field: &Field, value: &Value) -> TokenStream {
    let name = &field.name;
    let setter = format_ident!("set_{name}");
    let summary = format!("Sets {}", inside(&field.doc));
    let parsed = if let Crossing::Object(record) = &value.crossing {
        parsed(&record.to_string())
    } else {
        None
    };
    let (parameter, owned) = parsed.map_or_else(
        || (value.crossing.parameter(name), value.crossing.owned(name)),
        |parsed| {
            let helper = format_ident!("{}", parsed.helper);
            (
                quote!(#name: &DiplomatStr),
                quote!(eranos_bindings::#helper(&crate::text(#name)?).map_err(crate::failed)?),
            )
        },
    );
    let assigned = if value.optional {
        quote!(Some(#owned))
    } else {
        owned
    };
    if let Some(parsed) = parsed {
        let documentation = doc(&[&summary, parsed.fills, "# Errors", &parsed.errors()]);
        return quote! {
            #documentation
            pub fn #setter(&mut self, #parameter) -> Result<(), Box<EranosError>> {
                self.0.#name = #assigned;
                Ok(())
            }
        };
    }
    match (&value.crossing, value.optional) {
        (Crossing::Text, _) => {
            let documentation = doc_with_errors(&summary, INVALID_TEXT);
            quote! {
                #documentation
                pub fn #setter(&mut self, #parameter) -> Result<(), Box<EranosError>> {
                    self.0.#name = #assigned;
                    Ok(())
                }
            }
        }
        (Crossing::Integer(_), _) => {
            let documentation = doc(&[&summary]);
            quote! {
                #documentation
                pub const fn #setter(&mut self, #parameter) {
                    self.0.#name = #assigned;
                }
            }
        }
        (Crossing::List(_) | Crossing::Object(_) | Crossing::TextList, false) => {
            let documentation = doc(&[&summary]);
            quote! {
                #documentation
                pub fn #setter(&mut self, #parameter) {
                    self.0.#name.clone_from(&#name.0);
                }
            }
        }
        (
            Crossing::Choice(_)
            | Crossing::List(_)
            | Crossing::Object(_)
            | Crossing::Plain(_)
            | Crossing::TextList,
            _,
        ) => {
            let documentation = doc(&[&summary]);
            quote! {
                #documentation
                pub fn #setter(&mut self, #parameter) {
                    self.0.#name = #assigned;
                }
            }
        }
    }
}

/// The `clear_` method of the optional `field` of a record, constant when
/// the value it drops holds nothing to free.
fn clear(field: &Field, value: &Value) -> TokenStream {
    let name = &field.name;
    let clear = format_ident!("clear_{name}");
    let documentation = doc(&[&format!("Clears {}", inside(&field.doc))]);
    let constant = matches!(
        value.crossing,
        Crossing::Choice(_) | Crossing::Integer(_) | Crossing::Plain(_)
    )
    .then(|| quote!(const));
    quote! {
        #documentation
        pub #constant fn #clear(&mut self) {
            self.0.#name = None;
        }
    }
}

/// The C-like enum of the enum without data `item` of `variants`, which
/// converts to and from the core's.
fn choice(item: &Mirrored, variants: &[Variant]) -> Named {
    let (name, object) = (&item.name, eranos(&item.name));
    let summary = doc(&[&item.doc]);
    let variants = variants.iter().map(|variant| {
        let line = doc(&[&variant.doc]);
        let variant = &variant.name;
        quote!(#line #variant)
    });
    Named {
        name: object.to_string(),
        tokens: quote! {
            #summary
            #[derive(Debug)]
            #[diplomat::enum_convert(eranos_bindings::#name)]
            pub enum #object {
                #(#variants),*
            }
        },
    }
}

/// The opaque object of the enum with data `item` of `variants`, and the
/// C-like enum of its kinds.
///
/// # Errors
///
/// For a field whose type the bridge cannot carry, and an optional field.
fn data_enum(item: &Mirrored, variants: &[Variant], core: &Core) -> syn::Result<Vec<Named>> {
    let (name, object) = (&item.name, eranos(&item.name));
    let kind = format_ident!("{object}Kind");
    let kinds = variants.iter().map(|variant| {
        let line = doc(&[&variant.doc]);
        let variant = &variant.name;
        quote!(#line #variant)
    });
    let kinds_summary = doc(&[&format!("The kinds of {}.", words(name))]);
    let crossings = variants
        .iter()
        .map(|variant| {
            variant
                .fields
                .iter()
                .map(|field| {
                    let value = Value::of(&field.written, core)?;
                    if value.optional {
                        Err(syn::Error::new_spanned(
                            &field.name,
                            "a mirrored enum's variant holds no optional field",
                        ))
                    } else {
                        Ok(value.crossing)
                    }
                })
                .collect::<syn::Result<Vec<_>>>()
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let constructors = variants
        .iter()
        .zip(&crossings)
        .map(|(variant, crossings)| {
            parsed(&variant.name.to_string()).map_or_else(
                || Ok(create_variant(item, variant, crossings)),
                |parsed| create_parsed_variant(item, variant, parsed, core),
            )
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let arms = variants.iter().map(|variant| {
        let pattern = pattern(name, variant, None);
        let variant = &variant.name;
        quote!(#pattern => #kind::#variant)
    });
    let which = doc(&[&format!("Which kind of {} it is.", words(name))]);
    let getters = getters(variants, &crossings)
        .iter()
        .map(|getter| variant_getter(item, variants, getter))
        .collect::<Vec<_>>();
    let summary = doc(&[&item.doc]);
    let extras = extras(&object);
    Ok(vec![
        Named {
            name: kind.to_string(),
            tokens: quote! {
                #kinds_summary
                #[derive(Debug)]
                pub enum #kind {
                    #(#kinds),*
                }
            },
        },
        Named {
            name: object.to_string(),
            tokens: quote! {
                #summary
                #[diplomat::opaque]
                #[derive(Debug)]
                pub struct #object(pub(crate) eranos_bindings::#name);

                impl #object {
                    #(#constructors)*

                    #which
                    #[must_use]
                    pub const fn kind(&self) -> #kind {
                        match &self.0 {
                            #(#arms),*
                        }
                    }

                    #(#getters)*
                    #extras
                }
            },
        },
    ])
}

/// The constructor of `variant` of the enum with data `item`, which takes
/// the variant's fields in order, of `crossings`.
fn create_variant(item: &Mirrored, variant: &Variant, crossings: &[Crossing]) -> TokenStream {
    let (name, variant_name) = (&item.name, &variant.name);
    let parameters = variant
        .fields
        .iter()
        .zip(crossings)
        .map(|(field, crossing)| crossing.parameter(&field.name));
    let literal = if variant.fields.is_empty() {
        quote!(eranos_bindings::#name::#variant_name)
    } else {
        let initialisers = variant
            .fields
            .iter()
            .zip(crossings)
            .map(|(field, crossing)| initialiser(&field.name, crossing));
        quote!(eranos_bindings::#name::#variant_name { #(#initialisers),* })
    };
    constructor(
        &format_ident!("create_{}", crate::snake_case(&variant_name.to_string())),
        &variant.doc,
        parameters,
        &quote!(Box::new(Self(#literal))),
        crossings.contains(&Crossing::Text),
    )
}

/// The constructor of `variant` of the enum with data `item`, named after
/// the record the library parses, `parsed`, whose fields it holds: it takes
/// the text of the value, the first field, and the core's helper fills
/// every field.
///
/// # Errors
///
/// For a variant whose fields are not the record's, in name, type, and
/// order.
fn create_parsed_variant(
    item: &Mirrored,
    variant: &Variant,
    parsed: &Parsed,
    core: &Core,
) -> syn::Result<TokenStream> {
    let (name, variant_name) = (&item.name, &variant.name);
    let declared = |fields: Vec<&Field>| {
        fields
            .iter()
            .map(|field| format!("{}: {}", field.name, field.written.to_token_stream()))
            .collect::<Vec<_>>()
    };
    let holds_its_fields = core.item(parsed.record).is_some_and(|record| {
        declared(record.fields()) == declared(variant.fields.iter().collect())
    });
    let (true, Some(value)) = (holds_its_fields, variant.fields.first()) else {
        return Err(syn::Error::new_spanned(
            variant_name,
            "a variant named after a record the library parses holds the record's fields",
        ));
    };
    let value = &value.name;
    let fields = variant
        .fields
        .iter()
        .map(|field| &field.name)
        .collect::<Vec<_>>();
    let record = format_ident!("{}", parsed.record);
    let helper = format_ident!("{}", parsed.helper);
    let method = format_ident!("create_{}", crate::snake_case(&variant_name.to_string()));
    let documentation = doc(&[&variant.doc, parsed.fills, "# Errors", &parsed.errors()]);
    Ok(quote! {
        #documentation
        pub fn #method(#value: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
            eranos_bindings::#helper(&crate::text(#value)?)
                .map(|eranos_bindings::#record { #(#fields),* }| {
                    Box::new(Self(eranos_bindings::#name::#variant_name { #(#fields),* }))
                })
                .map_err(crate::failed)
        }
    })
}

/// The pattern of `variant` of the core's enum `name`, binding `field` when
/// given.
fn pattern(name: &Ident, variant: &Variant, field: Option<&Ident>) -> TokenStream {
    let variant_name = &variant.name;
    if variant.fields.is_empty() {
        quote!(eranos_bindings::#name::#variant_name)
    } else {
        let field = field.map(|field| quote!(#field,));
        quote!(eranos_bindings::#name::#variant_name { #field .. })
    }
}

/// A field that a variant of an enum with data holds.
#[derive(Clone, Copy)]
struct Held<'item> {
    /// How the field crosses.
    crossing: &'item Crossing,
    /// The field, as the variant declares it.
    field: &'item Field,
    /// The variant.
    variant: &'item Variant,
}

/// A getter of an enum with data: of a field name its variants hold with
/// one type, or of one variant's field when they hold it with several.
struct Getter<'item> {
    /// How the field crosses.
    crossing: Crossing,
    /// The field as the first variant that holds it declares it.
    field: &'item Field,
    /// Every variant that holds it, with its own declaration.
    holders: Vec<Held<'item>>,
    /// The getter's name: the field's, or the variant's and the field's.
    name: Ident,
}

/// The getters of the enum with data of `variants`, whose fields cross as
/// `crossings`, by name.
fn getters<'item>(
    variants: &'item [Variant],
    crossings: &'item [Vec<Crossing>],
) -> Vec<Getter<'item>> {
    let held = variants
        .iter()
        .zip(crossings)
        .flat_map(|(variant, crossings)| {
            variant
                .fields
                .iter()
                .zip(crossings)
                .map(move |(field, crossing)| Held {
                    crossing,
                    field,
                    variant,
                })
        })
        .collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    let mut getters = Vec::new();
    for first in &held {
        if !seen.insert(first.field.name.to_string()) {
            continue;
        }
        let holders = held
            .iter()
            .filter(|other| other.field.name == first.field.name)
            .copied()
            .collect::<Vec<_>>();
        let types = holders
            .iter()
            .map(|other| other.field.written.to_token_stream().to_string())
            .collect::<BTreeSet<_>>();
        if 1 == types.len() {
            getters.push(Getter {
                crossing: first.crossing.clone(),
                field: first.field,
                holders,
                name: first.field.name.clone(),
            });
        } else {
            getters.extend(holders.iter().map(|&holder| Getter {
                crossing: holder.crossing.clone(),
                field: holder.field,
                holders: vec![holder],
                name: format_ident!(
                    "{}_{}",
                    crate::snake_case(&holder.variant.name.to_string()),
                    holder.field.name
                ),
            }));
        }
    }
    getters.sort_by_key(|getter| getter.name.to_string());
    getters
}

/// The parts of an enum's getter that the crossing of its field decides.
struct GetterParts {
    /// Whether the getter is a constant function.
    constant: bool,
    /// The expression it evaluates in a variant that holds the field.
    found: TokenStream,
    /// Whether the getter is `#[must_use]`.
    must_use: bool,
    /// The getter's parameters, in their parentheses.
    receiver: TokenStream,
    /// The type the getter answers, inside the `Option` of a field some
    /// variants lack: none for text every variant holds, which it writes.
    answered: Option<TokenStream>,
}

/// The getter `getter` of the enum with data `item` of `variants`: absent
/// in the variants that do not hold its field, text written into a
/// `DiplomatWrite`, an integer as itself, a date, a time, or a C-like enum
/// converted, and a record or a list as a copy.
fn variant_getter(item: &Mirrored, variants: &[Variant], getter: &Getter<'_>) -> TokenStream {
    let (name, field) = (&getter.name, &getter.field.name);
    let holding = getter
        .holders
        .iter()
        .map(|holder| pattern(&item.name, holder.variant, Some(field)));
    let others = variants
        .iter()
        .filter(|variant| {
            !getter
                .holders
                .iter()
                .any(|holder| holder.variant.name == variant.name)
        })
        .map(|variant| pattern(&item.name, variant, None))
        .collect::<Vec<_>>();
    let optional = !others.is_empty();
    let absent = optional.then(|| quote!(#(#others)|* => None,));
    let mut text = described(getter);
    if optional {
        text.push(' ');
        text.push_str(&presence(getter));
    }
    let wrap = |found: TokenStream| {
        if optional {
            quote!(Some(#found))
        } else {
            found
        }
    };
    let GetterParts {
        constant,
        found,
        must_use,
        receiver,
        answered,
    } = match &getter.crossing {
        Crossing::Text => {
            let found = if optional {
                text.push_str(" Writes it, if present; absent otherwise.");
                quote!(crate::write_optional(write, Some(#field)))
            } else {
                quote!(crate::write_text(write, #field))
            };
            GetterParts {
                constant: false,
                found,
                must_use: false,
                receiver: quote!((&self, write: &mut DiplomatWrite)),
                answered: optional.then(|| quote!(())),
            }
        }
        Crossing::Integer(integer) => GetterParts {
            constant: true,
            found: wrap(quote!(*#field)),
            must_use: true,
            receiver: quote!((&self)),
            answered: Some(quote!(#integer)),
        },
        Crossing::Choice(converted) | Crossing::Plain(converted) => {
            let bridge = eranos(converted);
            GetterParts {
                constant: false,
                found: wrap(quote!((*#field).into())),
                must_use: true,
                receiver: quote!((&self)),
                answered: Some(quote!(#bridge)),
            }
        }
        Crossing::List(_) | Crossing::Object(_) | Crossing::TextList => {
            let opaque = getter.crossing.opaque();
            GetterParts {
                constant: false,
                found: wrap(quote!(Box::new(#opaque(#field.clone())))),
                must_use: true,
                receiver: quote!((&self)),
                answered: Some(quote!(Box<#opaque>)),
            }
        }
    };
    let answer = answered.map(|answered| {
        if optional {
            quote!(-> Option<#answered>)
        } else {
            quote!(-> #answered)
        }
    });
    let constant = constant.then(|| quote!(const));
    let must_use = must_use.then(|| quote!(#[must_use]));
    let documentation = doc(&[&text]);
    quote! {
        #documentation
        #must_use
        pub #constant fn #name #receiver #answer {
            match &self.0 {
                #(#holding)|* => #found,
                #absent
            }
        }
    }
}

/// The documentation of `getter`'s field: its own, or when its variants
/// document it apart, each variant's.
fn described(getter: &Getter<'_>) -> String {
    let first = &getter.field.doc;
    if getter
        .holders
        .iter()
        .all(|holder| holder.field.doc == *first)
    {
        return first.clone();
    }
    let parts = getter
        .holders
        .iter()
        .map(|holder| {
            format!(
                "in `{}`, {}",
                holder.variant.name,
                inside(holder.field.doc.trim_end_matches('.'))
            )
        })
        .collect::<Vec<_>>();
    format!("{}.", capitalised(&parts.join("; ")))
}

/// The sentence that says which variants hold `getter`'s field.
fn presence(getter: &Getter<'_>) -> String {
    let kinds = getter
        .holders
        .iter()
        .map(|holder| format!("`{}`", holder.variant.name))
        .collect::<Vec<_>>();
    let noun = if 1 == kinds.len() { "kind" } else { "kinds" };
    format!("Present in the {noun} {} alone.", listed(&kinds))
}

/// `items` as a list in a sentence: "A", "A and B", "A, B, and C".
fn listed(items: &[String]) -> String {
    let count = items.len();
    let mut text = String::new();
    for (index, item) in items.iter().enumerate() {
        if 0 < index && 2 < count {
            text.push_str(", ");
        }
        if 0 < index && index + 1 == count {
            text.push_str(if 2 < count { "and " } else { " and " });
        }
        text.push_str(item);
    }
    text
}

/// The opaque list of copies of the core's record `element`: mutable, with
/// `create` and `push`, when a call takes one, `taken`.
fn list_of(element: &Ident, taken: bool) -> Named {
    let (object, list) = (eranos(element), list(element));
    let summary = doc(&[&format!("A list of {} records.", words(element))]);
    let (opaque, create) = if taken {
        (
            quote!(#[diplomat::opaque_mut]),
            quote! {
                #[doc = "An empty list."]
                #[must_use]
                pub fn create() -> Box<Self> {
                    Box::new(Self(Vec::new()))
                }

                #[doc = "Adds a copy of `element` at the end."]
                pub fn push(&mut self, element: &#object) {
                    self.0.push(element.0.clone());
                }
            },
        )
    } else {
        (quote!(#[diplomat::opaque]), TokenStream::new())
    };
    let extras = extras(&list);
    Named {
        name: list.to_string(),
        tokens: quote! {
            #summary
            #opaque
            #[derive(Debug)]
            pub struct #list(pub(crate) Vec<eranos_bindings::#element>);

            impl #list {
                #create

                #[doc = "How many records the list holds."]
                #[must_use]
                pub const fn len(&self) -> usize {
                    self.0.len()
                }

                #[doc = "Whether the list holds no record."]
                #[must_use]
                pub const fn is_empty(&self) -> bool {
                    self.0.is_empty()
                }

                #[doc = "A copy of the record at `index`, absent past the end."]
                #[must_use]
                pub fn get(&self, index: usize) -> Option<Box<#object>> {
                    self.0
                        .get(index)
                        .map(|element| Box::new(#object(element.clone())))
                }

                #extras
            }
        },
    }
}

/// The methods of the bridge's type `object` that are not a field's, which
/// call the core's helpers: an expense's tip and total and the members it
/// names, an item's line total before its discount, its discount's amount,
/// and its line total, the members a split names, a balance's text, the
/// currencies, and the no-tip rule.
fn extras(object: &Ident) -> TokenStream {
    match object.to_string().as_str() {
        "EranosBalance" => quote! {
            /// Writes the balance as the library does: "gets back 12.00 EUR", "owes 12.00 EUR", or
            /// "settled".
            pub fn format(&self, write: &mut DiplomatWrite) {
                crate::write_text(write, &eranos_bindings::balance_format(&self.0));
            }
        },
        "EranosCurrencyDetails" => quote! {
            /// The currency of the ISO 4217 code `code`.
            ///
            /// # Errors
            ///
            /// [`EranosError`] of the kind `Money` for a code the library does not list, and of the
            /// kind `InvalidText` for text that is not UTF-8.
            pub fn of(code: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
                eranos_bindings::currency_details(&crate::text(code)?)
                    .map(|details| Box::new(Self(details)))
                    .map_err(crate::failed)
            }
        },
        "EranosCurrencyDetailsList" => quote! {
            /// Every currency of the ISO 4217 list that the library knows, by code.
            #[must_use]
            pub fn all() -> Box<Self> {
                Box::new(Self(eranos_bindings::currencies()))
            }
        },
        "EranosExpense" => quote! {
            /// The tip in the expense's currency, zero without one.
            ///
            /// # Errors
            ///
            /// [`EranosError`] of the kind `Expense` for a tip the expense cannot have, and of the
            /// kind `Money` for an amount out of range.
            pub fn tip_amount(&self) -> Result<Box<EranosMoney>, Box<EranosError>> {
                eranos_bindings::expense_tip_amount(&self.0)
                    .map(|money| Box::new(EranosMoney(money)))
                    .map_err(crate::failed)
            }

            /// The amount with the tip, what the payers paid.
            ///
            /// # Errors
            ///
            /// [`EranosError`] of the kind `Expense` for a tip the expense cannot have, and of the
            /// kind `Money` for an amount out of range.
            pub fn total(&self) -> Result<Box<EranosMoney>, Box<EranosError>> {
                eranos_bindings::expense_total(&self.0)
                    .map(|money| Box::new(EranosMoney(money)))
                    .map_err(crate::failed)
            }

            /// The identifiers of every member the expense names, payers and participants, each
            /// once and in the order of their identifiers.
            ///
            /// # Errors
            ///
            /// [`EranosError`] of the kind `Id` for an identifier that is not one.
            pub fn participants(&self) -> Result<Box<EranosTextList>, Box<EranosError>> {
                eranos_bindings::expense_members(&self.0)
                    .map(|members| Box::new(EranosTextList(members)))
                    .map_err(crate::failed)
            }
        },
        "EranosExpenseItem" => quote! {
            /// The unit price times the quantity, rounded half away from zero to the currency's minor
            /// unit, as the library computes it: the line total before the discount, zero when the
            /// product is below half a minor unit.
            ///
            /// # Errors
            ///
            /// As for `line_total`.
            pub fn line_total_before_discount(&self) -> Result<Box<EranosMoney>, Box<EranosError>> {
                eranos_bindings::expense_item_line_total_before_discount(&self.0)
                    .map(|money| Box::new(EranosMoney(money)))
                    .map_err(crate::failed)
            }

            /// What the discount takes off the line, as the library computes it: the line total
            /// before the discount minus the line total, zero without a discount.
            ///
            /// # Errors
            ///
            /// As for `line_total`.
            pub fn discount_amount(&self) -> Result<Box<EranosMoney>, Box<EranosError>> {
                eranos_bindings::expense_item_discount_amount(&self.0)
                    .map(|money| Box::new(EranosMoney(money)))
                    .map_err(crate::failed)
            }

            /// The unit price times the quantity, rounded half away from zero to the currency's minor
            /// unit, less the discount, as the library computes it: zero for a free line, whose
            /// product is below half a minor unit or whose discount takes the whole price.
            ///
            /// # Errors
            ///
            /// [`EranosError`] of the kind `Money` for a quantity that is not positive, a percentage
            /// discount below zero, or an amount in a currency the library does not list, of the kind
            /// `Expense` for an item the library
            /// refuses, a discount the line cannot take, a split that does not divide the line total,
            /// or a line total out of range among them, and of the kind `Id` for an identifier that
            /// is not one.
            pub fn line_total(&self) -> Result<Box<EranosMoney>, Box<EranosError>> {
                eranos_bindings::expense_item_line_total(&self.0)
                    .map(|money| Box::new(EranosMoney(money)))
                    .map_err(crate::failed)
            }
        },
        "EranosExpenseSplit" => quote! {
            /// The identifiers of every member who shares the cost, each once and in the order of
            /// their identifiers.
            ///
            /// # Errors
            ///
            /// [`EranosError`] of the kind `Id` for an identifier that is not one, and the errors of an
            /// item that does not convert.
            pub fn participants(&self) -> Result<Box<EranosTextList>, Box<EranosError>> {
                eranos_bindings::expense_split_participants(&self.0)
                    .map(|members| Box::new(EranosTextList(members)))
                    .map_err(crate::failed)
            }
        },
        "EranosSplit" => quote! {
            /// The identifiers of the members the split divides among, in the order it lists them.
            ///
            /// # Errors
            ///
            /// [`EranosError`] of the kind `Id` for an identifier that is not one.
            pub fn participants(&self) -> Result<Box<EranosTextList>, Box<EranosError>> {
                eranos_bindings::split_members(&self.0)
                    .map(|members| Box::new(EranosTextList(members)))
                    .map_err(crate::failed)
            }
        },
        "EranosSummary" => quote! {
            /// The totals of the member with the identifier `member`: all zero, and settled, for a
            /// member without entries.
            ///
            /// # Errors
            ///
            /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
            pub fn member(&self, member: &DiplomatStr) -> Result<Box<EranosMemberSummary>, Box<EranosError>> {
                Ok(Box::new(EranosMemberSummary(self.0.member(&crate::text(member)?))))
            }

            /// The spending in the category with the identifier `category`: zero for a category
            /// without expenses.
            ///
            /// # Errors
            ///
            /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
            pub fn category_total(&self, category: &DiplomatStr) -> Result<Box<EranosMoney>, Box<EranosError>> {
                Ok(Box::new(EranosMoney(self.0.category_total(&crate::text(category)?))))
            }

            /// The share of the member with the identifier `member` of the spending in the category
            /// with the identifier `category`: zero when they have none there.
            ///
            /// # Errors
            ///
            /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
            pub fn member_category_total(
                &self,
                member: &DiplomatStr,
                category: &DiplomatStr,
            ) -> Result<Box<EranosMoney>, Box<EranosError>> {
                Ok(Box::new(EranosMoney(
                    self.0.member_category_total(&crate::text(member)?, &crate::text(category)?),
                )))
            }
        },
        "EranosTipRule" => quote! {
            /// The no-tip rule, which an expense without a tip has: 0 % of the amount, shared
            /// proportionally.
            #[must_use]
            pub fn none() -> Box<Self> {
                Box::new(Self(eranos_bindings::tip_rule_none()))
            }
        },
        _ => TokenStream::new(),
    }
}

#[cfg(test)]
mod tests;
