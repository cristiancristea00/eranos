//! Quantities: how many units a line of a bill counts, a whole number or a
//! decimal with four places, stored as an exact integer.

use std::fmt;
use std::str::FromStr;

use crate::decimal;
use crate::error::MoneyError;

/// The decimal places of a quantity, which is stored in ten-thousandths.
const PLACES: u8 = 4;

/// The ten-thousandths in one unit.
const SCALE: u64 = 10_000;

/// The most ten-thousandths a quantity holds: those of `i64::MAX`, so that
/// every quantity crosses the bindings as a signed 64-bit integer.
const MAX: u64 = i64::MAX.unsigned_abs();

/// A positive quantity with four decimal places, stored as ten-thousandths:
/// 0.452 kg is 4 520.
///
/// The quantity of an item multiplies its unit price into its line total.
/// The text form has no trailing zeros, such as `1.5`, `3`, or `0.0001`, and
/// a serialised quantity is that text.
///
/// # Examples
///
/// ```
/// use eranos::Quantity;
///
/// let weight = Quantity::parse("0.452")?;
/// assert_eq!(weight.ten_thousandths(), 4_520);
/// assert_eq!(weight.to_string(), "0.452");
/// assert_eq!(Quantity::from_whole(3)?.to_string(), "3");
/// assert_eq!(Quantity::default(), Quantity::ONE);
/// assert!(Quantity::parse("0").is_err());
/// assert!(Quantity::parse("0.00005").is_err());
/// # Ok::<(), eranos::error::MoneyError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quantity(u64);

impl Quantity {
    /// One unit: the quantity of an item that does not set one.
    pub const ONE: Self = Self(SCALE);

    /// Returns `ten_thousandths / 10 000` units.
    ///
    /// # Errors
    ///
    /// [`MoneyError::InvalidQuantity`] when `ten_thousandths` is zero or above
    /// `i64::MAX`.
    pub const fn from_ten_thousandths(ten_thousandths: u64) -> Result<Self, MoneyError> {
        if 0 == ten_thousandths || MAX < ten_thousandths {
            return Err(MoneyError::InvalidQuantity);
        }
        Ok(Self(ten_thousandths))
    }

    /// Returns `units` whole units.
    ///
    /// # Errors
    ///
    /// [`MoneyError::InvalidQuantity`] when `units` is zero or its
    /// ten-thousandths are above `i64::MAX`.
    pub const fn from_whole(units: u64) -> Result<Self, MoneyError> {
        match units.checked_mul(SCALE) {
            Some(ten_thousandths) => Self::from_ten_thousandths(ten_thousandths),
            None => Err(MoneyError::InvalidQuantity),
        }
    }

    /// Parses a positive decimal with at most four decimal places, such as
    /// `2`, `1.5`, or `0.452`, without a sign or an exponent; surrounding
    /// whitespace is ignored.
    ///
    /// # Errors
    ///
    /// [`MoneyError::InvalidQuantity`] for anything else, zero included, and
    /// for a quantity whose ten-thousandths are above `i64::MAX`.
    pub fn parse(text: &str) -> Result<Self, MoneyError> {
        let ten_thousandths =
            decimal::parse_fixed(text.trim(), PLACES).map_err(|_| MoneyError::InvalidQuantity)?;
        Self::from_ten_thousandths(ten_thousandths)
    }

    /// Converts the double `value` into a quantity, for an app that holds
    /// quantities as doubles.
    ///
    /// The double is written in its shortest decimal form, rounded half away
    /// from zero to four decimal places, and parsed by [`parse`](Self::parse),
    /// as [`Money::from_f64`](crate::Money::from_f64) converts an amount:
    /// 0.45205 is 0.4521.
    ///
    /// # Errors
    ///
    /// - [`MoneyError::NotFinite`] when `value` is NaN or infinite.
    /// - [`MoneyError::InvalidQuantity`] when the rounded value is not
    ///   positive, as 0.00004 rounds to zero, or its ten-thousandths are
    ///   above `i64::MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::Quantity;
    ///
    /// assert_eq!(Quantity::from_f64(0.45205)?.ten_thousandths(), 4_521);
    /// assert_eq!(Quantity::from_f64(1.5)?.to_string(), "1.5");
    /// assert!(Quantity::from_f64(0.00004).is_err());
    /// # Ok::<(), eranos::error::MoneyError>(())
    /// ```
    pub fn from_f64(value: f64) -> Result<Self, MoneyError> {
        let text = decimal::rounded(value, PLACES).ok_or(MoneyError::NotFinite)?;
        Self::parse(&text)
    }

    /// The quantity in ten-thousandths: 15 000 for 1.5, at most `i64::MAX`.
    #[must_use]
    pub const fn ten_thousandths(self) -> u64 {
        self.0
    }
}

impl Default for Quantity {
    /// One unit, [`Quantity::ONE`].
    fn default() -> Self {
        Self::ONE
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // A quantity holds at most `i64::MAX` ten-thousandths, so the
        // fallback never applies.
        let ten_thousandths = i64::try_from(self.0).unwrap_or(i64::MAX);
        fmt::Display::fmt(&format_ten_thousandths(ten_thousandths), formatter)
    }
}

impl FromStr for Quantity {
    type Err = MoneyError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

eranos_core::serde_text!(
    Quantity,
    "a positive decimal quantity with at most 4 decimal places"
);

/// Formats `ten_thousandths` as a quantity: a decimal without trailing
/// zeros.
///
/// The text form of [`Quantity`] writes its ten-thousandths with it: 15 000
/// is `1.5`, 4 520 is `0.452`, 30 000 is `3`, and 1 is `0.0001`. It takes
/// any number, zero and negative ones included, which no quantity holds,
/// and writes it the same way, a negative one after a minus sign: the
/// binding core writes with it the ten-thousandths of a record it has not
/// checked, so that its text for a quantity is this crate's.
///
/// # Examples
///
/// ```
/// # // The facade does not re-export it: a caller writes a quantity through its `Quantity`.
/// use eranos_money::format_ten_thousandths;
///
/// assert_eq!(format_ten_thousandths(15_000).to_string(), "1.5");
/// assert_eq!(format_ten_thousandths(0).to_string(), "0");
/// assert_eq!(format_ten_thousandths(-4_520).to_string(), "-0.452");
/// ```
#[must_use]
pub fn format_ten_thousandths(ten_thousandths: i64) -> impl fmt::Display {
    fmt::from_fn(move |formatter| decimal::write_fixed(formatter, ten_thousandths, PLACES, ""))
}

#[cfg(test)]
mod tests;
