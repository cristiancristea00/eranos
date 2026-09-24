//! Transfers: money one member paid another, such as a repayment.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use. [`resolve_transfer`], which converts a transfer into
//! the group currency for the ledger, is the group's, and the facade does
//! not re-export it.
#![doc(test(attr(deny(warnings))))]

mod error;

use eranos_calendar::{CalendarDate, TimeOfDay};
use eranos_core::{MemberId, TransferId, text};
use eranos_money::{Cost, Currency, ExchangeRate, Money, MoneyError};
use eranos_settlement::Payment;

pub use error::TransferError;

/// Money one member paid directly to another, usually to settle up.
///
/// A transfer moves balances: the sender's debt shrinks and the recipient's
/// credit shrinks. It is not spending, so it counts towards no total or
/// category. Like an expense, it has a [`Cost`]: a transfer in another
/// currency than its group's carries the [`ExchangeRate`] into the group
/// currency. A transfer gets its identifier when it is built, and keeps it
/// through every edit.
///
/// # Examples
///
/// ```
/// use eranos::{Currency, CalendarDate, Money, Transfer};
/// use eranos::id::MemberId;
///
/// let (ana, bob) = (MemberId::generate(), MemberId::generate());
/// let repayment = Transfer::builder(Money::parse("30.00", Currency::EUR)?, CalendarDate::new(24, 9, 2026)?, bob, ana)
///     .note("Dinner")
///     .build()?;
/// assert_eq!(repayment.from(), bob);
/// assert!(Transfer::builder(repayment.amount(), repayment.date(), ana, ana).build().is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Transfer {
    id: TransferId,
    amount: Money,
    date: CalendarDate,
    from: MemberId,
    to: MemberId,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    exchange_rate: Option<ExchangeRate>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    note: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    time: Option<TimeOfDay>,
}

impl Transfer {
    /// Starts a transfer of `cost` on `date`, from `from` to `to`.
    ///
    /// A [`Money`] amount is a local cost, in the group currency;
    /// [`Money::at`] makes a converted one.
    pub fn builder(
        cost: impl Into<Cost>,
        date: CalendarDate,
        from: MemberId,
        to: MemberId,
    ) -> TransferBuilder {
        TransferBuilder {
            id: None,
            cost: cost.into(),
            date,
            from,
            to,
            note: None,
            time: None,
        }
    }

    /// The transfer recording `payment`, made on `date`, under a newly
    /// generated identifier.
    ///
    /// Recording every payment of a [`Settlement`](eranos_settlement::Settlement) brings
    /// every balance to zero.
    #[must_use]
    pub fn from_payment(payment: &Payment, date: CalendarDate) -> Self {
        // A settlement pays only from a debtor to a creditor, and only a
        // positive amount: the checks of `TransferBuilder::build` hold.
        debug_assert!(
            payment.from() != payment.to() && payment.amount().is_positive(),
            "a payment is between two members and positive"
        );
        Self {
            id: TransferId::generate(),
            amount: payment.amount(),
            date,
            from: payment.from(),
            to: payment.to(),
            exchange_rate: None,
            note: None,
            time: None,
        }
    }

    /// Starts a builder holding a copy of this transfer, to edit it; the
    /// copy keeps the identifier.
    pub fn to_builder(&self) -> TransferBuilder {
        TransferBuilder {
            id: Some(self.id),
            cost: self.cost(),
            date: self.date,
            from: self.from,
            to: self.to,
            note: self.note.clone(),
            time: self.time,
        }
    }

    /// The identifier.
    #[must_use]
    pub const fn id(&self) -> TransferId {
        self.id
    }

    /// The amount, in the transfer's own currency.
    #[must_use]
    pub const fn amount(&self) -> Money {
        self.amount
    }

    /// The cost: the amount and, for a transfer in another currency than
    /// its group's, the rate into the group currency.
    #[must_use]
    pub const fn cost(&self) -> Cost {
        Cost::from_parts(self.amount, self.exchange_rate)
    }

    /// The date of the transfer.
    #[must_use]
    pub const fn date(&self) -> CalendarDate {
        self.date
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

    /// The rate from the transfer's currency into its group's currency:
    /// [`ExchangeRate::ONE`] for a local cost.
    #[must_use]
    pub const fn exchange_rate(&self) -> ExchangeRate {
        self.cost().exchange_rate()
    }

    /// The same transfer at `exchange_rate` into its group's currency, or as
    /// a local cost with `None`, identifier included: what a group does to
    /// its transfers when it changes currency.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{CalendarDate, Currency, ExchangeRate, Money, Transfer};
    /// use eranos::id::MemberId;
    ///
    /// let (ana, bob) = (MemberId::generate(), MemberId::generate());
    /// let (amount, day) = (Money::parse("30.00", Currency::EUR)?, CalendarDate::new(24, 9, 2026)?);
    /// let repayment = Transfer::builder(amount, day, bob, ana).build()?;
    /// let rate = ExchangeRate::parse("4.97")?;
    /// assert_eq!(repayment.with_exchange_rate(Some(rate)).cost(), amount.at(rate));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn with_exchange_rate(mut self, exchange_rate: Option<ExchangeRate>) -> Self {
        self.exchange_rate = exchange_rate;
        self
    }

    /// A note, if any.
    #[must_use]
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    /// The time of day, if recorded.
    #[must_use]
    pub const fn time(&self) -> Option<TimeOfDay> {
        self.time
    }
}

/// Builds a [`Transfer`]; created by [`Transfer::builder`] or
/// [`Transfer::to_builder`].
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct TransferBuilder {
    id: Option<TransferId>,
    cost: Cost,
    date: CalendarDate,
    from: MemberId,
    to: MemberId,
    note: Option<String>,
    time: Option<TimeOfDay>,
}

impl TransferBuilder {
    /// Uses `id` instead of a generated identifier, so a test can compare a
    /// built transfer with one made elsewhere.
    #[cfg(test)]
    pub(crate) const fn with_id(mut self, id: TransferId) -> Self {
        self.id = Some(id);
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
    pub const fn date(mut self, date: CalendarDate) -> Self {
        self.date = date;
        self
    }

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

    /// Sets the note; blank text clears it.
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Sets the time of day, or clears it with `None`.
    pub fn time(mut self, time: impl Into<Option<TimeOfDay>>) -> Self {
        self.time = time.into();
        self
    }

    /// Validates the transfer and returns it, under a newly generated
    /// identifier unless the builder came from [`Transfer::to_builder`].
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
            id: self.id.unwrap_or_else(TransferId::generate),
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
    id: TransferId,
    amount: Money,
    date: CalendarDate,
    from: MemberId,
    to: MemberId,
    #[serde(default)]
    exchange_rate: Option<ExchangeRate>,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    time: Option<TimeOfDay>,
}

/// Builds the transfer `record` describes, through the checks of the builder.
#[cfg(feature = "serde")]
fn from_record(record: TransferRecord) -> Result<Transfer, TransferError> {
    let cost = Cost::from_parts(record.amount, record.exchange_rate);
    let builder = TransferBuilder {
        id: Some(record.id),
        ..Transfer::builder(cost, record.date, record.from, record.to)
    }
    .time(record.time);
    match record.note {
        Some(note) => builder.note(note).build(),
        None => builder.build(),
    }
}

// Written out rather than derived with `serde(try_from)`: its `TryFrom` impl
// from the private record would appear on the facade's page for `Transfer`.
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Transfer {
    fn deserialize<D: serde::Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        from_record(TransferRecord::deserialize(deserialiser)?).map_err(serde::de::Error::custom)
    }
}

/// A transfer's amount in the group currency `currency`, in minor units.
///
/// The group has checked the kind of the transfer's cost against
/// `currency` (`Group::check_cost`, or the rates of a currency change).
///
/// # Errors
///
/// [`MoneyError::Overflow`] when the conversion leaves the range of an
/// amount.
pub fn resolve_transfer(transfer: &Transfer, currency: Currency) -> Result<i64, MoneyError> {
    match transfer.cost().conversion() {
        None => Ok(transfer.amount().minor_units()),
        Some(rate) => Ok(rate.convert(transfer.amount(), currency)?.minor_units()),
    }
}

#[cfg(test)]
mod tests;
