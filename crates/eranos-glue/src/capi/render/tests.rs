//! The text of a generated file before rustfmt lays it out. Documentation
//! left as attributes, a doc comment wider than rustfmt's width, or two
//! items without a line between them would commit a file that reads as
//! machine output, and text that did not parse back into the same code
//! would fail the freshness test at once.

use quote::quote;

use super::*;
use crate::capi::compare;

/// A file of every kind of item the generator writes, and of one it does
/// not.
fn file() -> File {
    syn::parse2(quote! {
        #![doc = "A file."]
        #![recursion_limit = "256"]

        #[doc = "A module."]
        pub mod ffi {
            use crate::first::First;
            use crate::second::Second;

            #[doc = "A record."]
            #[derive(Debug)]
            pub struct Record(pub(crate) i32);

            #[doc = "Its methods, whose summary runs long enough to wrap past the width of the page once indented."]
            #[doc = ""]
            #[doc = "A second paragraph."]
            impl Record {
                #[doc = "Its value."]
                #[must_use]
                #[doc = "Always there."]
                pub const fn value(&self) -> i32 {
                    self.0
                }

                const LIMIT: i32 = 1;
            }

            #[doc = "A choice."]
            pub enum Choice {
                #[doc = "The first."]
                First,
                #[doc(hidden)]
                Second,
            }

            const TOP: i32 = 1;
        }
    })
    .unwrap()
}

/// Every run of documentation reads as doc comments, inner or outer as its
/// attributes are, at its item's depth and in its place among the other
/// attributes; items are a line apart but for two imports; and the text
/// parses back into the file's code.
#[test]
fn a_file_reads_as_doc_comments_and_items_a_line_apart() {
    let file = file();
    let text = text(&file);
    let lines = text.lines().collect::<Vec<_>>();
    assert_eq!(lines.first(), Some(&NOTICE[0]));
    assert_eq!(lines.get(1), Some(&NOTICE[1]));
    assert_eq!(lines.get(2), Some(&""));
    assert_eq!(lines.get(3), Some(&"//! A file."));
    assert!(lines.contains(&"/// A module."));
    assert!(lines.contains(&"    /// A record."));
    assert!(lines.contains(&"        /// Its value."));
    assert!(lines.contains(&"        /// Always there."));
    assert!(lines.contains(&"        /// The first."));
    assert!(lines.contains(&"    ///"));
    let imports = lines
        .iter()
        .position(|line| line.starts_with("    use crate :: first"))
        .unwrap();
    assert!(lines[imports + 1].starts_with("    use crate :: second"));
    assert_eq!(lines[imports + 2], "");
    let summary = lines
        .iter()
        .position(|line| line.starts_with("    /// Its methods"))
        .unwrap();
    assert_eq!(
        lines[summary],
        "    /// Its methods, whose summary runs long enough to wrap past the width of the page once"
    );
    assert_eq!(lines[summary + 1], "    /// indented.");
    assert!(lines.iter().all(|line| line.chars().count() <= WIDTH));
    assert!(text.contains("# [doc (hidden)]"));
    assert!(text.contains("# [must_use]"));
    assert!(compare::same(&text, &file).unwrap());
}

/// A paragraph wraps at the last word that fits, and a word wider than the
/// width has a line of its own.
#[test]
fn a_paragraph_wraps_at_the_last_word_that_fits() {
    assert_eq!(wrap("one two three", 7), ["one two", "three"]);
    assert_eq!(wrap("one two", 7), ["one two"]);
    assert_eq!(wrap("extraordinary no", 7), ["extraordinary", "no"]);
    assert_eq!(wrap("", 7), Vec::<String>::new());
}

/// Only a documentation attribute of text reads as a doc comment.
#[test]
fn only_documentation_of_text_is_a_doc_comment() {
    let attributes = syn::parse2::<syn::ItemStruct>(quote! {
        #[doc = "Text."]
        #[doc(hidden)]
        #[doc = 1]
        #[derive(Debug)]
        struct Record;
    })
    .unwrap()
    .attrs;
    assert_eq!(
        attributes.iter().map(doc_text).collect::<Vec<_>>(),
        [Some("Text.".to_owned()), None, None, None]
    );
}
