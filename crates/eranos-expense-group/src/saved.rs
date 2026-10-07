//! The saved form of a group: its details and entries, never the derived
//! totals.
//!
//! Loading replays every entry through the same checks as the `add_*`
//! methods, so a hand-edited or corrupted file cannot produce an
//! inconsistent group. Members are replayed unarchived and archived at the
//! end, because an expense may reference a member who was archived after it
//! was added.

use std::future::Future;

use eranos_category::Category;
use eranos_core::cooperative::{Cooperative, Pacer};
use eranos_core::{CategoryId, ExpenseGroupId, MemberId};
use eranos_expense::Expense;
use eranos_member::Member;
use eranos_money::CurrencyCode;
use eranos_transfer::Transfer;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{ENTRIES_PER_YIELD, ExpenseGroup, FORMAT};
use crate::Language;
use crate::error::ExpenseGroupError;

#[derive(Serialize)]
struct ExpenseGroupView<'group> {
    format: u32,
    id: ExpenseGroupId,
    name: &'group str,
    currency: CurrencyCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'group str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<&'group Language>,
    members: Vec<&'group Member>,
    /// The archived members, in identifier order.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    archived: Vec<MemberId>,
    categories: Vec<&'group Category>,
    expenses: Vec<&'group Expense>,
    transfers: Vec<&'group Transfer>,
}

/// A saved group, read but not yet loaded: its details and entries, before
/// the group checks them against each other.
///
/// Deserialising a [`ExpenseGroup`] loads it in one step and reports a refused file
/// as the deserialiser's error, which holds only the refusal's message.
/// Deserialising a `SavedExpenseGroup` and calling [`into_expense_group`](Self::into_expense_group)
/// reports the refusal as a [`ExpenseGroupError`] instead, so an app can tell a
/// file from a newer release ([`ExpenseGroupError::UnsupportedFormat`]) from a
/// damaged one. Each entry is checked on its own as it is read, since every
/// entity deserialises through its validating constructor: an entry that
/// could never have been built fails the read itself.
///
/// # Examples
///
/// ```
/// use eranos::error::ExpenseGroupError;
/// use eranos::{CurrencyCode, ExpenseGroup};
/// # // The facade re-exports `SavedExpenseGroup` only with its own `serde` feature,
/// # // which a test of this crate alone does not turn on.
/// use eranos_expense_group::SavedExpenseGroup;
///
/// let group = ExpenseGroup::builder("Lisbon", CurrencyCode::EUR).build()?;
/// let json = serde_json::to_string(&group)?;
///
/// // A file this version wrote loads.
/// let saved: SavedExpenseGroup = serde_json::from_str(&json)?;
/// assert_eq!(saved.into_expense_group()?.id(), group.id());
///
/// // A file from a later release is refused, with the format it names.
/// let newer: SavedExpenseGroup = serde_json::from_str(&json.replace(r#""format":2"#, r#""format":3"#))?;
/// assert_eq!(
///     newer.into_expense_group().map(|group| group.id()),
///     Err(ExpenseGroupError::UnsupportedFormat { format: 3 })
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct SavedExpenseGroup {
    format: u32,
    id: ExpenseGroupId,
    name: String,
    currency: CurrencyCode,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    language: Option<Language>,
    #[serde(default)]
    members: Vec<Member>,
    #[serde(default)]
    archived: Vec<MemberId>,
    #[serde(default)]
    categories: Vec<Category>,
    #[serde(default)]
    expenses: Vec<Expense>,
    #[serde(default)]
    transfers: Vec<Transfer>,
}

impl Serialize for ExpenseGroup {
    fn serialize<S: Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
        ExpenseGroupView {
            format: FORMAT,
            id: self.id,
            name: &self.name,
            currency: self.currency,
            description: self.description.as_deref(),
            language: self.language.as_ref(),
            members: self.members.values().map(|record| &record.member).collect(),
            archived: self
                .members
                .iter()
                .filter(|(_, record)| record.archived)
                .map(|(&id, _)| id)
                .collect(),
            // Every group has the built-in category, so it is not written.
            categories: self
                .categories
                .values()
                .filter(|category| CategoryId::UNCATEGORISED != category.id())
                .collect(),
            expenses: self
                .expenses
                .values()
                .map(|record| &record.expense)
                .collect(),
            transfers: self
                .transfers
                .values()
                .map(|record| &record.transfer)
                .collect(),
        }
        .serialize(serialiser)
    }
}

impl<'de> Deserialize<'de> for ExpenseGroup {
    fn deserialize<D: Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        SavedExpenseGroup::deserialize(deserialiser)?
            .into_expense_group()
            .map_err(D::Error::custom)
    }
}

impl SavedExpenseGroup {
    /// Loads the group, replaying its entries through the same checks as
    /// the `add_*` methods.
    ///
    /// # Errors
    ///
    /// - [`ExpenseGroupError::UnsupportedFormat`] when the file names another
    ///   format than the one this version writes.
    /// - [`ExpenseGroupError::EmptyName`] when the name is blank.
    /// - The errors of [`ExpenseGroup::add_member`], [`ExpenseGroup::add_category`],
    ///   [`ExpenseGroup::add_expense`], [`ExpenseGroup::add_transfer`], and
    ///   [`ExpenseGroup::archive_member`] for an entry that does not fit the
    ///   others: a repeated identifier or category name, an unknown or
    ///   archived member, an unknown category, or a missing exchange rate.
    pub fn into_expense_group(self) -> Result<ExpenseGroup, ExpenseGroupError> {
        self.into_expense_group_async().complete()
    }

    /// Loads the group as [`into_expense_group`](Self::into_expense_group) does, yielding
    /// while it replays the entries.
    ///
    /// It yields once per 1 024 replayed steps: a member, a category, an
    /// expense, a transfer, or an archiving. The call owns the saved form,
    /// so it can move to another task or thread, and a call dropped before
    /// it completes leaves nothing behind; a copy of the `SavedExpenseGroup` can
    /// start the load again.
    ///
    /// # Errors
    ///
    /// As for [`into_expense_group`](Self::into_expense_group).
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::cooperative::block_on;
    /// use eranos::{CurrencyCode, ExpenseGroup};
    /// # // The facade re-exports `SavedExpenseGroup` only with its own `serde` feature,
    /// # // which a test of this crate alone does not turn on.
    /// use eranos_expense_group::SavedExpenseGroup;
    ///
    /// let group = ExpenseGroup::builder("Lisbon", CurrencyCode::EUR).build()?;
    /// let saved: SavedExpenseGroup = serde_json::from_str(&serde_json::to_string(&group)?)?;
    ///
    /// // Awaited like any future; `block_on` runs it on this thread.
    /// assert_eq!(block_on(saved.into_expense_group_async())?, group);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn into_expense_group_async(
        self,
    ) -> Cooperative<impl Future<Output = Result<ExpenseGroup, ExpenseGroupError>> + Send + 'static>
    {
        Cooperative::new(self.into_expense_group_paced())
    }

    /// The computation of [`into_expense_group_async`](Self::into_expense_group_async).
    async fn into_expense_group_paced(self) -> Result<ExpenseGroup, ExpenseGroupError> {
        if FORMAT != self.format {
            return Err(ExpenseGroupError::UnsupportedFormat {
                format: self.format,
            });
        }
        let mut builder = ExpenseGroup::builder(self.name, self.currency);
        if let Some(description) = self.description {
            builder = builder.description(description);
        }
        let mut group = builder.language(self.language).build()?;
        group.id = self.id;
        let mut pacer = Pacer::new(ENTRIES_PER_YIELD);
        for member in self.members {
            group.add_member(member)?;
            pacer.advance(1).await;
        }
        for category in self.categories {
            group.add_category(category)?;
            pacer.advance(1).await;
        }
        for expense in self.expenses {
            group.add_expense(expense)?;
            pacer.advance(1).await;
        }
        for transfer in self.transfers {
            group.add_transfer(transfer)?;
            pacer.advance(1).await;
        }
        for member in self.archived {
            group.archive_member(member)?;
            pacer.advance(1).await;
        }
        Ok(group)
    }
}
