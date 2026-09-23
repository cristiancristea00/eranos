//! Identifiers link entries across devices, files, and app screens. These
//! tests keep generated IDs unique and ordered by creation, keep the text
//! form a standard hyphenated UUID that reads back as the same ID, and keep
//! the byte form and its ordering stable for storage.

use super::*;

use std::collections::HashSet;
use std::iter;
use std::thread;

use proptest::prelude::*;

/// A UUID with a different value in every byte, so a byte-order mistake
/// shows.
const BYTES: [u8; 16] = [
    0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
];

/// The hyphenated text form of [`BYTES`].
const HYPHENATED: &str = "00112233-4455-6677-8899-aabbccddeeff";

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

            /// Other systems that check the UUID version would refuse the
            /// IDs, and creation order would be lost with a random version.
            #[test]
            fn generated_ids_are_version_7_uuids() {
                // Version nibble and RFC 9562 variant bits.
                let bytes = $name::generate().to_bytes();
                assert_eq!(bytes[6] >> 4, 7);
                assert_eq!(bytes[8] >> 6, 0b10);
            }

            /// Saved references to an entry would not read back.
            #[test]
            fn the_text_form_is_hyphenated_and_round_trips() {
                // A fixed ID.
                let id = $name::from_bytes(BYTES);
                assert_eq!(id.to_string(), HYPHENATED);
                assert_eq!(HYPHENATED.parse::<$name>(), Ok(id));
                // A generated ID.
                let generated = $name::generate();
                let text = generated.to_string();
                assert_eq!(text.len(), 36);
                assert_eq!(text.parse::<$name>(), Ok(generated));
            }

            /// IDs pasted or imported in another common UUID form would be
            /// refused.
            #[test]
            fn other_uuid_forms_parse_to_the_same_id() {
                let id = $name::from_bytes(BYTES);
                // Simple, braced, and URN forms.
                for text in [
                    "00112233445566778899aabbccddeeff",
                    "{00112233-4455-6677-8899-aabbccddeeff}",
                    "urn:uuid:00112233-4455-6677-8899-aabbccddeeff",
                ] {
                    assert_eq!(text.parse::<$name>(), Ok(id), "{text}");
                }
                // Uppercase hexadecimal digits.
                assert_eq!(
                    "00112233-4455-6677-8899-AABBCCDDEEFF".parse::<$name>(),
                    Ok(id)
                );
                // Surrounding whitespace.
                assert_eq!(format!(" \t{HYPHENATED}\n").parse::<$name>(), Ok(id));
            }

            /// A corrupted reference would be read as the ID of some other
            /// entry.
            #[test]
            fn garbage_is_rejected_with_invalid_id() {
                // Wrong length, digits, or separators.
                for text in [
                    "",
                    "   ",
                    "not-a-uuid",
                    "00112233-4455-6677-8899-aabbccddeef",
                    "00112233-4455-6677-8899-aabbccddeeff0",
                    "00112233-4455-6677-8899-aabbccddeefg",
                    "00112233_4455_6677_8899_aabbccddeeff",
                    "0011 2233-4455-6677-8899-aabbccddeeff",
                    "{00112233-4455-6677-8899-aabbccddeeff",
                ] {
                    assert_eq!(
                        text.parse::<$name>(),
                        Err(ValueError::InvalidId),
                        "{text:?}"
                    );
                }
            }

            /// IDs stored as 16 bytes would read back as different IDs.
            #[test]
            fn the_byte_form_round_trips() {
                // Fixed bytes.
                assert_eq!($name::from_bytes(BYTES).to_bytes(), BYTES);
                // A generated ID.
                let id = $name::generate();
                assert_eq!($name::from_bytes(id.to_bytes()), id);
            }

            /// Entries sorted by ID would not be sorted the way their stored
            /// bytes are.
            #[test]
            fn ordering_follows_the_bytes() {
                // The first byte decides before the last.
                let mut first_byte = [0; 16];
                first_byte[0] = 1;
                let mut last_byte = [0; 16];
                last_byte[15] = 0xff;
                assert!($name::from_bytes(last_byte) < $name::from_bytes(first_byte));
                // The smallest and the largest ID.
                assert!($name::from_bytes([0; 16]) < $name::from_bytes([0xff; 16]));
            }

            proptest! {
                /// Entries sorted by ID would disagree with the same entries
                /// sorted by their stored bytes.
                #[test]
                fn id_order_matches_byte_order(
                    first in any::<[u8; 16]>(),
                    second in any::<[u8; 16]>(),
                ) {
                    prop_assert_eq!(
                        $name::from_bytes(first).cmp(&$name::from_bytes(second)),
                        first.cmp(&second)
                    );
                }
            }
        }
    };
}

identifier_tests!(group_id, GroupId);
identifier_tests!(member_id, MemberId);
identifier_tests!(category_id, CategoryId);
identifier_tests!(expense_id, ExpenseId);
identifier_tests!(transfer_id, TransferId);
