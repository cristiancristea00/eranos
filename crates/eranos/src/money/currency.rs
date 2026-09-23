//! ISO 4217 currencies.

use std::fmt;
use std::str::FromStr;

use crate::error::ValueError;

/// An active ISO 4217 currency, such as [`Currency::EUR`].
///
/// Every active code with a minor unit has a constant; precious metals,
/// special drawing rights, and testing codes (minor unit "N.A.") do not. The
/// text form is the three-letter code, and ordering is by code.
///
/// # Examples
///
/// ```
/// use eranos::Currency;
///
/// let currency = Currency::from_code("ron")?;
/// assert_eq!(currency, Currency::RON);
/// assert_eq!(currency.code(), "RON");
/// assert_eq!(Currency::JPY.decimal_places(), 0);
/// assert_eq!(Currency::BHD.decimal_places(), 3);
/// # Ok::<(), eranos::error::ValueError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Currency(iso4217::Code);

/// Declares every currency: a private enum of the codes (whose order is the
/// ordering of `Currency`), a lookup table sorted by code, and one public
/// constant per code.
macro_rules! currencies {
    ($($code:ident $decimal_places:literal $name:literal;)+) => {
        /// Every code, in code order.
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[expect(clippy::upper_case_acronyms, reason = "the variants are the ISO 4217 codes")]
        pub(super) enum Code {
            $($code),+
        }

        impl Code {
            pub(super) const fn text(self) -> &'static str {
                match self {
                    $(Self::$code => stringify!($code)),+
                }
            }

            pub(super) const fn decimal_places(self) -> u8 {
                match self {
                    $(Self::$code => $decimal_places),+
                }
            }
        }

        /// Every currency with its code, sorted by code for binary search.
        pub(super) const TABLE: &[(&str, Currency)] = &[$((stringify!($code), Currency::$code)),+];

        impl Currency {
            $(
                #[doc = concat!(
                    "The ", $name, " (`", stringify!($code), "`), with ",
                    stringify!($decimal_places), " decimal places."
                )]
                pub const $code: Self = Self(Code::$code);
            )+

            /// Every currency, ordered by code.
            ///
            /// A function rather than a constant: `ALL` is the Albanian lek.
            #[must_use]
            pub const fn all() -> &'static [Self] {
                const EVERY_CURRENCY: &[Currency] = &[$(Currency::$code),+];
                EVERY_CURRENCY
            }
        }
    };
}

mod iso4217;

use iso4217::TABLE;

impl Currency {
    /// Looks up an ISO 4217 code, ignoring ASCII case and surrounding
    /// whitespace.
    ///
    /// # Errors
    ///
    /// [`ValueError::UnknownCurrency`] when the code is not an active ISO 4217
    /// code with a minor unit.
    pub fn from_code(code: &str) -> Result<Self, ValueError> {
        let code = code.trim();
        let [first, second, third] = *code.as_bytes() else {
            return Err(ValueError::UnknownCurrency);
        };
        let upper = [
            first.to_ascii_uppercase(),
            second.to_ascii_uppercase(),
            third.to_ascii_uppercase(),
        ];
        TABLE
            .binary_search_by(|(candidate, _)| candidate.as_bytes().cmp(&upper))
            .ok()
            .and_then(|position| TABLE.get(position))
            .map(|&(_, currency)| currency)
            .ok_or(ValueError::UnknownCurrency)
    }

    /// The three-letter code, such as `"EUR"`.
    #[must_use]
    pub const fn code(self) -> &'static str {
        self.0.text()
    }

    /// The number of decimal places of the minor unit: 2 for `EUR` (cents),
    /// 0 for `JPY`, 3 for `BHD`.
    #[must_use]
    pub const fn decimal_places(self) -> u8 {
        self.0.decimal_places()
    }

    /// `10^decimal_places`: how many minor units make one major unit.
    pub(crate) fn minor_units_per_major(self) -> i64 {
        10_i64.pow(u32::from(self.decimal_places()))
    }
}

impl fmt::Debug for Currency {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl FromStr for Currency {
    type Err = ValueError;

    fn from_str(code: &str) -> Result<Self, Self::Err> {
        Self::from_code(code)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Currency {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.code())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Currency {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        crate::serde_text::deserialize(deserializer, "an ISO 4217 currency code")
    }
}

#[cfg(test)]
mod tests;
