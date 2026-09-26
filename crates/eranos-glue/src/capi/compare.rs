//! Whether a committed file holds the code the generator writes, however
//! rustfmt laid it out. The comparison reads both as tokens, so line breaks
//! and spaces cannot differ, and it also reads past what rustfmt changes in
//! the tokens themselves: a comma before a closing delimiter, which it adds
//! to a list it breaks over lines and removes from one it joins, and the
//! braces around a `match` arm's expression, which it adds when the
//! expression does not fit after the arrow. Documentation compares as its
//! paragraphs, whatever its lines and its attributes, so the wrapping of a
//! doc comment cannot differ either.

use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::ToTokens;
use syn::LitStr;

use crate::docs::paragraphs;

/// A piece of code as the comparison reads it.
#[derive(Debug, PartialEq, Eq)]
enum Piece {
    /// A group's closing delimiter.
    Close,
    /// A run of documentation attributes, inner when `true`: the texts of
    /// its attributes, until [`normalised`] turns them into paragraphs.
    Docs(bool, Vec<String>),
    /// A group's opening delimiter.
    Open(Delimiter),
    /// Any other token, as it prints.
    Token(String),
}

/// Whether the text `committed` holds the code of `generated`.
///
/// # Errors
///
/// For committed text that is not Rust.
pub(crate) fn same(committed: &str, generated: &syn::File) -> syn::Result<bool> {
    let committed = syn::parse_file(committed)?;
    Ok(normalised(committed.into_token_stream()) == normalised(generated.to_token_stream()))
}

/// The pieces of `tokens`, each run of documentation as its paragraphs,
/// their words one space apart.
fn normalised(tokens: TokenStream) -> Vec<Piece> {
    let mut pieces = Vec::new();
    walk(tokens, &mut pieces);
    for piece in &mut pieces {
        if let Piece::Docs(_, texts) = piece {
            *texts = paragraphs(texts)
                .iter()
                .map(|paragraph| paragraph.split_whitespace().collect::<Vec<_>>().join(" "))
                .collect();
        }
    }
    pieces
}

/// Adds the pieces of `tokens` to `into`.
fn walk(tokens: TokenStream, into: &mut Vec<Piece>) {
    let trees = tokens.into_iter().collect::<Vec<_>>();
    let mut index = 0;
    while let Some(tree) = trees.get(index) {
        index += 1;
        if let TokenTree::Punct(punct) = tree
            && '#' == punct.as_char()
        {
            let inner =
                matches!(trees.get(index), Some(TokenTree::Punct(bang)) if '!' == bang.as_char());
            let at = if inner { index + 1 } else { index };
            if let Some(TokenTree::Group(group)) = trees.get(at)
                && let Some(text) = doc_text(group)
            {
                if let Some(Piece::Docs(was_inner, texts)) = into.last_mut()
                    && *was_inner == inner
                {
                    texts.push(text);
                } else {
                    into.push(Piece::Docs(inner, vec![text]));
                }
                index = at + 1;
                continue;
            }
        }
        match tree {
            TokenTree::Group(group) => {
                if Delimiter::Brace == group.delimiter()
                    && after_arrow(into)
                    && !has_statements(group)
                {
                    walk(group.stream(), into);
                    if let Some(TokenTree::Punct(comma)) = trees.get(index)
                        && ',' == comma.as_char()
                    {
                        index += 1;
                    }
                    comma(trees.get(index), into);
                } else {
                    into.push(Piece::Open(group.delimiter()));
                    walk(group.stream(), into);
                    into.push(Piece::Close);
                }
            }
            TokenTree::Punct(punct) if ',' == punct.as_char() => comma(trees.get(index), into),
            TokenTree::Ident(_) | TokenTree::Literal(_) | TokenTree::Punct(_) => {
                into.push(Piece::Token(tree.to_string()));
            }
        }
    }
}

/// The text of the bracketed `group` of an attribute when it is
/// `doc = "…"`.
fn doc_text(group: &Group) -> Option<String> {
    if Delimiter::Bracket != group.delimiter() {
        return None;
    }
    let mut trees = group.stream().into_iter();
    match (trees.next(), trees.next(), trees.next(), trees.next()) {
        (
            Some(TokenTree::Ident(name)),
            Some(TokenTree::Punct(equals)),
            Some(TokenTree::Literal(literal)),
            None,
        ) if '=' == equals.as_char() => match name.to_string().as_str() {
            "doc" => syn::parse2::<LitStr>(TokenTree::Literal(literal).into_token_stream())
                .ok()
                .map(|text| text.value()),
            _ => None,
        },
        _ => None,
    }
}

/// Whether the pieces so far end with a `match` arm's arrow, `=>`.
fn after_arrow(pieces: &[Piece]) -> bool {
    matches!(pieces, [.., Piece::Token(equals), Piece::Token(greater)] if "=" == equals && ">" == greater)
}

/// Whether the braced `group` holds statements, so it is a block that
/// rustfmt would keep in its braces.
fn has_statements(group: &Group) -> bool {
    group
        .stream()
        .into_iter()
        .any(|tree| matches!(&tree, TokenTree::Punct(punct) if ';' == punct.as_char()))
}

/// Adds a comma to `into`, unless it is a trailing one: the last of its
/// group, or the last of a list of generic arguments, before `next`.
fn comma(next: Option<&TokenTree>, into: &mut Vec<Piece>) {
    let trailing = match next {
        None => true,
        Some(TokenTree::Punct(punct)) => '>' == punct.as_char(),
        Some(TokenTree::Group(_) | TokenTree::Ident(_) | TokenTree::Literal(_)) => false,
    };
    if !trailing {
        into.push(Piece::Token(",".to_owned()));
    }
}

#[cfg(test)]
mod tests;
