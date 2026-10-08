//! Transfers between members, as the bindings carry them.

use eranos::transfer as library;
use eranos_money::FromCost;

use crate::calendar::{CalendarDate, TimeOfDay};
use crate::money::Cost;
use crate::{EranosError, identifier};

/// Money one member paid another, such as a repayment.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transfer {
    /// The identifier, which the app gives: its own, or one
    /// [`identifier_generate`](crate::identifier_generate) returns.
    pub id: String,
    /// The member who paid, as their identifier.
    pub sender: String,
    /// The member who was paid, as their identifier.
    pub receiver: String,
    /// What was paid.
    pub cost: Cost,
    /// The day it was paid.
    pub date: CalendarDate,
    /// The time of day it was paid.
    pub time: Option<TimeOfDay>,
    /// A note on the transfer.
    pub note: Option<String>,
}

impl Transfer {
    /// The record of `transfer`.
    pub(crate) fn from_library(transfer: &library::Transfer) -> Self {
        Self {
            id: transfer.id().to_string(),
            sender: transfer.sender().to_string(),
            receiver: transfer.receiver().to_string(),
            cost: Cost::from_cost(transfer.cost()),
            date: CalendarDate::from_library(transfer.date()),
            time: transfer.time().map(TimeOfDay::from_library),
            note: transfer.note().map(str::to_owned),
        }
    }

    /// The library's transfer, for a record to add, under the record's
    /// identifier.
    ///
    /// # Errors
    ///
    /// An [`EranosError::Id`] for an identifier the rules refuse, the errors
    /// of the fields' conversions, and those of
    /// [`TransferBuilder::build`](library::TransferBuilder::build).
    pub(crate) fn to_new(&self) -> Result<library::Transfer, EranosError> {
        let builder = library::Transfer::builder_with_id(
            identifier(&self.id)?,
            identifier(&self.sender)?,
            identifier(&self.receiver)?,
            self.cost.to_library()?,
            self.date.to_library()?,
        );
        Ok(self.with_optional_fields(builder)?.build()?)
    }

    /// The library's transfer that replaces `existing`, keeping its
    /// identifier, with every field of the record.
    ///
    /// # Errors
    ///
    /// The errors of the fields' conversions and those of
    /// [`TransferBuilder::build`](library::TransferBuilder::build).
    pub(crate) fn replacing(
        &self,
        existing: &library::Transfer,
    ) -> Result<library::Transfer, EranosError> {
        let builder = existing
            .to_builder()
            .sender(identifier(&self.sender)?)
            .receiver(identifier(&self.receiver)?)
            .cost(self.cost.to_library()?)
            .date(self.date.to_library()?);
        Ok(self.with_optional_fields(builder)?.build()?)
    }

    /// `builder` with the record's time and note, each cleared when the
    /// record has none.
    fn with_optional_fields(
        &self,
        builder: library::TransferBuilder,
    ) -> Result<library::TransferBuilder, EranosError> {
        let time = self.time.map(TimeOfDay::to_library).transpose()?;
        let builder = builder.time(time);
        Ok(match &self.note {
            Some(note) => builder.note(note.as_str()),
            None => builder.clear_note(),
        })
    }
}

#[cfg(test)]
mod tests;
