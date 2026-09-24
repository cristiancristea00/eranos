//! Transfers: money one member paid another, such as a repayment.

use crate::date::{Date, Time};
use crate::error::TransferError;
use crate::id::MemberId;
use crate::ledger::MemberPair;
use crate::money::{Cost, ExchangeRate, Money};
use crate::text;

/// Money one member paid directly to another, usually to settle up.
///
/// A transfer moves balances: the sender's debt shrinks and the recipient's
/// credit shrinks. It is not spending, so it counts towards no total or
/// category. Like an expense, it has a [`Cost`]: a transfer in another
/// currency than its group's carries the [`ExchangeRate`] into the group
/// currency.
///
/// # Examples
///
/// ```
/// use eranos::{Currency, Date, Money, Transfer};
/// use eranos::id::MemberId;
///
/// let (ana, bob) = (MemberId::generate(), MemberId::generate());
/// let repayment = Transfer::builder(bob, ana, Money::parse("30.00", Currency::EUR)?, Date::new(2026, 9, 24)?)
///     .note("Dinner")
///     .build()?;
/// assert_eq!(repayment.from(), bob);
/// assert!(Transfer::builder(ana, ana, repayment.amount(), repayment.date()).build().is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "TransferRecord")
)]
pub struct Transfer {
    amount: Money,
    date: Date,
    from: MemberId,
    to: MemberId,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    exchange_rate: Option<ExchangeRate>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    note: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    time: Option<Time>,
}

impl Transfer {
    /// Starts a transfer of `cost` from `from` to `to` on `date`.
    ///
    /// A [`Money`] amount is a local cost, in the group currency;
    /// [`Money::at`] makes a converted one.
    pub fn builder(
        from: MemberId,
        to: MemberId,
        cost: impl Into<Cost>,
        date: Date,
    ) -> TransferBuilder {
        TransferBuilder {
            cost: cost.into(),
            date,
            from,
            to,
            note: None,
            time: None,
        }
    }

    /// Starts a builder holding a copy of this transfer, to edit it.
    pub fn to_builder(&self) -> TransferBuilder {
        TransferBuilder {
            cost: self.cost(),
            date: self.date,
            from: self.from,
            to: self.to,
            note: self.note.clone(),
            time: self.time,
        }
    }

    /// The member who paid.
    #[must_use]
    pub const fn from(&self) -> MemberId {
        self.from
    }

    /// The member who received the money.
    #[must_use]
    pub const fn to(&self) -> MemberId {
        self.to
    }

    /// The cost: the amount and, for a transfer in another currency than
    /// its group's, the rate into the group currency.
    #[must_use]
    pub const fn cost(&self) -> Cost {
        Cost::from_parts(self.amount, self.exchange_rate)
    }

    /// The amount, in the transfer's own currency.
    #[must_use]
    pub const fn amount(&self) -> Money {
        self.amount
    }

    /// The date of the transfer.
    #[must_use]
    pub const fn date(&self) -> Date {
        self.date
    }

    /// The time of day, if recorded.
    #[must_use]
    pub const fn time(&self) -> Option<Time> {
        self.time
    }

    /// The rate from the transfer's currency into its group's currency:
    /// [`ExchangeRate::ONE`] for a local cost.
    #[must_use]
    pub const fn exchange_rate(&self) -> ExchangeRate {
        self.cost().exchange_rate()
    }

    /// A note, if any.
    #[must_use]
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    /// The sender and the recipient.
    pub(crate) const fn pair(&self) -> MemberPair {
        MemberPair {
            from: self.from,
            to: self.to,
        }
    }

    pub(crate) const fn set_exchange_rate(&mut self, exchange_rate: Option<ExchangeRate>) {
        self.exchange_rate = exchange_rate;
    }

    /// A transfer the caller guarantees is valid: two different members and
    /// a positive amount.
    pub(crate) fn new_unchecked(from: MemberId, to: MemberId, amount: Money, date: Date) -> Self {
        debug_assert!(
            from != to && amount.is_positive(),
            "an unchecked transfer must be valid"
        );
        Self {
            amount,
            date,
            from,
            to,
            exchange_rate: None,
            note: None,
            time: None,
        }
    }
}

/// Builds a [`Transfer`]; created by [`Transfer::builder`] or
/// [`Transfer::to_builder`].
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct TransferBuilder {
    cost: Cost,
    date: Date,
    from: MemberId,
    to: MemberId,
    note: Option<String>,
    time: Option<Time>,
}

impl TransferBuilder {
    /// Replaces the member who paid.
    pub const fn from(mut self, from: MemberId) -> Self {
        self.from = from;
        self
    }

    /// Replaces the member who received the money.
    pub const fn to(mut self, to: MemberId) -> Self {
        self.to = to;
        self
    }

    /// Replaces the cost.
    ///
    /// A [`Money`] amount makes the cost local; to change the amount of a
    /// converted cost and keep its rate, pass
    /// `transfer.cost().with_amount(amount)`.
    pub fn cost(mut self, cost: impl Into<Cost>) -> Self {
        self.cost = cost.into();
        self
    }

    /// Replaces the date.
    pub const fn date(mut self, date: Date) -> Self {
        self.date = date;
        self
    }

    /// Sets the time of day, or clears it with `None`.
    pub fn time(mut self, time: impl Into<Option<Time>>) -> Self {
        self.time = time.into();
        self
    }

    /// Sets the note; blank text clears it.
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Validates the transfer and returns it.
    ///
    /// # Errors
    ///
    /// - [`TransferError::SameMember`] when the sender is the recipient.
    /// - [`TransferError::NonPositiveAmount`] when the amount is not
    ///   positive.
    pub fn build(self) -> Result<Transfer, TransferError> {
        if self.from == self.to {
            return Err(TransferError::SameMember);
        }
        let amount = self.cost.amount();
        if !amount.is_positive() {
            return Err(TransferError::NonPositiveAmount);
        }
        Ok(Transfer {
            amount,
            date: self.date,
            from: self.from,
            to: self.to,
            exchange_rate: self.cost.conversion(),
            note: self.note.and_then(text::optional),
            time: self.time,
        })
    }
}

/// The serialised form of a [`Transfer`], validated on the way in.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct TransferRecord {
    amount: Money,
    date: Date,
    from: MemberId,
    to: MemberId,
    #[serde(default)]
    exchange_rate: Option<ExchangeRate>,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    time: Option<Time>,
}

#[cfg(feature = "serde")]
impl TryFrom<TransferRecord> for Transfer {
    type Error = TransferError;

    fn try_from(record: TransferRecord) -> Result<Self, Self::Error> {
        let cost = Cost::from_parts(record.amount, record.exchange_rate);
        let builder = Self::builder(record.from, record.to, cost, record.date).time(record.time);
        match record.note {
            Some(note) => builder.note(note).build(),
            None => builder.build(),
        }
    }
}

#[cfg(test)]
mod tests;
