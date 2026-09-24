//! ISO 4217 currencies.
//!
//! The codes, minor units, and names come from `iso4217-static`, generated
//! from SIX Group's List One and pinned exactly; its type stays private.

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use iso4217_static::Currency as IsoCurrency;

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
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Currency {
    /// The minor unit of `iso`: `lookup` refuses a code without one, so
    /// `decimal_places` needs no fallback.
    decimal_places: u8,
    iso: IsoCurrency,
}

// Written out because the fields are in alphabetical order: a derived
// ordering would compare the decimal places first, and the crate's own
// ordering is by numeric code.
impl Ord for Currency {
    fn cmp(&self, other: &Self) -> Ordering {
        self.code().cmp(other.code())
    }
}

impl PartialOrd for Currency {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Declares one public constant per code, looked up in `iso4217-static` when
/// the crate is compiled, and `all`, which lists them in the order given.
macro_rules! currencies {
    ($($code:ident)+) => {
        impl Currency {
            $(
                #[doc = concat!("The currency with ISO 4217 code `", stringify!($code), "`.")]
                pub const $code: Self = Self::lookup(stringify!($code)).expect(concat!(
                    "iso4217-static lists ",
                    stringify!($code),
                    " with a minor unit"
                ));
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

impl Currency {
    /// Looks up an ISO 4217 code, ignoring ASCII case and surrounding
    /// whitespace.
    ///
    /// # Errors
    ///
    /// [`ValueError::UnknownCurrency`] when the code is not an active ISO 4217
    /// code with a minor unit.
    pub fn from_code(code: &str) -> Result<Self, ValueError> {
        Self::lookup(&code.trim().to_ascii_uppercase()).ok_or(ValueError::UnknownCurrency)
    }

    /// The currency whose code is exactly `code`, three uppercase ASCII
    /// letters, if `iso4217-static` lists it with a minor unit. The constants
    /// call it when the crate is compiled, so a code the pinned release lacks
    /// fails the build.
    const fn lookup(code: &str) -> Option<Self> {
        let Ok(iso) = IsoCurrency::from_str_slice(code) else {
            return None;
        };
        // Precious metals, drawing rights, and testing codes have no minor
        // unit and are not currencies here.
        let Some(decimal_places) = iso.minor_unit() else {
            return None;
        };
        Some(Self {
            decimal_places,
            iso,
        })
    }

    /// The number of decimal places of the minor unit: 2 for `EUR` (cents),
    /// 0 for `JPY`, 3 for `BHD`.
    #[must_use]
    pub const fn decimal_places(self) -> u8 {
        self.decimal_places
    }

    /// `10^decimal_places`: how many minor units make one major unit.
    pub(crate) fn minor_units_per_major(self) -> i64 {
        10_i64.pow(u32::from(self.decimal_places()))
    }

    /// The three-letter code, such as `"EUR"`.
    #[must_use]
    pub const fn code(self) -> &'static str {
        self.iso.as_str()
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
    fn serialize<S: serde::Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
        serialiser.serialize_str(self.code())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Currency {
    fn deserialize<D: serde::Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        crate::serde_text::deserialise(deserialiser, "an ISO 4217 currency code")
    }
}

#[cfg(test)]
mod tests;
