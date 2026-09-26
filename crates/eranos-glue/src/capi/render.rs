//! The text of a generated file before rustfmt lays it out: every run of
//! documentation attributes as doc comments, each paragraph wrapped to
//! rustfmt's width at its depth, and a blank line between items. rustfmt
//! keeps comments and blank lines as they are and lays out the code, so
//! the file it writes reads as a hand-written one.

use quote::{ToTokens, quote};
use syn::{Attribute, Expr, File, ImplItem, Item, Lit, Meta};

use crate::docs::paragraphs;

/// The width rustfmt lays code out to, which the doc comments keep to.
const WIDTH: usize = 100;

/// The comment that opens every generated file.
const NOTICE: [&str; 2] = [
    "// Written by `cargo xtask generate` from the binding core's records and calls: edit those,",
    "// or the generator in eranos-glue, and run the command again, never this file.",
];

/// The text of `file`.
pub(crate) fn text(file: &File) -> String {
    let mut text = String::new();
    for line in NOTICE {
        text.push_str(line);
        text.push('\n');
    }
    text.push('\n');
    attributes(&file.attrs, 0, &mut text);
    items(&file.items, 0, &mut text);
    text
}

/// Writes `items` at `depth`, a blank line between each two but for two
/// imports, which rustfmt sorts as one group.
fn items(items: &[Item], depth: usize, text: &mut String) {
    let mut previous: Option<&Item> = None;
    for item in items {
        if let Some(previous) = previous
            && !(matches!(previous, Item::Use(_)) && matches!(item, Item::Use(_)))
        {
            text.push('\n');
        }
        item_text(item, depth, text);
        previous = Some(item);
    }
}

/// Writes `item` at `depth`: a module's items, an `impl`'s functions, and
/// an enum's variants each with their own doc comments.
fn item_text(item: &Item, depth: usize, text: &mut String) {
    let indent = "    ".repeat(depth);
    if let Item::Mod(module) = item
        && let Some((_, content)) = &module.content
    {
        attributes(&module.attrs, depth, text);
        let (visibility, name) = (&module.vis, &module.ident);
        line(
            text,
            &indent,
            &quote!(#visibility mod #name).to_string(),
            " {",
        );
        items(content, depth + 1, text);
        line(text, &indent, "}", "");
    } else if let Item::Impl(implementation) = item {
        attributes(&implementation.attrs, depth, text);
        let implemented = &implementation.self_ty;
        line(text, &indent, &quote!(impl #implemented).to_string(), " {");
        let inner = "    ".repeat(depth + 1);
        for (index, member) in implementation.items.iter().enumerate() {
            if 0 < index {
                text.push('\n');
            }
            if let ImplItem::Fn(function) = member {
                attributes(&function.attrs, depth + 1, text);
                let mut bare = function.clone();
                bare.attrs.clear();
                line(text, &inner, &bare.to_token_stream().to_string(), "");
            } else {
                line(text, &inner, &member.to_token_stream().to_string(), "");
            }
        }
        line(text, &indent, "}", "");
    } else if let Item::Enum(choice) = item {
        attributes(&choice.attrs, depth, text);
        let (visibility, name) = (&choice.vis, &choice.ident);
        line(
            text,
            &indent,
            &quote!(#visibility enum #name).to_string(),
            " {",
        );
        let inner = "    ".repeat(depth + 1);
        for variant in &choice.variants {
            attributes(&variant.attrs, depth + 1, text);
            let mut bare = variant.clone();
            bare.attrs.clear();
            line(text, &inner, &bare.to_token_stream().to_string(), ",");
        }
        line(text, &indent, "}", "");
    } else if let Item::Struct(record) = item {
        attributes(&record.attrs, depth, text);
        let mut bare = record.clone();
        bare.attrs.clear();
        line(text, &indent, &bare.to_token_stream().to_string(), "");
    } else {
        line(text, &indent, &item.to_token_stream().to_string(), "");
    }
}

/// Writes `attributes` at `depth`, in their order: each run of
/// documentation as doc comments, outer or inner as the attributes are,
/// and every other attribute as it is.
fn attributes(attributes: &[Attribute], depth: usize, text: &mut String) {
    let indent = "    ".repeat(depth);
    let mut run = Vec::new();
    let mut inner = false;
    for attribute in attributes {
        if let Some(documentation) = doc_text(attribute) {
            inner = matches!(attribute.style, syn::AttrStyle::Inner(_));
            run.push(documentation);
        } else {
            comments(&run, &indent, inner, text);
            run.clear();
            line(text, &indent, &attribute.to_token_stream().to_string(), "");
        }
    }
    comments(&run, &indent, inner, text);
}

/// Writes the doc comments of `run`, the texts of a run of documentation
/// attributes, at `indent`: a paragraph's words on lines of rustfmt's width
/// at most, and an empty comment between paragraphs.
fn comments(run: &[String], indent: &str, inner: bool, text: &mut String) {
    let marker = if inner { "//!" } else { "///" };
    let width = WIDTH.saturating_sub(indent.len() + marker.len() + 1);
    for (index, paragraph) in paragraphs(run).iter().enumerate() {
        if 0 < index {
            line(text, indent, marker, "");
        }
        for wrapped in wrap(paragraph, width) {
            text.push_str(indent);
            text.push_str(marker);
            text.push(' ');
            text.push_str(&wrapped);
            text.push('\n');
        }
    }
}

/// The text of `attribute` when it is a documentation attribute of text.
fn doc_text(attribute: &Attribute) -> Option<String> {
    if attribute.path().is_ident("doc")
        && let Meta::NameValue(pair) = &attribute.meta
        && let Expr::Lit(literal) = &pair.value
        && let Lit::Str(documentation) = &literal.lit
    {
        Some(documentation.value())
    } else {
        None
    }
}

/// `paragraph` on lines of `width` characters at most, but for a word
/// longer than that, which has a line of its own.
fn wrap(paragraph: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in paragraph.split_whitespace() {
        if let Some(last) = lines.last_mut()
            && last.chars().count() + 1 + word.chars().count() <= width
        {
            last.push(' ');
            last.push_str(word);
        } else {
            lines.push(word.to_owned());
        }
    }
    lines
}

/// Writes a line of `code` at `indent`, followed by `end`.
fn line(text: &mut String, indent: &str, code: &str, end: &str) {
    text.push_str(indent);
    text.push_str(code);
    text.push_str(end);
    text.push('\n');
}

#[cfg(test)]
mod tests;
