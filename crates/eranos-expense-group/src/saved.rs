//! The saved form of a group: its details and entries, never the derived
//! totals.
//!
//! Loading checks every entry as the `add_*` methods do, so a hand-edited
//! or corrupted file cannot produce an inconsistent group, and range-checks
//! the totals once, after the last entry: the entries replay in the order
//! of their identifiers, not the order the group was built in, and the
//! sums between two of them can leave the range where the group's never
//! did. Members are replayed unarchived and archived at the end, because
//! an expense may reference a member who was archived after it was added.

use std::future::Future;

use eranos_category::Category;
use eranos_core::cooperative::{Cooperative, Pacer};
use eranos_core::{CategoryId, ExpenseGroupId, MemberId};
use eranos_expense::Expense;
use eranos_ledger::Delta;
use eranos_member::Member;
use eranos_money::CurrencyCode;
use eranos_transfer::Transfer;
use serde::de::{Error as _, IgnoredAny};
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
/// damaged one: the reader stops at a `format` that is not this version's,
/// wherever the key stands, so a newer file reports its format whatever
/// its shape. Each entry is checked on its own as it is read, since every
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
/// let newer: SavedExpenseGroup = serde_json::from_str(&json.replace(r#""format":4"#, r#""format":5"#))?;
/// assert_eq!(
///     newer.into_expense_group().map(|group| group.id()),
///     Err(ExpenseGroupError::UnsupportedFormat { format: 5 })
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct SavedExpenseGroup {
    format: u32,
    /// The contents, read only from a file of the format this version
    /// writes: `None` names a file of another format.
    contents: Option<SavedContents>,
}

/// The details and entries of a saved group of the format this version
/// writes.
#[derive(Debug, Clone)]
struct SavedContents {
    id: ExpenseGroupId,
    name: String,
    currency: CurrencyCode,
    description: Option<String>,
    language: Option<Language>,
    members: Vec<Member>,
    archived: Vec<MemberId>,
    categories: Vec<Category>,
    expenses: Vec<Expense>,
    transfers: Vec<Transfer>,
}

/// The keys of a saved group; any other key is skipped, so a file of this
/// format written by a later release, with a key this version does not
/// know, still loads.
#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum Key {
    Format,
    Id,
    Name,
    Currency,
    Description,
    Language,
    Members,
    Archived,
    Categories,
    Expenses,
    Transfers,
    #[serde(other)]
    Other,
}

/// Sets `slot` to `value`, or refuses a key the file repeats.
fn once<T, E: serde::de::Error>(
    slot: &mut Option<T>,
    key: &'static str,
    value: T,
) -> Result<(), E> {
    if slot.is_some() {
        return Err(E::duplicate_field(key));
    }
    *slot = Some(value);
    Ok(())
}

/// Reads a saved group key by key, and stops at `format` when it names
/// another format than this version writes: the rest of such a file may
/// have a shape this version does not know, and reading it would report a
/// shape, where the format is what the app must learn.
struct SavedVisitor;

impl<'de> serde::de::Visitor<'de> for SavedVisitor {
    type Value = SavedExpenseGroup;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a saved group")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut format = None;
        let mut id = None;
        let mut name = None;
        let mut currency = None;
        let mut description = None;
        let mut language = None;
        let mut members = None;
        let mut archived = None;
        let mut categories = None;
        let mut expenses = None;
        let mut transfers = None;
        while let Some(key) = map.next_key::<Key>()? {
            match key {
                Key::Format => {
                    let value = map.next_value::<u32>()?;
                    once(&mut format, "format", value)?;
                    if FORMAT != value {
                        while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                        return Ok(SavedExpenseGroup {
                            format: value,
                            contents: None,
                        });
                    }
                }
                Key::Id => once(&mut id, "id", map.next_value()?)?,
                Key::Name => once(&mut name, "name", map.next_value()?)?,
                Key::Currency => once(&mut currency, "currency", map.next_value()?)?,
                Key::Description => once(&mut description, "description", map.next_value()?)?,
                Key::Language => once(&mut language, "language", map.next_value()?)?,
                Key::Members => once(&mut members, "members", map.next_value()?)?,
                Key::Archived => once(&mut archived, "archived", map.next_value()?)?,
                Key::Categories => once(&mut categories, "categories", map.next_value()?)?,
                Key::Expenses => once(&mut expenses, "expenses", map.next_value()?)?,
                Key::Transfers => once(&mut transfers, "transfers", map.next_value()?)?,
                Key::Other => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(SavedExpenseGroup {
            format: format.ok_or_else(|| A::Error::missing_field("format"))?,
            contents: Some(SavedContents {
                id: id.ok_or_else(|| A::Error::missing_field("id"))?,
                name: name.ok_or_else(|| A::Error::missing_field("name"))?,
                currency: currency.ok_or_else(|| A::Error::missing_field("currency"))?,
                description: description.unwrap_or_default(),
                language: language.unwrap_or_default(),
                members: members.unwrap_or_default(),
                archived: archived.unwrap_or_default(),
                categories: categories.unwrap_or_default(),
                expenses: expenses.unwrap_or_default(),
                transfers: transfers.unwrap_or_default(),
            }),
        })
    }
}

impl<'de> Deserialize<'de> for SavedExpenseGroup {
    fn deserialize<D: Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        deserialiser.deserialize_map(SavedVisitor)
    }
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
    /// Loads the group, checking its entries as the `add_*` methods do and
    /// range-checking the totals once, after the last.
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
    /// - [`ExpenseGroupError::Overflow`] when the totals of every entry
    ///   together would leave the range of an amount.
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
        let Some(contents) = self.contents else {
            return Err(ExpenseGroupError::UnsupportedFormat {
                format: self.format,
            });
        };
        let mut builder =
            ExpenseGroup::builder_with_id(contents.id, contents.name, contents.currency);
        if let Some(description) = contents.description {
            builder = builder.description(description);
        }
        let mut group = builder.language(contents.language).build()?;
        let mut pacer = Pacer::new(ENTRIES_PER_YIELD);
        for member in contents.members {
            group.add_member(member)?;
            pacer.advance(1).await;
        }
        for category in contents.categories {
            group.add_category(category)?;
            pacer.advance(1).await;
        }
        let mut delta = Delta::default();
        for expense in contents.expenses {
            group.stage_expense(expense, &mut delta)?;
            pacer.advance(1).await;
        }
        for transfer in contents.transfers {
            group.stage_transfer(transfer, &mut delta)?;
            pacer.advance(1).await;
        }
        group.apply_staged(&delta)?;
        for member in contents.archived {
            group.archive_member(member)?;
            pacer.advance(1).await;
        }
        Ok(group)
    }
}
