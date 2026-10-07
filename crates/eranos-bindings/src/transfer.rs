//! Transfers between members, as the bindings carry them.

use eranos::TransferId;
use eranos::transfer as library;
use eranos_money::FromCost;

use crate::calendar::{CalendarDate, TimeOfDay};
use crate::money::Cost;
use crate::{EranosError, identifier, identifier_to_add};

/// Money one member paid another, such as a repayment.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transfer {
    /// The identifier: the app's own for a transfer to add, or absent for the
    /// library to generate one, and present for a transfer the group holds or
    /// one to replace.
    pub id: Option<String>,
    /// The member who paid, as their identifier.
    pub from: String,
    /// The member who was paid, as their identifier.
    pub to: String,
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
            id: Some(transfer.id().to_string()),
            from: transfer.from().to_string(),
            to: transfer.to().to_string(),
            cost: Cost::from_cost(transfer.cost()),
            date: CalendarDate::from_library(transfer.date()),
            time: transfer.time().map(TimeOfDay::from_library),
            note: transfer.note().map(str::to_owned),
        }
    }

    /// The library's transfer, for a record to add: under the record's
    /// identifier, or a generated one when it has none.
    ///
    /// # Errors
    ///
    /// An [`EranosError::Id`] for an identifier the rules refuse, the errors
    /// of the fields' conversions, and those of
    /// [`TransferBuilder::build`](library::TransferBuilder::build).
    pub(crate) fn to_new(&self) -> Result<library::Transfer, EranosError> {
        let id: Option<TransferId> = identifier_to_add(self.id.as_deref())?;
        let builder = library::Transfer::builder(
            identifier(&self.from)?,
            identifier(&self.to)?,
            self.cost.to_library()?,
            self.date.to_library()?,
        )
        .id(id);
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
            .from(identifier(&self.from)?)
            .to(identifier(&self.to)?)
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
        Ok(builder
            .time(time)
            .note(self.note.clone().unwrap_or_default()))
    }
}

#[cfg(test)]
mod tests;
