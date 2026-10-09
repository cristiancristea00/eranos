//! Fixed exchange rates as exact decimals.

use std::fmt;
use std::str::FromStr;

use crate::allocation::divide_rounding_half_away;
use crate::decimal;
use crate::error::MoneyError;
use crate::{CurrencyCode, Money};

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
/// use eranos::{CurrencyCode, ExchangeRate, Money};
///
/// // 1 EUR = 4.9765 RON
/// let rate = ExchangeRate::parse("4.9765")?;
/// let converted = rate.convert(Money::parse("10.00", CurrencyCode::EUR)?, CurrencyCode::RON)?;
/// assert_eq!(converted, Money::parse("49.77", CurrencyCode::RON)?); // 49.765 rounds half up
/// # Ok::<(), eranos::error::MoneyError>(())
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
    /// [`MoneyError::InvalidExchangeRate`] when `mantissa` is zero or `scale`
    /// is above 18.
    pub const fn from_decimal(mantissa: u64, scale: u8) -> Result<Self, MoneyError> {
        if 0 == mantissa || MAX_SCALE < scale {
            return Err(MoneyError::InvalidExchangeRate);
        }
        let (mut mantissa, mut scale) = (mantissa, scale);
        while 0 < scale && 0 == mantissa % 10 {
            mantissa /= 10;
            scale -= 1;
        }
        Ok(Self { mantissa, scale })
    }

    /// Parses a positive decimal such as `4.9765` or `0.2011`.
    ///
    /// # Errors
    ///
    /// [`MoneyError::InvalidExchangeRate`] when the text is not digits with
    /// an optional `.` followed by digits, is zero, has more than 18 decimal
    /// places, or has more significant digits than fit in 64 bits.
    pub fn parse(text: &str) -> Result<Self, MoneyError> {
        let text = text.trim();
        let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
        let is_digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
        if whole.is_empty() || !is_digits(whole) || !is_digits(fraction) || text.ends_with('.') {
            return Err(MoneyError::InvalidExchangeRate);
        }
        let scale = u8::try_from(fraction.len()).map_err(|_| MoneyError::InvalidExchangeRate)?;
        let mantissa = whole
            .bytes()
            .chain(fraction.bytes())
            .try_fold(0_u64, |value, digit| {
                value.checked_mul(10)?.checked_add(u64::from(digit - b'0'))
            })
            .ok_or(MoneyError::InvalidExchangeRate)?;
        Self::from_decimal(mantissa, scale)
    }

    /// Converts the double `value` into an exchange rate, for an app that
    /// holds rates as doubles.
    ///
    /// The double is written in its shortest decimal form, rounded half away
    /// from zero to 18 decimal places, and parsed by [`parse`](Self::parse),
    /// as [`Money::from_f64`] converts an amount. A rate keeps every digit of
    /// that form up to the 18th place and no more: 4.9765 is 4.9765, and
    /// `0.1 + 0.2` is 0.30000000000000004, the digits of the double.
    ///
    /// # Errors
    ///
    /// - [`MoneyError::NotFinite`] when `value` is NaN or infinite.
    /// - [`MoneyError::InvalidExchangeRate`] when the rounded value is not
    ///   positive or has more significant digits than fit in 64 bits.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::ExchangeRate;
    ///
    /// assert_eq!(ExchangeRate::from_f64(4.9765)?, ExchangeRate::parse("4.9765")?);
    /// assert_eq!(ExchangeRate::from_f64(25.5)?.scale(), 1);
    /// assert!(ExchangeRate::from_f64(0.0).is_err());
    /// # Ok::<(), eranos::error::MoneyError>(())
    /// ```
    pub fn from_f64(value: f64) -> Result<Self, MoneyError> {
        let text = decimal::rounded(value, MAX_SCALE).ok_or(MoneyError::NotFinite)?;
        Self::parse(&text)
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
    /// [`MoneyError::Overflow`] when the converted amount does not fit in
    /// 64-bit minor units.
    pub fn convert(self, amount: Money, target: CurrencyCode) -> Result<Money, MoneyError> {
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
            .ok_or(MoneyError::Overflow)?;
        let denominator = 10_i128.pow(denominator_power);
        let converted = divide_rounding_half_away(numerator, denominator);
        let minor_units = i64::try_from(converted).map_err(|_| MoneyError::Overflow)?;
        Ok(Money::from_minor_units(minor_units, target))
    }
}

impl fmt::Display for ExchangeRate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scale = usize::from(self.scale);
        if 0 == scale {
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
    type Err = MoneyError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

eranos_core::serde_text!(ExchangeRate, "a positive decimal exchange rate");

/// The rate that converts one currency into another, as a currency change
/// needs one for every currency in use: `ExpenseGroup::change_currency`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CurrencyRate {
    currency: CurrencyCode,
    rate: ExchangeRate,
}

impl CurrencyRate {
    /// The rate that converts `currency`: one unit of it is worth `rate`
    /// units of the target currency.
    #[must_use]
    pub const fn new(currency: CurrencyCode, rate: ExchangeRate) -> Self {
        Self { currency, rate }
    }

    /// The currency the rate converts.
    #[must_use]
    pub const fn currency(self) -> CurrencyCode {
        self.currency
    }

    /// How many units of the target currency one unit of the currency is
    /// worth.
    #[must_use]
    pub const fn rate(self) -> ExchangeRate {
        self.rate
    }
}

#[cfg(test)]
mod tests;
