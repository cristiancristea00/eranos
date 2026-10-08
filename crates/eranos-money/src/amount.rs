//! Amounts of money in minor units.

use std::fmt;

use crate::decimal;
use crate::error::MoneyError;
use crate::{Cost, CurrencyCode, ExchangeRate};

/// An amount of money: a whole number of minor units (cents) in one currency.
///
/// The text form is the decimal amount and the code, such as `12.34 EUR`;
/// formatting for display in a locale belongs to the apps. There is no
/// ordering, because comparing amounts in different currencies is
/// meaningless; compare [`minor_units`](Self::minor_units) of amounts in the
/// same currency instead.
///
/// # Examples
///
/// ```
/// use eranos::{CurrencyCode, Money};
///
/// let price = Money::parse("12.34", CurrencyCode::EUR)?;
/// assert_eq!(price.minor_units(), 1234);
/// assert_eq!(price.to_string(), "12.34 EUR");
///
/// let tip = Money::from_minor_units(166, CurrencyCode::EUR);
/// assert_eq!(price.checked_add(tip)?, Money::parse("14.00", CurrencyCode::EUR)?);
/// # Ok::<(), eranos::error::MoneyError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Money {
    minor_units: i64,
    currency: CurrencyCode,
}

impl Money {
    /// Returns `minor_units` of `currency`: `from_minor_units(1234,
    /// CurrencyCode::EUR)` is 12.34 EUR.
    #[must_use]
    pub const fn from_minor_units(minor_units: i64, currency: CurrencyCode) -> Self {
        Self {
            minor_units,
            currency,
        }
    }

    /// Returns zero in `currency`.
    #[must_use]
    pub const fn zero(currency: CurrencyCode) -> Self {
        Self::from_minor_units(0, currency)
    }

    /// Parses a decimal amount such as `12.34`, `-5`, or `0.5`.
    ///
    /// `.` is the only decimal separator and no grouping separators are
    /// accepted; locale-specific input is converted by the apps.
    ///
    /// # Errors
    ///
    /// - [`MoneyError::InvalidAmount`] when the text is not digits with an
    ///   optional leading `-` and an optional `.` followed by digits.
    /// - [`MoneyError::TooManyDecimals`] when it has more decimal places than
    ///   the currency.
    /// - [`MoneyError::Overflow`] when it does not fit in 64-bit minor units.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{CurrencyCode, Money};
    ///
    /// assert_eq!(Money::parse("1500", CurrencyCode::JPY)?.minor_units(), 1500);
    /// assert_eq!(Money::parse("0.5", CurrencyCode::EUR)?.minor_units(), 50);
    /// assert!(Money::parse("0.505", CurrencyCode::EUR).is_err());
    /// assert!(Money::parse("1,50", CurrencyCode::EUR).is_err());
    /// # Ok::<(), eranos::error::MoneyError>(())
    /// ```
    pub fn parse(text: &str, currency: CurrencyCode) -> Result<Self, MoneyError> {
        let text = text.trim();
        let (negative, unsigned) = text
            .strip_prefix('-')
            .map_or((false, text), |rest| (true, rest));
        let (whole, fraction) = match unsigned.split_once('.') {
            Some((whole, fraction)) => (whole, Some(fraction)),
            None => (unsigned, None),
        };
        let is_digits =
            |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
        if !is_digits(whole) || fraction.is_some_and(|fraction| !is_digits(fraction)) {
            return Err(MoneyError::InvalidAmount);
        }
        let fraction = fraction.unwrap_or("");
        let allowed = currency.decimal_places();
        if fraction.len() > usize::from(allowed) {
            return Err(MoneyError::TooManyDecimals { allowed, currency });
        }
        // Right-pad the fraction to the currency's decimal places: "12.3" EUR
        // is 1230 minor units.
        let padding = usize::from(allowed) - fraction.len();
        let mut digits = whole
            .bytes()
            .chain(fraction.bytes())
            .chain(std::iter::repeat_n(b'0', padding));
        // Accumulated in i128 and narrowed once, so the most negative amount,
        // whose magnitude exceeds i64::MAX, still parses.
        let magnitude = digits
            .try_fold(0_i128, |value, digit| {
                value.checked_mul(10)?.checked_add(i128::from(digit - b'0'))
            })
            .ok_or(MoneyError::Overflow)?;
        let signed = if negative { -magnitude } else { magnitude };
        let minor_units = i64::try_from(signed).map_err(|_| MoneyError::Overflow)?;
        Ok(Self::from_minor_units(minor_units, currency))
    }

    /// Converts the double `value` into an amount of `currency`, for an app
    /// that holds amounts as doubles.
    ///
    /// The double is written in its shortest decimal form, the shortest
    /// text that reads back as the same double, rounded half away from zero
    /// to the currency's decimal places, and parsed by [`parse`](Self::parse).
    /// Rounding the text rather than the double keeps the cents the user
    /// typed: the double nearest 1.005 lies below it, yet 1.005 EUR is
    /// 1.01 EUR. Every binding converts through this function, so the same
    /// double is the same amount in every language.
    ///
    /// # Errors
    ///
    /// - [`MoneyError::NotFinite`] when `value` is NaN or infinite.
    /// - [`MoneyError::Overflow`] when the rounded amount does not fit in
    ///   64-bit minor units.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{CurrencyCode, Money};
    ///
    /// assert_eq!(Money::from_f64(1.005, CurrencyCode::EUR)?.to_string(), "1.01 EUR");
    /// assert_eq!(Money::from_f64(-2.675, CurrencyCode::EUR)?.minor_units(), -268);
    /// assert_eq!(Money::from_f64(1234.5, CurrencyCode::JPY)?.to_string(), "1235 JPY");
    /// assert!(Money::from_f64(f64::NAN, CurrencyCode::EUR).is_err());
    /// # Ok::<(), eranos::error::MoneyError>(())
    /// ```
    pub fn from_f64(value: f64, currency: CurrencyCode) -> Result<Self, MoneyError> {
        let text =
            decimal::rounded(value, currency.decimal_places()).ok_or(MoneyError::NotFinite)?;
        Self::parse(&text, currency)
    }

    /// The amount in minor units: 1234 for 12.34 EUR.
    #[must_use]
    pub const fn minor_units(self) -> i64 {
        self.minor_units
    }

    /// The currency.
    #[must_use]
    pub const fn currency(self) -> CurrencyCode {
        self.currency
    }

    /// Whether the amount is zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        0 == self.minor_units
    }

    /// Whether the amount is greater than zero.
    #[must_use]
    pub const fn is_positive(self) -> bool {
        0 < self.minor_units
    }

    /// Whether the amount is less than zero.
    #[must_use]
    pub const fn is_negative(self) -> bool {
        0 > self.minor_units
    }

    /// This amount, in another currency than its group's, at `rate` into
    /// the group currency.
    ///
    /// The result is a [`Cost::Converted`] cost even at [`ExchangeRate::ONE`],
    /// the rate of a currency pegged at par.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{CurrencyCode, ExchangeRate, Money};
    ///
    /// // Fuel paid in RON, for a group in EUR: 1 RON = 0.2 EUR.
    /// let fuel = Money::parse("400.00", CurrencyCode::RON)?.at(ExchangeRate::parse("0.2")?);
    /// assert_eq!(fuel.exchange_rate(), ExchangeRate::parse("0.2")?);
    /// # Ok::<(), eranos::error::MoneyError>(())
    /// ```
    #[must_use]
    pub const fn at(self, rate: ExchangeRate) -> Cost {
        Cost::Converted { amount: self, rate }
    }

    /// Adds two amounts of the same currency.
    ///
    /// # Errors
    ///
    /// - [`MoneyError::CurrencyMismatch`] when the currencies differ.
    /// - [`MoneyError::Overflow`] when the sum does not fit in 64-bit minor
    ///   units.
    pub fn checked_add(self, other: Self) -> Result<Self, MoneyError> {
        self.same_currency(other)?;
        let sum = self
            .minor_units
            .checked_add(other.minor_units)
            .ok_or(MoneyError::Overflow)?;
        Ok(Self::from_minor_units(sum, self.currency))
    }

    /// Subtracts an amount of the same currency.
    ///
    /// # Errors
    ///
    /// - [`MoneyError::CurrencyMismatch`] when the currencies differ.
    /// - [`MoneyError::Overflow`] when the difference does not fit in 64-bit
    ///   minor units.
    pub fn checked_sub(self, other: Self) -> Result<Self, MoneyError> {
        self.same_currency(other)?;
        let difference = self
            .minor_units
            .checked_sub(other.minor_units)
            .ok_or(MoneyError::Overflow)?;
        Ok(Self::from_minor_units(difference, self.currency))
    }

    fn same_currency(self, other: Self) -> Result<(), MoneyError> {
        if self.currency == other.currency {
            Ok(())
        } else {
            Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: other.currency,
            })
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} {}",
            format_minor_units(self.minor_units, self.currency.decimal_places()),
            self.currency
        )
    }
}

/// Formats `minor_units` as a decimal number of `decimal_places` places.
///
/// The text form of [`Money`] writes its amount with it: 1234 at two places
/// is `12.34`, -5 at three is `-0.005`, and 1500 at none is `1500`. The
/// point goes between the digits, padded with zeros to one more digit than
/// the decimal places, so every `u8` of decimal places formats: dividing by
/// `10^decimal_places` would overflow an `i64` from 19 places.
///
/// # Examples
///
/// ```
/// # // The facade does not re-export it: a caller writes an amount through its `Money`.
/// use eranos_money::format_minor_units;
///
/// assert_eq!(format_minor_units(1_234, 2).to_string(), "12.34");
/// assert_eq!(format_minor_units(-5, 3).to_string(), "-0.005");
/// assert_eq!(format_minor_units(1_500, 0).to_string(), "1500");
/// ```
#[must_use]
pub fn format_minor_units(minor_units: i64, decimal_places: u8) -> impl fmt::Display {
    fmt::from_fn(move |formatter| {
        let sign = if 0 > minor_units { "-" } else { "" };
        let magnitude = minor_units.unsigned_abs();
        if 0 == decimal_places {
            return write!(formatter, "{sign}{magnitude}");
        }
        let places = usize::from(decimal_places);
        let width = places + 1;
        let digits = format!("{magnitude:0width$}");
        // `digits` holds at least `width` ASCII digits, so the split falls
        // inside it, on a character boundary, and leaves a digit before the
        // point.
        let (whole, fraction) = digits.split_at(digits.len() - places);
        write!(formatter, "{sign}{whole}.{fraction}")
    })
}

#[cfg(test)]
mod tests;
