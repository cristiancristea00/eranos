//! Transfers: money one member paid another, such as a repayment.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use. [`resolve_transfer`] and [`resolve_transfer_at`], which
//! convert a transfer into the group currency for the ledger, are the group's,
//! and the facade does not re-export them; nor does it re-export
//! [`FromTransferError`], through which a
//! crate that tells every failure apart converts one without the wildcard arm
//! a `match` outside this crate would need.
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

mod error;

use eranos_calendar::{CalendarDate, TimeOfDay};
use eranos_core::{MemberId, TransferId, text};
use eranos_money::{Cost, CurrencyCode, ExchangeRate, Money, MoneyError};
use eranos_settlement::Payment;

pub use error::{FromTransferError, TransferError};

/// Money one member paid directly to another, usually to settle up.
///
/// A transfer moves balances: the sender's debt shrinks and the recipient's
/// credit shrinks. It is not spending, so it counts towards no total or
/// category. Like an expense, it has a [`Cost`]: a transfer in another
/// currency than its group's carries the [`ExchangeRate`] into the group
/// currency. A transfer has the identifier given to
/// [`builder_with_id`](Self::builder_with_id) or
/// [`builder_from_payment_with_id`](Self::builder_from_payment_with_id), or
/// one that [`builder`](Self::builder) or
/// [`builder_from_payment`](Self::builder_from_payment) generates, and keeps
/// it through every edit.
///
/// # Examples
///
/// ```
/// use eranos::{CurrencyCode, CalendarDate, Money, Transfer};
/// use eranos::id::MemberId;
///
/// let (ana, bob) = (MemberId::generate(), MemberId::generate());
/// let repayment = Transfer::builder(bob, ana, Money::parse("30.00", CurrencyCode::EUR)?, CalendarDate::new(2026, 9, 24)?)
///     .note("Dinner")
///     .build()?;
/// assert_eq!(repayment.from(), bob);
/// assert!(Transfer::builder(ana, ana, repayment.amount(), repayment.date()).build().is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Transfer {
    id: TransferId,
    from: MemberId,
    to: MemberId,
    amount: Money,
    date: CalendarDate,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    exchange_rate: Option<ExchangeRate>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    time: Option<TimeOfDay>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    note: Option<String>,
}

impl Transfer {
    /// Starts a transfer from `from` to `to` of `cost` on `date`, under an
    /// identifier generated when it is called;
    /// [`builder_with_id`](Self::builder_with_id) takes one instead.
    ///
    /// A [`Money`] amount is a local cost, in the group currency;
    /// [`Money::at`] makes a converted one.
    pub fn builder(
        from: MemberId,
        to: MemberId,
        cost: impl Into<Cost>,
        date: CalendarDate,
    ) -> TransferBuilder {
        Self::builder_with_id(TransferId::generate(), from, to, cost, date)
    }

    /// Starts a transfer from `from` to `to` of `cost` on `date`, under the
    /// identifier `id`, such as the key an app's own store gives it;
    /// [`builder`](Self::builder) generates one instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{CalendarDate, CurrencyCode, Money, Transfer};
    /// use eranos::id::{MemberId, TransferId};
    ///
    /// let (ana, bob) = (MemberId::generate(), MemberId::generate());
    /// let (amount, date) = (Money::parse("30.00", CurrencyCode::EUR)?, CalendarDate::new(2026, 9, 24)?);
    /// let repayment = Transfer::builder_with_id(TransferId::new("repayment-1")?, bob, ana, amount, date).build()?;
    /// assert_eq!(repayment.id().as_str(), "repayment-1");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn builder_with_id(
        id: TransferId,
        from: MemberId,
        to: MemberId,
        cost: impl Into<Cost>,
        date: CalendarDate,
    ) -> TransferBuilder {
        TransferBuilder {
            id,
            from,
            to,
            cost: cost.into(),
            date,
            time: None,
            note: None,
        }
    }

    /// Starts the transfer recording `payment`, made on `date`, under an
    /// identifier generated when it is called: from its payer to its payee,
    /// of its amount, as a local cost in the group currency;
    /// [`builder_from_payment_with_id`](Self::builder_from_payment_with_id)
    /// takes the identifier instead.
    ///
    /// Recording every payment of a
    /// [`SettlementPlan`](eranos_settlement::SettlementPlan) brings every
    /// balance to zero. A settlement pays only from one member to another,
    /// and only a positive amount, so `build` accepts what this starts.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{CalendarDate, CurrencyCode, ExpenseGroup, Member, Money, SettlementStrategy, Transfer};
    ///
    /// let mut group = ExpenseGroup::builder("Flat", CurrencyCode::EUR).build()?;
    /// let ana = group.add_member(Member::builder("Ana").build()?)?;
    /// let bob = group.add_member(Member::builder("Bob").build()?)?;
    /// let day = CalendarDate::new(2026, 9, 24)?;
    /// group.add_transfer(Transfer::builder(ana, bob, Money::parse("30.00", CurrencyCode::EUR)?, day).build()?)?;
    ///
    /// let payments = group.settlement(SettlementStrategy::Greedy)?.payments().to_vec();
    /// for payment in &payments {
    ///     group.add_transfer(Transfer::builder_from_payment(payment, day).note("Settled").build()?)?;
    /// }
    /// assert!(group.settlement(SettlementStrategy::Greedy)?.is_empty());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn builder_from_payment(payment: &Payment, date: CalendarDate) -> TransferBuilder {
        Self::builder_from_payment_with_id(TransferId::generate(), payment, date)
    }

    /// Starts the transfer recording `payment`, made on `date`, as
    /// [`builder_from_payment`](Self::builder_from_payment) does, under the
    /// identifier `id`; `builder_from_payment` generates one instead.
    pub fn builder_from_payment_with_id(
        id: TransferId,
        payment: &Payment,
        date: CalendarDate,
    ) -> TransferBuilder {
        Self::builder_with_id(id, payment.from(), payment.to(), payment.amount(), date)
    }

    /// Starts a builder holding a copy of this transfer, to edit it; the
    /// copy keeps the identifier.
    pub fn to_builder(&self) -> TransferBuilder {
        TransferBuilder {
            id: self.id,
            from: self.from,
            to: self.to,
            cost: self.cost(),
            date: self.date,
            time: self.time,
            note: self.note.clone(),
        }
    }

    /// The identifier.
    #[must_use]
    pub const fn id(&self) -> TransferId {
        self.id
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
    /// use eranos::{CalendarDate, CurrencyCode, ExchangeRate, Money, Transfer};
    /// use eranos::id::MemberId;
    ///
    /// let (ana, bob) = (MemberId::generate(), MemberId::generate());
    /// let (amount, day) = (Money::parse("30.00", CurrencyCode::EUR)?, CalendarDate::new(2026, 9, 24)?);
    /// let repayment = Transfer::builder(bob, ana, amount, day).build()?;
    /// let rate = ExchangeRate::parse("4.97")?;
    /// assert_eq!(repayment.with_exchange_rate(Some(rate)).cost(), amount.at(rate));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn with_exchange_rate(mut self, exchange_rate: Option<ExchangeRate>) -> Self {
        self.exchange_rate = exchange_rate;
        self
    }

    /// The time of day, if recorded.
    #[must_use]
    pub const fn time(&self) -> Option<TimeOfDay> {
        self.time
    }

    /// A note, if any.
    #[must_use]
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}

/// Builds a [`Transfer`]; created by [`Transfer::builder`],
/// [`Transfer::builder_from_payment`], their twins that take the
/// identifier, or [`Transfer::to_builder`].
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct TransferBuilder {
    id: TransferId,
    from: MemberId,
    to: MemberId,
    cost: Cost,
    date: CalendarDate,
    time: Option<TimeOfDay>,
    note: Option<String>,
}

impl TransferBuilder {
    /// Replaces the identifier.
    pub const fn id(mut self, id: TransferId) -> Self {
        self.id = id;
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

    /// Sets the time of day, or clears it with `None`.
    pub fn time(mut self, time: impl Into<Option<TimeOfDay>>) -> Self {
        self.time = time.into();
        self
    }

    /// Sets the note, trimmed; blank text leaves it absent, as
    /// [`clear_note`](Self::clear_note) does.
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Clears the note, which [`note`](Self::note) sets.
    pub fn clear_note(mut self) -> Self {
        self.note = None;
        self
    }

    /// Validates the transfer and returns it, under the identifier the
    /// builder holds.
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
            id: self.id,
            from: self.from,
            to: self.to,
            amount,
            date: self.date,
            exchange_rate: self.cost.conversion(),
            time: self.time,
            note: self.note.and_then(text::optional),
        })
    }
}

/// The serialised form of a [`Transfer`], validated on the way in.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct TransferRecord {
    id: TransferId,
    from: MemberId,
    to: MemberId,
    amount: Money,
    date: CalendarDate,
    #[serde(default)]
    exchange_rate: Option<ExchangeRate>,
    #[serde(default)]
    time: Option<TimeOfDay>,
    #[serde(default)]
    note: Option<String>,
}

/// Builds the transfer `record` describes, through the checks of the builder.
#[cfg(feature = "serde")]
fn from_record(record: TransferRecord) -> Result<Transfer, TransferError> {
    let cost = Cost::from_parts(record.amount, record.exchange_rate);
    let builder = Transfer::builder_with_id(record.id, record.from, record.to, cost, record.date)
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
/// `currency` (`ExpenseGroup::check_cost`, or the rates of a currency change).
///
/// # Errors
///
/// [`MoneyError::Overflow`] when the conversion leaves the range of an
/// amount.
pub fn resolve_transfer(transfer: &Transfer, currency: CurrencyCode) -> Result<i64, MoneyError> {
    resolve_transfer_at(transfer, transfer.cost().conversion(), currency)
}

/// A transfer's amount in the group currency, as [`resolve_transfer`] gives
/// it, at the rate `conversion`.
///
/// The rate replaces the transfer's own, and `None` makes it a local cost:
/// a currency change resolves each transfer at its new rate before it gives
/// the transfer that rate.
///
/// The caller has checked the kind of cost that `conversion` makes against
/// `currency`, as a currency change does with its rates.
///
/// # Errors
///
/// [`MoneyError::Overflow`] when the conversion leaves the range of an
/// amount.
pub fn resolve_transfer_at(
    transfer: &Transfer,
    conversion: Option<ExchangeRate>,
    currency: CurrencyCode,
) -> Result<i64, MoneyError> {
    match conversion {
        None => Ok(transfer.amount().minor_units()),
        Some(rate) => Ok(rate.convert(transfer.amount(), currency)?.minor_units()),
    }
}

#[cfg(test)]
mod tests;
