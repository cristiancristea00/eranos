//! Amounts of money in minor units.

use std::fmt;

use crate::error::MoneyError;
use crate::{Cost, Currency, ExchangeRate};

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
/// use eranos::{Currency, Money};
///
/// let price = Money::parse("12.34", Currency::EUR)?;
/// assert_eq!(price.minor_units(), 1234);
/// assert_eq!(price.to_string(), "12.34 EUR");
///
/// let tip = Money::from_minor_units(Currency::EUR, 166);
/// assert_eq!(price.checked_add(tip)?, Money::parse("14.00", Currency::EUR)?);
/// # Ok::<(), eranos::error::MoneyError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Money {
    currency: Currency,
    minor_units: i64,
}

impl Money {
    /// Returns `minor_units` of `currency`: `from_minor_units(Currency::EUR,
    /// 1234)` is 12.34 EUR.
    #[must_use]
    pub const fn from_minor_units(currency: Currency, minor_units: i64) -> Self {
        Self {
            currency,
            minor_units,
        }
    }

    /// Returns zero in `currency`.
    #[must_use]
    pub const fn zero(currency: Currency) -> Self {
        Self::from_minor_units(currency, 0)
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
    /// use eranos::{Currency, Money};
    ///
    /// assert_eq!(Money::parse("1500", Currency::JPY)?.minor_units(), 1500);
    /// assert_eq!(Money::parse("0.5", Currency::EUR)?.minor_units(), 50);
    /// assert!(Money::parse("0.505", Currency::EUR).is_err());
    /// assert!(Money::parse("1,50", Currency::EUR).is_err());
    /// # Ok::<(), eranos::error::MoneyError>(())
    /// ```
    pub fn parse(text: &str, currency: Currency) -> Result<Self, MoneyError> {
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
        Ok(Self::from_minor_units(currency, minor_units))
    }

    /// The currency.
    #[must_use]
    pub const fn currency(self) -> Currency {
        self.currency
    }

    /// The amount in minor units: 1234 for 12.34 EUR.
    #[must_use]
    pub const fn minor_units(self) -> i64 {
        self.minor_units
    }

    /// Whether the amount is zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.minor_units == 0
    }

    /// Whether the amount is greater than zero.
    #[must_use]
    pub const fn is_positive(self) -> bool {
        self.minor_units > 0
    }

    /// Whether the amount is less than zero.
    #[must_use]
    pub const fn is_negative(self) -> bool {
        self.minor_units < 0
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
    /// use eranos::{Currency, ExchangeRate, Money};
    ///
    /// // Fuel paid in RON, for a group in EUR: 1 RON = 0.2 EUR.
    /// let fuel = Money::parse("400.00", Currency::RON)?.at(ExchangeRate::parse("0.2")?);
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
        Ok(Self::from_minor_units(self.currency, sum))
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
        Ok(Self::from_minor_units(self.currency, difference))
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
        let sign = if self.minor_units < 0 { "-" } else { "" };
        let magnitude = self.minor_units.unsigned_abs();
        let decimal_places = usize::from(self.currency.decimal_places());
        if decimal_places == 0 {
            return write!(formatter, "{sign}{magnitude} {}", self.currency);
        }
        let per_major = self.currency.minor_units_per_major().unsigned_abs();
        let whole = magnitude / per_major;
        let fraction = magnitude % per_major;
        write!(
            formatter,
            "{sign}{whole}.{fraction:0decimal_places$} {}",
            self.currency
        )
    }
}

#[cfg(test)]
mod tests;
