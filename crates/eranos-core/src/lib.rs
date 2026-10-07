//! Identifiers for groups and their entries, and the rules for the text
//! every entry holds.
//!
//! An identifier is text, stored as it is given: at most 64 bytes of UTF-8,
//! not empty, without control characters, and without whitespace at either
//! end. Two identifiers are equal when their bytes are, so case matters, and
//! identifiers order by their bytes. An identifier is the one an app gives
//! an entity, such as the key of its own store, or a generated one. A
//! generated identifier is a UUID v7 written as lowercase hyphenated text:
//! unique across devices, so entries created offline on two phones never
//! collide, and in creation order, since such text orders as the UUIDs'
//! timestamps do. With the `sequential-ids` feature, the identifiers of a
//! process count up from one instead, so a program that builds the same
//! entries in the same order gets the same identifiers on every run. The
//! feature is for examples and tests: two devices would generate the same
//! identifiers.
//!
//! The `text` module trims text and keeps blank optional text absent, and
//! `serde_text` (feature `serde`) reads the values whose serialised form is
//! their text form. [`cooperative`] holds the futures of the library's long
//! computations, which yield at points of their own choosing, and their two
//! drivers.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use. The facade does not re-export [`FromIdError`], through
//! which a crate that tells every failure apart converts one without the
//! wildcard arm a `match` outside this crate would need, nor
//! [`CategoryKey`], the map key with which a group and its ledger list
//! Uncategorised first.
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

pub mod cooperative;
mod error;
#[cfg(feature = "serde")]
pub mod serde_text;
pub mod text;

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
#[cfg(feature = "sequential-ids")]
use std::sync::atomic::{self, AtomicU64};
#[cfg(not(feature = "sequential-ids"))]
use std::sync::{Mutex, PoisonError};

#[cfg(feature = "sequential-ids")]
use uuid::Builder;
use uuid::Uuid;

pub use error::{FromIdError, IdError};

/// The most bytes of UTF-8 an identifier holds.
const MAX_LENGTH: usize = 64;

/// Held while an identifier is generated. uuid's shared v7 context reads the
/// clock before it takes its own lock, and when a thread is paused between
/// the two across a second boundary, its stale reading pushes the context's
/// timestamp almost a second ahead, so the identifiers generated once the
/// clock catches up sort before the earlier ones (uuid 1.26.1). Reading the
/// clock under this lock keeps every identifier eranos generates in creation
/// order. It guards no data, so a thread that panicked while holding it
/// leaves nothing to recover.
#[cfg(not(feature = "sequential-ids"))]
static GENERATION: Mutex<()> = Mutex::new(());

/// A new UUID v7, generated under [`GENERATION`].
#[cfg(not(feature = "sequential-ids"))]
fn generate_uuid() -> Uuid {
    let generating = GENERATION.lock().unwrap_or_else(PoisonError::into_inner);
    let uuid = Uuid::now_v7();
    drop(generating);
    uuid
}

/// The counter the next sequential identifier holds in its timestamp field,
/// shared by every identifier type. The field has 48 bits, so a process
/// that generates fewer than 2⁴⁸ identifiers never repeats one; the first,
/// `00000000-0001-7000-8000-000000000000`, sorts after the nil UUID text of
/// [`CategoryId::UNCATEGORISED`].
#[cfg(feature = "sequential-ids")]
static SEQUENCE: AtomicU64 = AtomicU64::new(1);

/// The next sequential UUID v7: the counter in the timestamp field, and zero
/// in every random bit. Every change to one atomic falls in a single order,
/// so each thread's increments return increasing values without a stronger
/// ordering than `Relaxed`.
#[cfg(feature = "sequential-ids")]
fn generate_uuid() -> Uuid {
    Builder::from_unix_timestamp_millis(SEQUENCE.fetch_add(1, atomic::Ordering::Relaxed), &[0; 10])
        .into_uuid()
}

/// The text of an identifier: its UTF-8 bytes, then zero bytes up to
/// [`MAX_LENGTH`].
///
/// No valid text holds U+0000, a control character, so the text ends at the
/// first zero byte, or fills the buffer. The derived comparisons compare the
/// buffers byte by byte, and so order them as their texts: where one text
/// is a prefix of another, the shorter has a zero byte where the longer has
/// a non-zero one, so it sorts first.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct IdentifierText([u8; MAX_LENGTH]);

impl IdentifierText {
    /// The text of a new identifier: a UUID v7, as lowercase hyphenated
    /// text.
    fn generate() -> Self {
        from_valid(
            generate_uuid()
                .hyphenated()
                .encode_lower(&mut Uuid::encode_buffer()),
        )
    }

    /// The identifier text `text`, as it is.
    ///
    /// # Errors
    ///
    /// The first failure, in the order of [`IdError`]'s variants.
    fn new(text: &str) -> Result<Self, IdError> {
        if text.is_empty() {
            return Err(IdError::Empty);
        }
        if MAX_LENGTH < text.len() {
            return Err(IdError::TooLong);
        }
        if text.chars().any(char::is_control) {
            return Err(IdError::ControlCharacter);
        }
        if text.starts_with(char::is_whitespace) || text.ends_with(char::is_whitespace) {
            return Err(IdError::SurroundingWhitespace);
        }
        Ok(from_valid(text))
    }

    /// The text.
    fn as_str(&self) -> &str {
        let Self(bytes) = self;
        let length = bytes
            .iter()
            .position(|&byte| 0 == byte)
            .unwrap_or(MAX_LENGTH);
        // The bytes before `length` are the UTF-8 of text that `new` checked
        // or that generation wrote, so neither fallback is taken.
        bytes
            .get(..length)
            .and_then(|text| std::str::from_utf8(text).ok())
            .unwrap_or_default()
    }
}

/// The buffer of `text`, which holds at most [`MAX_LENGTH`] bytes and no
/// U+0000: `new` checked it, or generation or a constant wrote it.
const fn from_valid(text: &str) -> IdentifierText {
    let mut bytes = [0; MAX_LENGTH];
    // `text` holds at most MAX_LENGTH bytes, so the split is within the
    // buffer, and its head has the length of `text`.
    let (head, _) = bytes.split_at_mut(text.len());
    head.copy_from_slice(text.as_bytes());
    IdentifierText(bytes)
}

macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        ///
        /// The text form is the identifier's text, as it was given or
        /// generated, and ordering follows its bytes: creation order for
        /// generated identifiers.
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("use eranos::id::", stringify!($name), ";")]
        ///
        #[doc = concat!("let id = ", stringify!($name), "::new(\"lisbon-2026\")?;")]
        /// assert_eq!(id.as_str(), "lisbon-2026");
        #[doc = concat!("assert_eq!(id.to_string().parse::<", stringify!($name), ">()?, id);")]
        ///
        /// // A generated identifier is a UUID v7 as lowercase hyphenated text.
        #[doc = concat!("assert_eq!(", stringify!($name), "::generate().as_str().len(), 36);")]
        /// # Ok::<(), eranos::error::IdError>(())
        /// ```
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(IdentifierText);

        impl $name {
            /// Generates a new identifier, a UUID v7 as lowercase hyphenated
            /// text, from the current time and random bits or, with the
            /// `sequential-ids` feature, from the next value of a counter that
            /// every identifier type shares.
            ///
            /// Identifiers generated by one process are strictly increasing.
            #[must_use]
            pub fn generate() -> Self {
                Self(IdentifierText::generate())
            }

            /// Returns the identifier `text`, as it is: it is not trimmed,
            /// and its case is kept.
            ///
            /// # Errors
            ///
            /// - [`IdError::Empty`] when the text is empty.
            /// - [`IdError::TooLong`] when it is longer than 64 bytes of
            ///   UTF-8.
            /// - [`IdError::ControlCharacter`] when it holds a control
            ///   character, such as a tab or a line break.
            /// - [`IdError::SurroundingWhitespace`] when it starts or ends with
            ///   whitespace.
            pub fn new(text: &str) -> Result<Self, IdError> {
                IdentifierText::new(text).map(Self)
            }

            /// The identifier's text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_tuple(stringify!($name))
                    .field(&self.as_str())
                    .finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = IdError;

            /// Reads the identifier `text` as [`new`](Self::new) does.
            fn from_str(text: &str) -> Result<Self, Self::Err> {
                Self::new(text)
            }
        }

        #[cfg(feature = "serde")]
        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
                serialiser.serialize_str(self.as_str())
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
                serde_text::deserialise(deserialiser, "an identifier")
            }
        }
    };
}

define_id! {
    /// Identifies a group.
    ExpenseGroupId
}

define_id! {
    /// Identifies a member of a group.
    MemberId
}

define_id! {
    /// Identifies a category of a group.
    CategoryId
}

impl CategoryId {
    /// The built-in category every expense has until it is given another,
    /// the same in every group: the text of the nil UUID,
    /// `00000000-0000-0000-0000-000000000000`, which sorts before every
    /// generated identifier. A group lists it before every other category,
    /// whatever their identifiers. The apps show its name in the user's
    /// language.
    pub const UNCATEGORISED: Self = Self(from_valid("00000000-0000-0000-0000-000000000000"));
}

/// A map key that lists categories as a group does.
///
/// [`CategoryId::UNCATEGORISED`] comes first, then every other category in
/// identifier order, which identifier order alone does not give: text such
/// as `-food` sorts before the nil UUID text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CategoryKey(CategoryId);

impl CategoryKey {
    /// The key of the category `id`.
    #[must_use]
    pub const fn new(id: CategoryId) -> Self {
        Self(id)
    }

    /// The category.
    #[must_use]
    pub const fn id(self) -> CategoryId {
        self.0
    }
}

// Written out: Uncategorised before every other category, and the others in
// identifier order.
impl Ord for CategoryKey {
    fn cmp(&self, other: &Self) -> Ordering {
        (CategoryId::UNCATEGORISED != self.0)
            .cmp(&(CategoryId::UNCATEGORISED != other.0))
            .then_with(|| self.0.cmp(&other.0))
    }
}

impl PartialOrd for CategoryKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

define_id! {
    /// Identifies an expense of a group.
    ExpenseId
}

define_id! {
    /// Identifies a transfer (repayment) of a group.
    TransferId
}

#[cfg(test)]
mod tests;
