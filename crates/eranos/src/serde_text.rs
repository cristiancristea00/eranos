//! Deserialisation of the types whose serialised form is their text form.

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::Deserializer;
use serde::de::{Error, Visitor};

/// Deserialises a string and parses it with `T`'s [`FromStr`], so the value
/// passes the same validation as when it is parsed directly.
pub(crate) fn deserialise<'de, D, T>(deserialiser: D, expected: &'static str) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    deserialiser.deserialize_str(TextVisitor {
        expected,
        target: PhantomData,
    })
}

struct TextVisitor<T> {
    expected: &'static str,
    target: PhantomData<T>,
}

impl<T> Visitor<'_> for TextVisitor<T>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.expected)
    }

    fn visit_str<E: Error>(self, text: &str) -> Result<T, E> {
        text.parse().map_err(E::custom)
    }
}
