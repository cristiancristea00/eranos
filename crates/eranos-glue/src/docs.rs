//! A call's documentation, and the documentation of each form written for
//! it: the group handle's synchronous and asynchronous forms, the read
//! handle's, and the foreign bindings' copies of them.

use proc_macro2::Ident;
use syn::{Attribute, Expr, Lit};

use crate::calls::Access;

/// A call's documentation: the paragraphs of its doc comment, each on one
/// line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Docs {
    /// The first paragraph, which ends with a full stop.
    summary: String,
    /// The paragraphs after it.
    details: Vec<String>,
}

impl Docs {
    /// The documentation that the `#[doc]` attributes among `attributes`
    /// give the call `name`.
    ///
    /// # Errors
    ///
    /// For a `#[doc]` that is not a string, a call without documentation,
    /// and a summary that does not end with a full stop, which the
    /// asynchronous form's summary replaces.
    pub(crate) fn parse(attributes: &[Attribute], name: &Ident) -> syn::Result<Self> {
        let mut paragraphs: Vec<String> = Vec::new();
        let mut open = false;
        for attribute in attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident("doc"))
        {
            let value = &attribute.meta.require_name_value()?.value;
            let text = if let Expr::Lit(literal) = value
                && let Lit::Str(text) = &literal.lit
            {
                text.value()
            } else {
                return Err(syn::Error::new_spanned(
                    value,
                    "a group call's documentation is text",
                ));
            };
            if text.trim().is_empty() {
                open = false;
            }
            for line in text.lines().map(str::trim) {
                if line.is_empty() {
                    open = false;
                } else if open && let Some(paragraph) = paragraphs.last_mut() {
                    paragraph.push(' ');
                    paragraph.push_str(line);
                } else {
                    paragraphs.push(line.to_owned());
                    open = true;
                }
            }
        }
        let mut paragraphs = paragraphs.into_iter();
        match paragraphs.next() {
            Some(summary) if summary.ends_with('.') => Ok(Self {
                summary,
                details: paragraphs.collect(),
            }),
            _ => Err(syn::Error::new_spanned(
                name,
                "a group call is documented, its summary ending with a full stop",
            )),
        }
    }

    /// The documentation of the synchronous form, which fails as the call's
    /// operation does, unless it is `infallible`, and on wasm when the lock
    /// its `access` takes is taken.
    pub(crate) fn synchronous(&self, access: Access, infallible: bool) -> String {
        let busy = match access {
            Access::Read => {
                "[`GroupFailure::Busy`](crate::GroupFailure::Busy) on wasm while a change holds the lock or waits for it."
            }
            Access::Write => {
                "[`GroupFailure::Busy`](crate::GroupFailure::Busy) on wasm while another call holds the lock."
            }
        };
        documented(&self.summary, &self.details, infallible, busy)
    }

    /// The documentation of the asynchronous form, whose summary says it is
    /// a future that waits for the lock, and which fails as the call's
    /// operation does, unless it is `infallible`, and when it is cancelled.
    pub(crate) fn asynchronous(&self, infallible: bool) -> String {
        // `parse` checked that the summary ends with a full stop.
        let summary = format!(
            "{}, as a future that waits for the lock.",
            self.summary.trim_end_matches('.')
        );
        documented(
            &summary,
            &self.details,
            infallible,
            "[`EranosError::Cancelled`].",
        )
    }

    /// The documentation of the read handle's form, which fails as the
    /// call's operation does, unless it is `infallible`, and once the handle
    /// is released.
    pub(crate) fn reader(&self, infallible: bool) -> String {
        documented(
            &self.summary,
            &self.details,
            infallible,
            "[`EranosError::Released`] after [`release`](Self::release).",
        )
    }
}

/// `summary` and `details` with an `# Errors` section: the operation's
/// refusal, unless it is `infallible`, and `failure`, the form's own.
fn documented(summary: &str, details: &[String], infallible: bool, failure: &str) -> String {
    let refusal = if infallible {
        ""
    } else {
        "The library's refusal, and "
    };
    let paragraphs = std::iter::once(summary.to_owned())
        .chain(details.iter().cloned())
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("{paragraphs}\n\n# Errors\n\n{refusal}{failure}")
}

/// `text` as a foreign binding's generated code shows it, where a Rust link
/// would read as raw markdown: every intra-doc link, `` [`Name`] `` or
/// `` [`Name`](path) ``, as its code span.
pub(crate) fn foreign(text: &str) -> String {
    let mut foreign = String::with_capacity(text.len());
    let mut rest = text;
    while let Some((before, after)) = rest.split_once("[`") {
        foreign.push_str(before);
        let Some((code, after)) = after.split_once("`]") else {
            foreign.push_str("[`");
            rest = after;
            continue;
        };
        foreign.push('`');
        foreign.push_str(code);
        foreign.push('`');
        rest = after
            .strip_prefix('(')
            .and_then(|link| link.split_once(')'))
            .map_or(after, |(_, after)| after);
    }
    foreign.push_str(rest);
    foreign
}

#[cfg(test)]
mod tests;
