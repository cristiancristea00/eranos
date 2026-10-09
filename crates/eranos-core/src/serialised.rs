//! The macros that write the serde impls of the values whose serialised form
//! is their text, and of the values checked by a constructor on the way in.
//! Each impl is compiled with the `serde` feature of the crate that invokes
//! the macro, which enables this crate's.

/// Writes the serde impls of `$name`, whose serialised form is its text.
///
/// The value serialises as the text its method `$text` returns, or as its
/// `Display` text when no method is named, and deserialises through its
/// `FromStr` with `serde_text::deserialise`, so a value passes the same
/// checks as when it is parsed. `$expected` says what a deserialiser
/// expected when the input is not a string.
#[macro_export]
macro_rules! serde_text {
    ($name:ty, $expected:literal) => {
        #[cfg(feature = "serde")]
        impl ::serde::Serialize for $name {
            fn serialize<S: ::serde::Serializer>(
                &self,
                serialiser: S,
            ) -> ::core::result::Result<S::Ok, S::Error> {
                serialiser.collect_str(self)
            }
        }

        $crate::serde_text!(@deserialise $name, $expected);
    };
    ($name:ty, $text:ident, $expected:literal) => {
        #[cfg(feature = "serde")]
        impl ::serde::Serialize for $name {
            fn serialize<S: ::serde::Serializer>(
                &self,
                serialiser: S,
            ) -> ::core::result::Result<S::Ok, S::Error> {
                serialiser.serialize_str(self.$text())
            }
        }

        $crate::serde_text!(@deserialise $name, $expected);
    };
    (@deserialise $name:ty, $expected:literal) => {
        #[cfg(feature = "serde")]
        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D: ::serde::Deserializer<'de>>(
                deserialiser: D,
            ) -> ::core::result::Result<Self, D::Error> {
                $crate::serde_text::deserialise(deserialiser, $expected)
            }
        }
    };
}

/// Writes the `Deserialize` impl of `$name`, a type with invariants.
///
/// It deserialises `$form`, the type's serialised form, and builds the value
/// with `$build`, its checking constructor, whose refusal becomes the
/// deserialiser's error, so a value passes the same checks as when it is
/// built.
///
/// The impl is written out rather than derived with `serde(try_from)`, whose
/// `TryFrom` impl from the form would be public, on the facade's page for
/// the type.
#[macro_export]
macro_rules! deserialise_through {
    ($name:ty, $form:ty, $build:expr) => {
        #[cfg(feature = "serde")]
        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D: ::serde::Deserializer<'de>>(
                deserialiser: D,
            ) -> ::core::result::Result<Self, D::Error> {
                let form = <$form as ::serde::Deserialize>::deserialize(deserialiser)?;
                ($build)(form).map_err(::serde::de::Error::custom)
            }
        }
    };
}
