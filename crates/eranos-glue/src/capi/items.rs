//! The binding core's mirrored items as the C bridge reads them: each
//! record's fields and each enum's variants, with their documentation, and
//! the trait of group calls, found in the core's source files. The
//! documentation's links to the core's items read as code spans, since the
//! C glue holds none of them.

use proc_macro2::Ident;
use syn::{Attribute, Fields, Item, ItemTrait, Type};

use crate::calls::Calls;
use crate::docs::{Docs, foreign};

/// A field of a record or of an enum's variant.
pub(crate) struct Field {
    /// Its documentation, one line.
    pub(crate) doc: String,
    /// Its name.
    pub(crate) name: Ident,
    /// Its type, as the core declares it.
    pub(crate) written: Type,
}

/// A variant of an enum.
pub(crate) struct Variant {
    /// Its documentation, one line.
    pub(crate) doc: String,
    /// Its fields, none for a unit variant.
    pub(crate) fields: Vec<Field>,
    /// Its name.
    pub(crate) name: Ident,
}

/// What a mirrored item holds.
pub(crate) enum Shape {
    /// A choice of variants, each unit or with named fields.
    Choice(Vec<Variant>),
    /// Named fields.
    Record(Vec<Field>),
}

/// A mirrored item of the core.
pub(crate) struct Mirrored {
    /// Its documentation, one line.
    pub(crate) doc: String,
    /// Its name.
    pub(crate) name: Ident,
    /// Its fields or variants.
    pub(crate) shape: Shape,
}

impl Mirrored {
    /// Whether it is an enum whose variants hold nothing, which crosses as
    /// a C-like enum.
    pub(crate) fn is_plain_enum(&self) -> bool {
        matches!(&self.shape, Shape::Choice(variants) if variants.iter().all(|variant| variant.fields.is_empty()))
    }

    /// Every field it holds: a record's, or those of every variant, in
    /// order.
    pub(crate) fn fields(&self) -> Vec<&Field> {
        match &self.shape {
            Shape::Choice(variants) => variants
                .iter()
                .flat_map(|variant| &variant.fields)
                .collect(),
            Shape::Record(fields) => fields.iter().collect(),
        }
    }
}

/// The core's mirrored items and its trait of calls.
pub(crate) struct Core {
    /// The calls the trait lists.
    pub(crate) calls: Calls,
    /// Every mirrored item, in the order of the sources.
    pub(crate) items: Vec<Mirrored>,
}

impl Core {
    /// The items and the calls of `sources`, the text of each of the core's
    /// source files.
    ///
    /// # Errors
    ///
    /// For a source that is not Rust, a mirrored item the rules refuse, no
    /// trait of calls or more than one, and a trait the calls' rules refuse.
    pub(crate) fn parse(sources: &[String]) -> syn::Result<Self> {
        let mut items = Vec::new();
        let mut traits = Vec::new();
        for source in sources {
            for item in syn::parse_file(source)?.items {
                if let Item::Trait(trait_item) = &item
                    && marked(&trait_item.attrs, "calls")
                {
                    traits.push(trait_item.clone());
                } else if let Some(mirrored) = mirrored(&item)? {
                    items.push(mirrored);
                }
            }
        }
        let [trait_item]: [ItemTrait; 1] = traits.try_into().map_err(|_| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "the core holds one trait of group calls, marked `#[calls]`",
            )
        })?;
        Ok(Self {
            calls: Calls::parse(&trait_item)?,
            items,
        })
    }

    /// The mirrored item named `name`.
    pub(crate) fn item(&self, name: &str) -> Option<&Mirrored> {
        self.items.iter().find(|item| item.name == name)
    }
}

/// Whether `attributes` hold the glue's attribute `name`, written with the
/// macros crate's path or without it.
fn marked(attributes: &[Attribute], name: &str) -> bool {
    attributes.iter().any(|attribute| {
        attribute
            .path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == name)
    })
}

/// The mirror of `item`, when it is marked `#[mirrored]`.
///
/// # Errors
///
/// For documentation that is not text, and a field without a name.
fn mirrored(item: &Item) -> syn::Result<Option<Mirrored>> {
    let (attributes, name, shape) = if let Item::Struct(record) = item
        && marked(&record.attrs, "mirrored")
    {
        (
            &record.attrs,
            &record.ident,
            Shape::Record(fields(&record.fields)?),
        )
    } else if let Item::Enum(choice) = item
        && marked(&choice.attrs, "mirrored")
    {
        let variants = choice
            .variants
            .iter()
            .map(|variant| {
                Ok(Variant {
                    doc: line(&variant.attrs)?,
                    fields: fields(&variant.fields)?,
                    name: variant.ident.clone(),
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;
        (&choice.attrs, &choice.ident, Shape::Choice(variants))
    } else {
        return Ok(None);
    };
    Ok(Some(Mirrored {
        doc: line(attributes)?,
        name: name.clone(),
        shape,
    }))
}

/// The fields of a record or a variant.
///
/// # Errors
///
/// For documentation that is not text, and a field without a name, which
/// `#[mirrored]` refuses too.
fn fields(fields: &Fields) -> syn::Result<Vec<Field>> {
    fields
        .iter()
        .map(|field| {
            Ok(Field {
                doc: line(&field.attrs)?,
                name: field.ident.clone().ok_or_else(|| {
                    syn::Error::new_spanned(field, "a mirrored item names its fields")
                })?,
                written: field.ty.clone(),
            })
        })
        .collect()
}

/// The documentation among `attributes` as one line: its paragraphs joined
/// with a space, and its links as code spans.
///
/// # Errors
///
/// For documentation that is not text.
fn line(attributes: &[Attribute]) -> syn::Result<String> {
    Ok(foreign(&Docs::lines(attributes)?.join(" ")))
}

#[cfg(test)]
mod tests;
