//! The C glue's bridge, written from the binding core: an opaque object for
//! every record and enum with data that the group calls reach, with their
//! lists and C-like enums, and the group, its read handle, the cancellation,
//! and the outcomes of the asynchronous calls.
//!
//! `diplomat-tool` reads the C glue's source files, not what its macros
//! expand to, so the bridge is code in files: `cargo xtask generate` writes
//! them and rustfmt lays them out, and a test of xtask fails while a
//! committed file holds other code than the generator writes. What the core
//! cannot say, the generator holds as templates: the group's constructors
//! and read handle, the cancellation, and the few methods of a record that
//! are not a field's.

mod compare;
mod crossing;
mod expense_group;
mod items;
mod records;
mod render;

use std::collections::BTreeSet;
use std::fmt;

use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{format_ident, quote};

use crate::capi::crossing::Plan;
use crate::capi::items::Core;

/// The C glue's hand-written types that the bridge names: the plain structs
/// of a date and a time of day, amounts, and lists of text, with methods of
/// their own, in its module `values`.
const VALUES: [&str; 4] = [
    "EranosCalendarDate",
    "EranosMoney",
    "EranosTextList",
    "EranosTimeOfDay",
];

/// The failure every call that takes text reports besides its own.
const INVALID_TEXT: &str = "[`EranosError`] of the kind `InvalidText` for text that is not UTF-8.";

/// A file of the C glue's bridge.
pub struct Generated {
    /// Where it goes, from the C glue's `src` directory.
    path: &'static str,
    /// Its code.
    file: syn::File,
}

impl fmt::Debug for Generated {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Generated")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl Generated {
    /// Where the file goes, from the C glue's `src` directory:
    /// `generated/records.rs`.
    #[must_use]
    pub const fn path(&self) -> &'static str {
        self.path
    }

    /// The file's text before rustfmt lays it out: its documentation as doc
    /// comments wrapped to rustfmt's width, and a blank line between items.
    #[must_use]
    pub fn text(&self) -> String {
        render::text(&self.file)
    }

    /// Whether `committed`, the text of the file as the repository holds
    /// it, holds this file's code, however it is laid out and wrapped.
    ///
    /// # Errors
    ///
    /// For committed text that is not Rust.
    pub fn is_written_in(&self, committed: &str) -> syn::Result<bool> {
        compare::same(committed, &self.file)
    }
}

/// The files of the C glue's bridge that `sources`, the text of each of the
/// binding core's source files, give.
///
/// # Errors
///
/// For a source that is not Rust, a core without its trait of calls, and a
/// record, an enum, or a call whose types the bridge cannot carry.
pub(crate) fn bridge(sources: &[String]) -> syn::Result<Vec<Generated>> {
    let core = Core::parse(sources)?;
    let plan = Plan::of(&core)?;
    let records = records::items(&core, &plan)?;
    let written = records.iter().map(|named| named.name.clone()).collect();
    let group = expense_group::items(&core)?;
    Ok(vec![
        Generated {
            path: "generated/expense_group.rs",
            file: module(group, &written),
        },
        Generated {
            path: "generated/records.rs",
            file: module(records, &BTreeSet::new()),
        },
    ])
}

/// An item of a bridge module, and the items it comes with, such as its
/// `impl`, under the name that orders them.
pub(crate) struct Named {
    /// The name of the item's type.
    pub(crate) name: String,
    /// The code of the item and of the items it comes with.
    pub(crate) tokens: TokenStream,
}

/// The `use` declaration of a bridge module for the names it takes from one
/// module.
struct Import {
    /// The names it imports, none when the module uses none of them.
    names: Vec<Ident>,
    /// The module's path.
    path: TokenStream,
}

/// The file of a bridge module holding `items`, in the order of their names,
/// with an import of every type it names from the C glue's other modules:
/// `records`, the names of the generated module `records`, and the
/// hand-written ones.
fn module(mut items: Vec<Named>, records: &BTreeSet<String>) -> syn::File {
    items.sort_by(|left, right| left.name.cmp(&right.name));
    let defined = items
        .iter()
        .map(|named| named.name.clone())
        .collect::<BTreeSet<_>>();
    let mut used = BTreeSet::new();
    for named in &items {
        names(&named.tokens, &mut used);
    }
    let values = VALUES.map(str::to_owned).into_iter().collect();
    let imported = |names: &BTreeSet<String>| {
        names
            .iter()
            .filter(|name| used.contains(*name) && !defined.contains(*name))
            .map(|name| format_ident!("{name}"))
            .collect::<Vec<_>>()
    };
    let imports = [
        Import {
            names: imported(&["EranosError".to_owned()].into()),
            path: quote!(crate::error::ffi),
        },
        Import {
            names: imported(records),
            path: quote!(crate::generated::records::ffi),
        },
        Import {
            names: imported(&values),
            path: quote!(crate::values::ffi),
        },
        Import {
            names: imported(&["DiplomatStr".to_owned(), "DiplomatWrite".to_owned()].into()),
            path: quote!(diplomat_runtime),
        },
        Import {
            names: imported(&["Arc".to_owned(), "OnceLock".to_owned(), "mpsc".to_owned()].into()),
            path: quote!(std::sync),
        },
    ]
    .into_iter()
    .filter_map(|Import { names, path }| match names.as_slice() {
        [] => None,
        [name] => Some(quote!(use #path::#name;)),
        _ => Some(quote!(use #path::{#(#names),*};)),
    });
    let items = items.iter().map(|named| &named.tokens);
    file(quote! {
        #[doc = "The types and functions of the C and C++ APIs; C names each type as it is here, and C++ puts it in the namespace `eranos::detail`."]
        #[expect(
            clippy::unnecessary_box_returns,
            reason = "the functions Diplomat writes hand out an opaque type only in a box"
        )]
        #[diplomat::bridge]
        #[diplomat::attr(cpp, namespace = "eranos::detail")]
        pub mod ffi {
            #(#imports)*
            #(#items)*
        }
    })
}

/// The file of `tokens`. The generator writes Rust, so they always parse;
/// tokens that did not would stay in the file as they are, verbatim, which
/// rustfmt and the compiler refuse when `cargo xtask generate` writes it.
fn file(tokens: TokenStream) -> syn::File {
    syn::parse2(tokens.clone()).unwrap_or_else(|_| syn::File {
        shebang: None,
        frontmatter: None,
        attrs: Vec::new(),
        items: vec![syn::Item::Verbatim(tokens)],
    })
}

/// Adds to `into` every identifier of `tokens`, those inside groups too.
fn names(tokens: &TokenStream, into: &mut BTreeSet<String>) {
    for tree in tokens.clone() {
        if let TokenTree::Ident(ident) = &tree {
            into.insert(ident.to_string());
        } else if let TokenTree::Group(group) = &tree {
            names(&group.stream(), into);
        }
    }
}

/// The name of the bridge's type for the core's `name`: `Expense` is
/// `EranosExpense`.
fn eranos(name: &Ident) -> Ident {
    format_ident!("Eranos{name}")
}

/// The name of the bridge's list of the core's `element`:
/// `EranosExpenseList`.
fn list(element: &Ident) -> Ident {
    format_ident!("Eranos{element}List")
}

/// The core's type `name` in words: `MemberAmount` is "member amount".
fn words(name: &Ident) -> String {
    crate::snake_case(&name.to_string()).replace('_', " ")
}

/// The nouns, among those the core names its records for, that count
/// nothing, and so take no indefinite article.
const UNCOUNTABLE: [&str; 1] = ["money"];

/// The noun of `words`, their last word: "shares" of "expense shares".
fn noun(words: &str) -> &str {
    words.rsplit_once(' ').map_or(words, |(_, noun)| noun)
}

/// Whether `words` name things in the plural: their noun ends in `s`, as
/// "expense shares" and "member parts" do, but not in `ss` or `us`, as the
/// singular "address" and "status" do.
fn plural(words: &str) -> bool {
    let noun = noun(words);
    noun.ends_with('s') && !noun.ends_with("ss") && !noun.ends_with("us")
}

/// `words` after their indefinite article: "an expense", "a member amount";
/// a plural noun, as in "expense shares", and an uncountable one, "money",
/// take none.
fn article(words: &str) -> String {
    if plural(words) || UNCOUNTABLE.contains(&noun(words)) {
        words.to_owned()
    } else if words.starts_with(['a', 'e', 'i', 'o', 'u']) {
        format!("an {words}")
    } else {
        format!("a {words}")
    }
}

/// `text` as it starts a sentence: its first letter in upper case.
fn capitalised(text: &str) -> String {
    let mut characters = text.chars();
    characters
        .next()
        .map(|first| first.to_uppercase().chain(characters).collect())
        .unwrap_or_default()
}

/// `text` as it reads inside a sentence: its first letter in lower case,
/// unless its first word is an initialism, such as "ISO" or "IBAN".
fn inside(text: &str) -> String {
    let mut characters = text.chars();
    match (characters.next(), characters.next()) {
        (Some(first), Some(second)) if !second.is_uppercase() => {
            first.to_lowercase().chain(text.chars().skip(1)).collect()
        }
        _ => text.to_owned(),
    }
}

/// The documentation attribute of `paragraphs`.
fn doc(paragraphs: &[&str]) -> TokenStream {
    let text = paragraphs.join("\n\n");
    quote!(#[doc = #text])
}

/// The documentation attribute of `summary`, followed by an `# Errors`
/// section of `errors`.
fn doc_with_errors(summary: &str, errors: &str) -> TokenStream {
    doc(&[summary, "# Errors", errors])
}

#[cfg(test)]
mod tests;
