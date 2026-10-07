//! Identifiers link entries across devices, files, and app screens. These
//! tests keep an identifier's text exactly as it was given, refuse the text
//! the rules exclude, keep ordering the byte order of the text, and keep
//! generated identifiers unique, ordered by creation, and lowercase
//! hyphenated UUID v7 text. Under the `sequential-ids` feature they keep the
//! counted identifiers in the same shape; the tests share the counter, so
//! they pin order, never a value.

use super::*;

use std::collections::HashSet;
use std::iter;
use std::mem;
use std::thread;
#[cfg(not(feature = "sequential-ids"))]
use std::time::{SystemTime, UNIX_EPOCH};

use proptest::prelude::*;

/// The text of the nil UUID, Uncategorised's identifier.
const NIL: &str = "00000000-0000-0000-0000-000000000000";

/// Text with no whitespace at either end, from a few letters, digits, a
/// hyphen, and characters of two, three, and four bytes, at most 57 bytes
/// long; the small alphabet makes prefixes and repeats common.
const VALID_TEXT: &str = "[ab0-9é€😀-]{1,7}( [ab0-9é€😀-]{1,7})?";

/// Text that is not an identifier, and the failure that refuses it.
#[derive(Debug)]
struct Refusal {
    text: String,
    error: IdError,
}

/// The refusal of `text` with `error`.
fn refusal(text: &str, error: IdError) -> Refusal {
    Refusal {
        text: text.to_owned(),
        error,
    }
}

/// The text of the UUID with these bytes, lowercase and hyphenated.
fn uuid_text(bytes: [u8; 16]) -> String {
    Uuid::from_bytes(bytes).hyphenated().to_string()
}

/// Declares the same tests for one identifier type, in a module named after
/// it.
macro_rules! identifier_tests {
    ($module:ident, $name:ident) => {
        mod $module {
            use super::*;

            /// Entries created in a row would collide or sort out of
            /// creation order.
            #[test]
            fn generated_ids_are_unique_and_strictly_increasing() {
                let generate_many = || {
                    iter::repeat_with($name::generate)
                        .take(1_000)
                        .collect::<Vec<_>>()
                };
                // One thread.
                let ids = generate_many();
                assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
                // Several threads at once.
                let workers: Vec<_> = iter::repeat_with(|| thread::spawn(generate_many))
                    .take(4)
                    .collect();
                let mut seen = HashSet::new();
                for worker in workers {
                    let ids = worker.join().unwrap();
                    assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
                    assert!(ids.into_iter().all(|id| seen.insert(id)));
                }
            }

            /// Other systems that read the identifiers as UUIDs would refuse
            /// them, and creation order would be lost with a random version
            /// or with uppercase digits, which sort before lowercase ones.
            #[test]
            fn generated_ids_are_lowercase_hyphenated_uuid_v7_text() {
                let id = $name::generate();
                let text = id.as_str();
                assert_eq!(text.len(), 36);
                assert!(
                    text.chars()
                        .all(|character| '-' == character || character.is_ascii_hexdigit()),
                    "{text}"
                );
                assert_eq!(text.to_ascii_lowercase(), text);
                // The version digit, and the RFC 9562 variant.
                assert_eq!(text.chars().nth(14), Some('7'));
                assert!(
                    text.chars()
                        .nth(19)
                        .is_some_and(|variant| "89ab".contains(variant)),
                    "{text}"
                );
                assert_eq!(Uuid::try_parse(text).unwrap().get_version_num(), 7);
            }

            /// Names, places, and other text that callers choose would come
            /// back changed: trimmed, folded to one case, or cut.
            #[test]
            fn new_keeps_the_text_as_it_is() {
                for text in [
                    "a",
                    "Ană Popescu",
                    "lisbon-2026",
                    "ANA",
                    "😀",
                    "a b\u{a0}c",
                ] {
                    assert_eq!($name::new(text).unwrap().as_str(), text);
                }
                // Exactly 64 bytes, in one-byte and in four-byte characters.
                for text in ["a".repeat(64), "😀".repeat(16)] {
                    assert_eq!($name::new(&text).unwrap().as_str(), text);
                }
            }

            /// Text that a file, a screen, or another system could not show or
            /// compare as written would become an identifier.
            #[test]
            fn new_refuses_what_the_rules_exclude() {
                let refusals = [
                    refusal("", IdError::Empty),
                    refusal(&"a".repeat(65), IdError::TooLong),
                    // The limit counts bytes: 33 characters of two bytes.
                    refusal(&"é".repeat(33), IdError::TooLong),
                    refusal("a\tb", IdError::ControlCharacter),
                    refusal("a\nb", IdError::ControlCharacter),
                    refusal("\u{0}", IdError::ControlCharacter),
                    refusal("a\u{7f}", IdError::ControlCharacter),
                    refusal(" ana", IdError::SurroundingWhitespace),
                    refusal("ana ", IdError::SurroundingWhitespace),
                    refusal("\u{a0}ana", IdError::SurroundingWhitespace),
                    refusal("ana\u{3000}", IdError::SurroundingWhitespace),
                    refusal("   ", IdError::SurroundingWhitespace),
                ];
                for Refusal { text, error } in refusals {
                    assert_eq!($name::new(&text), Err(error), "{text:?}");
                    assert_eq!(text.parse::<$name>(), Err(error), "{text:?}");
                }
            }

            /// The apps and the bindings report the first failure; another
            /// order would report another failure for the same text.
            #[test]
            fn new_checks_in_the_order_of_the_failures() {
                // Too long and full of control characters.
                assert_eq!($name::new(&"\t".repeat(65)), Err(IdError::TooLong));
                // A control character that is also whitespace, at the end.
                assert_eq!($name::new("\u{7} "), Err(IdError::ControlCharacter));
                assert_eq!($name::new("a\t"), Err(IdError::ControlCharacter));
            }

            /// Saved references to an entry would not read back as the same
            /// entry.
            #[test]
            fn the_text_form_round_trips() {
                for id in [$name::new("Ană Popescu").unwrap(), $name::generate()] {
                    let text = id.to_string();
                    assert_eq!(text, id.as_str());
                    assert_eq!(text.parse::<$name>(), Ok(id));
                    assert_eq!($name::new(id.as_str()), Ok(id));
                }
            }

            /// Two spellings of one UUID, or one name in two cases, would be
            /// taken for the same entry, though a file or another device
            /// tells them apart.
            #[test]
            fn identifiers_compare_byte_for_byte() {
                let lower = $name::new("0199a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b").unwrap();
                let upper = $name::new("0199A1B2-C3D4-7E5F-8A6B-7C8D9E0F1A2B").unwrap();
                assert_ne!(lower, upper);
                // Uppercase letters come before lowercase ones in UTF-8.
                assert!(upper < lower);
                assert_ne!($name::new("ana").unwrap(), $name::new("Ana").unwrap());
            }

            /// Lists keyed by identifier would interleave an entry's
            /// identifier with longer ones that start with it.
            #[test]
            fn a_prefix_sorts_first() {
                let id = |text: &str| $name::new(text).unwrap();
                assert!(id("an") < id("ana"));
                assert!(id("ana") < id("ana-2"));
                assert!(id("ana") < id("anb"));
                assert!(id(&"a".repeat(63)) < id(&"a".repeat(64)));
            }

            /// An identifier is a fixed-size `Copy` value; a larger one
            /// would make every map key and every pair of members larger.
            #[test]
            fn an_identifier_is_64_bytes() {
                assert_eq!(mem::size_of::<$name>(), 64);
            }

            /// Test failures and logs would print the buffer's zero bytes,
            /// or an identifier no reader could tell from another type's.
            #[test]
            fn debug_names_the_type_and_quotes_the_text() {
                assert_eq!(
                    format!("{:?}", $name::new("ana").unwrap()),
                    concat!(stringify!($name), "(\"ana\")")
                );
            }

            /// Saved groups would not read back, or files written by earlier
            /// releases, whose identifiers are UUID text, would be refused.
            #[cfg(feature = "serde")]
            #[test]
            fn the_serialised_form_is_the_text() {
                let id = $name::new("Ană Popescu").unwrap();
                let json = serde_json::to_string(&id).unwrap();
                assert_eq!(json, "\"Ană Popescu\"");
                assert_eq!(serde_json::from_str::<$name>(&json).unwrap(), id);
                // UUID text reads as it is.
                let uuid = "0199a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
                assert_eq!(
                    serde_json::from_str::<$name>(&format!("\"{uuid}\""))
                        .unwrap()
                        .as_str(),
                    uuid
                );
                // Refused text, and a value that is not text.
                let refused = serde_json::from_str::<$name>("\" ana\"").unwrap_err();
                assert!(
                    refused
                        .to_string()
                        .starts_with("an identifier cannot start or end with whitespace"),
                    "{refused}"
                );
                let number = serde_json::from_str::<$name>("7").unwrap_err();
                assert!(number.to_string().contains("expected an identifier"), "{number}");
            }

            proptest! {
                /// Entries sorted by identifier would disagree with the same
                /// entries sorted by their text, in another binding or in a
                /// file.
                #[test]
                fn id_order_matches_the_byte_order_of_the_text(
                    first in VALID_TEXT,
                    second in VALID_TEXT,
                ) {
                    let (first_id, second_id) = ($name::new(&first).unwrap(), $name::new(&second).unwrap());
                    prop_assert_eq!(first_id.cmp(&second_id), first.as_bytes().cmp(second.as_bytes()));
                    prop_assert_eq!(first_id == second_id, first == second);
                }

                /// Generated identifiers would not sort in creation order: the
                /// text of UUIDs must order as their bytes, whose leading
                /// timestamp orders them.
                #[test]
                fn uuid_text_orders_as_the_uuid_bytes(
                    first in any::<[u8; 16]>(),
                    second in any::<[u8; 16]>(),
                ) {
                    prop_assert_eq!(
                        $name::new(&uuid_text(first)).unwrap().cmp(&$name::new(&uuid_text(second)).unwrap()),
                        first.cmp(&second)
                    );
                }
            }
        }
    };
}

/// Expenses saved without a category, and the category every group has
/// built in, would not match the constant.
#[test]
fn uncategorised_is_the_nil_uuid_text() {
    assert_eq!(CategoryId::UNCATEGORISED.as_str(), NIL);
    assert_eq!(CategoryId::new(NIL), Ok(CategoryId::UNCATEGORISED));
    // It sorts before every UUID text, and so before every generated
    // identifier.
    assert_eq!(uuid_text([0; 16]), NIL);
    assert!(CategoryId::UNCATEGORISED < CategoryId::generate());
}

/// A group's categories and the ledger's category totals would list
/// Uncategorised after every category whose identifier sorts before the nil
/// UUID text, such as `-food`.
#[test]
fn a_category_key_puts_uncategorised_first_and_the_others_in_identifier_order() {
    let food = CategoryId::new("-food").unwrap();
    let fuel = CategoryId::new("fuel").unwrap();
    assert!(CategoryId::UNCATEGORISED > food);
    let mut keys = [fuel, CategoryId::UNCATEGORISED, food].map(CategoryKey::new);
    keys.sort();
    assert_eq!(
        keys.map(CategoryKey::id),
        [CategoryId::UNCATEGORISED, food, fuel]
    );
    assert!(CategoryKey::new(CategoryId::UNCATEGORISED) < CategoryKey::new(food));
    assert!(CategoryKey::new(fuel) > CategoryKey::new(food));
    assert_eq!(
        CategoryKey::new(food).cmp(&CategoryKey::new(food)),
        Ordering::Equal
    );
}

/// A regression witness: with uuid's shared v7 context alone, a thread
/// paused between reading the clock and taking uuid's lock across a second
/// boundary pushed the context almost a second ahead, and the identifiers
/// generated once the clock caught up sorted before earlier ones, in every
/// thread at once (1,612 times in 12 seconds with 64 threads). Sixty-four
/// threads, more than there are cores, generate across the next second
/// boundary, and each must see its identifiers strictly increase.
#[cfg(not(feature = "sequential-ids"))]
#[test]
fn identifiers_stay_in_creation_order_across_a_second_boundary() {
    let second = || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    };
    let started = second();
    let workers: Vec<_> = iter::repeat_with(|| {
        thread::spawn(move || {
            let mut previous = MemberId::generate();
            // The clock is read once per batch, so the threads spend their
            // time contending for the generator.
            while second() == started {
                for _ in 0..1_000 {
                    let next = MemberId::generate();
                    assert!(previous < next, "{previous} came before {next}");
                    previous = next;
                }
            }
        })
    })
    .take(64)
    .collect();
    for worker in workers {
        worker.join().unwrap();
    }
}

/// Examples that print identifiers would differ from run to run, or a
/// counted identifier would not be told from one generated from the clock.
#[cfg(feature = "sequential-ids")]
#[test]
fn sequential_ids_count_up_with_every_random_bit_zero() {
    // Two types share the counter, in the order this thread asked.
    let first = ExpenseGroupId::generate();
    let second = MemberId::generate();
    assert!(first.as_str() < second.as_str());
    let uuid_bytes = |text: &str| Uuid::try_parse(text).unwrap().into_bytes();
    let (first, second) = (uuid_bytes(first.as_str()), uuid_bytes(second.as_str()));
    assert!(first[..6] < second[..6]);
    // Only the version and variant bits are set outside the counter.
    for bytes in [first, second] {
        assert_eq!(bytes[6], 0x70);
        assert_eq!(bytes[7], 0);
        assert_eq!(bytes[8], 0x80);
        assert_eq!(bytes[9..], [0; 7]);
    }
    // The counter is far below the clock, so a counted identifier sorts
    // before every UUID v7 generated from the time.
    assert!(
        MemberId::new(&uuid_text(second)).unwrap()
            < MemberId::new(&uuid_text(Uuid::now_v7().into_bytes())).unwrap()
    );
}

identifier_tests!(group_id, ExpenseGroupId);
identifier_tests!(member_id, MemberId);
identifier_tests!(category_id, CategoryId);
identifier_tests!(expense_id, ExpenseId);
identifier_tests!(transfer_id, TransferId);
