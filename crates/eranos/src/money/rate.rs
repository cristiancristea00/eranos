//! Fixed exchange rates as exact decimals.

use std::fmt;
use std::str::FromStr;

use crate::error::ValueError;
use crate::money::allocation::divide_rounding_half_away;
use crate::money::{Currency, Money};

/// The most decimal places an [`ExchangeRate`] may have.
const MAX_SCALE: u8 = 18;

/// A fixed exchange rate: how many units of the target currency one unit of
/// the source currency is worth.
///
/// Stored as an exact positive decimal (`mantissa / 10^scale`), so a rate
/// typed as `4.9765` converts exactly as written. An entry's rate converts
/// from the entry's currency into its group's currency.
///
/// # Examples
///
/// ```
/// use eranos::{Currency, ExchangeRate, Money};
///
/// // 1 EUR = 4.9765 RON
/// let rate = ExchangeRate::parse("4.9765")?;
/// let converted = rate.convert(Money::parse("10.00", Currency::EUR)?, Currency::RON)?;
/// assert_eq!(converted, Money::parse("49.77", Currency::RON)?); // 49.765 rounds half up
/// # Ok::<(), eranos::error::ValueError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExchangeRate {
    // Normalised: no trailing zero digits in the mantissa while scale > 0, so
    // equal rates have equal fields.
    mantissa: u64,
    scale: u8,
}

impl ExchangeRate {
    /// The rate 1, at which one unit of the source currency is worth one
    /// unit of the target: the rate of a local [`Cost`](crate::Cost), and of
    /// a currency pegged at par.
    pub const ONE: Self = Self {
        mantissa: 1,
        scale: 0,
    };

    /// Returns the rate `mantissa / 10^scale`: `from_decimal(49765, 4)` is
    /// 4.9765.
    ///
    /// # Errors
    ///
    /// [`ValueError::InvalidExchangeRate`] when `mantissa` is zero or `scale`
    /// is above 18.
    pub const fn from_decimal(mantissa: u64, scale: u8) -> Result<Self, ValueError> {
        if mantissa == 0 || scale > MAX_SCALE {
            return Err(ValueError::InvalidExchangeRate);
        }
        let (mut mantissa, mut scale) = (mantissa, scale);
        while scale > 0 && mantissa % 10 == 0 {
            mantissa /= 10;
            scale -= 1;
        }
        Ok(Self { mantissa, scale })
    }

    /// Parses a positive decimal such as `4.9765` or `0.2011`.
    ///
    /// # Errors
    ///
    /// [`ValueError::InvalidExchangeRate`] when the text is not digits with
    /// an optional `.` followed by digits, is zero, has more than 18 decimal
    /// places, or has more significant digits than fit in 64 bits.
    pub fn parse(text: &str) -> Result<Self, ValueError> {
        let text = text.trim();
        let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
        let is_digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
        if whole.is_empty() || !is_digits(whole) || !is_digits(fraction) || text.ends_with('.') {
            return Err(ValueError::InvalidExchangeRate);
        }
        let scale = u8::try_from(fraction.len()).map_err(|_| ValueError::InvalidExchangeRate)?;
        let mantissa = whole
            .bytes()
            .chain(fraction.bytes())
            .try_fold(0_u64, |value, digit| {
                value.checked_mul(10)?.checked_add(u64::from(digit - b'0'))
            })
            .ok_or(ValueError::InvalidExchangeRate)?;
        Self::from_decimal(mantissa, scale)
    }

    /// The digits of the rate without the decimal point: 49765 for 4.9765.
    #[must_use]
    pub const fn mantissa(self) -> u64 {
        self.mantissa
    }

    /// The number of decimal places: 4 for 4.9765.
    #[must_use]
    pub const fn scale(self) -> u8 {
        self.scale
    }

    /// Converts `amount` into `target`, rounding half away from zero to the
    /// target's minor unit.
    ///
    /// Half away from zero is the rounding rule of EU Regulation 1103/97
    /// (art. 5) for currency conversions.
    ///
    /// # Errors
    ///
    /// [`ValueError::Overflow`] when the converted amount does not fit in
    /// 64-bit minor units.
    pub fn convert(self, amount: Money, target: Currency) -> Result<Money, ValueError> {
        let source_places = i32::from(amount.currency().decimal_places());
        let target_places = i32::from(target.decimal_places());
        // amount × mantissa × 10^(target − source) / 10^scale, with the
        // powers of ten moved to whichever side keeps them non-negative.
        let shift = target_places - source_places;
        let numerator_power = shift.max(0).unsigned_abs();
        let denominator_power = u32::from(self.scale) + (-shift).max(0).unsigned_abs();
        // |amount × mantissa| < 2^127 always fits in i128. A currency has at
        // most four decimal places and a rate at most 18, so the powers of
        // ten are at most 10^4 and 10^22.
        let numerator = (i128::from(amount.minor_units()) * i128::from(self.mantissa))
            .checked_mul(10_i128.pow(numerator_power))
            .ok_or(ValueError::Overflow)?;
        let denominator = 10_i128.pow(denominator_power);
        let converted = divide_rounding_half_away(numerator, denominator);
        let minor_units = i64::try_from(converted).map_err(|_| ValueError::Overflow)?;
        Ok(Money::from_minor_units(target, minor_units))
    }
}

impl fmt::Display for ExchangeRate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scale = usize::from(self.scale);
        if scale == 0 {
            return write!(formatter, "{}", self.mantissa);
        }
        let divisor = 10_u64.pow(u32::from(self.scale));
        write!(
            formatter,
            "{}.{:0scale$}",
            self.mantissa / divisor,
            self.mantissa % divisor
        )
    }
}

impl FromStr for ExchangeRate {
    type Err = ValueError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for ExchangeRate {
    fn serialize<S: serde::Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
        serialiser.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ExchangeRate {
    fn deserialize<D: serde::Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        crate::serde_text::deserialise(deserialiser, "a positive decimal exchange rate")
    }
}

#[cfg(test)]
mod tests;
