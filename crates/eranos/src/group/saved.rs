//! The saved form of a group: its details and entries, never the derived
//! totals.
//!
//! Loading replays every entry through the same checks as the `add_*`
//! methods, so a hand-edited or corrupted file cannot produce an
//! inconsistent group. Members are replayed unarchived and archived at the
//! end, because an expense may reference a member who was archived after it
//! was added.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{FORMAT, Group};
use crate::category::Category;
use crate::error::GroupError;
use crate::expense::Expense;
use crate::group::Language;
use crate::id::{CategoryId, GroupId, MemberId};
use crate::member::Member;
use crate::money::Currency;
use crate::transfer::Transfer;

#[derive(Serialize)]
struct GroupView<'group> {
    format: u32,
    id: GroupId,
    categories: Vec<&'group Category>,
    currency: Currency,
    expenses: Vec<&'group Expense>,
    members: Vec<&'group Member>,
    name: &'group str,
    transfers: Vec<&'group Transfer>,
    /// The archived members, in identifier order.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    archived: Vec<MemberId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'group str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<&'group Language>,
}

/// A saved group, read but not yet loaded: its details and entries, before
/// the group checks them against each other.
///
/// Deserialising a [`Group`] loads it in one step and reports a refused file
/// as the deserialiser's error, which holds only the refusal's message.
/// Deserialising a `SavedGroup` and calling [`into_group`](Self::into_group)
/// reports the refusal as a [`GroupError`] instead, so an app can tell a
/// file from a newer release ([`GroupError::UnsupportedFormat`]) from a
/// damaged one. Each entry is checked on its own as it is read, since every
/// entity deserialises through its validating constructor: an entry that
/// could never have been built fails the read itself.
///
/// # Examples
///
/// ```
/// use eranos::error::GroupError;
/// use eranos::{Currency, Group, SavedGroup};
///
/// let group = Group::builder(Currency::EUR, "Lisbon").build()?;
/// let json = serde_json::to_string(&group)?;
///
/// // A file this version wrote loads.
/// let saved: SavedGroup = serde_json::from_str(&json)?;
/// assert_eq!(saved.into_group()?.id(), group.id());
///
/// // A file from a later release is refused, with the format it names.
/// let newer: SavedGroup = serde_json::from_str(&json.replace(r#""format":2"#, r#""format":3"#))?;
/// assert_eq!(
///     newer.into_group().map(|group| group.id()),
///     Err(GroupError::UnsupportedFormat { format: 3 })
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct SavedGroup {
    format: u32,
    id: GroupId,
    currency: Currency,
    name: String,
    #[serde(default)]
    archived: Vec<MemberId>,
    #[serde(default)]
    categories: Vec<Category>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    expenses: Vec<Expense>,
    #[serde(default)]
    language: Option<Language>,
    #[serde(default)]
    members: Vec<Member>,
    #[serde(default)]
    transfers: Vec<Transfer>,
}

impl Serialize for Group {
    fn serialize<S: Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
        GroupView {
            format: FORMAT,
            id: self.id,
            // Every group has the built-in category, so it is not written.
            categories: self
                .categories
                .values()
                .filter(|category| category.id() != CategoryId::UNCATEGORISED)
                .collect(),
            currency: self.currency,
            expenses: self
                .expenses
                .values()
                .map(|record| &record.expense)
                .collect(),
            members: self.members.values().map(|record| &record.member).collect(),
            name: &self.name,
            transfers: self
                .transfers
                .values()
                .map(|record| &record.transfer)
                .collect(),
            archived: self
                .members
                .iter()
                .filter(|(_, record)| record.archived)
                .map(|(&id, _)| id)
                .collect(),
            description: self.description.as_deref(),
            language: self.language.as_ref(),
        }
        .serialize(serialiser)
    }
}

impl<'de> Deserialize<'de> for Group {
    fn deserialize<D: Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        SavedGroup::deserialize(deserialiser)?
            .into_group()
            .map_err(D::Error::custom)
    }
}

impl SavedGroup {
    /// Loads the group, replaying its entries through the same checks as
    /// the `add_*` methods.
    ///
    /// # Errors
    ///
    /// - [`GroupError::UnsupportedFormat`] when the file names another
    ///   format than the one this version writes.
    /// - [`GroupError::EmptyName`] when the name is blank.
    /// - The errors of [`Group::add_member`], [`Group::add_category`],
    ///   [`Group::add_expense`], [`Group::add_transfer`], and
    ///   [`Group::archive_member`] for an entry that does not fit the
    ///   others: a repeated identifier or category name, an unknown or
    ///   archived member, an unknown category, or a missing exchange rate.
    pub fn into_group(self) -> Result<Group, GroupError> {
        if self.format != FORMAT {
            return Err(GroupError::UnsupportedFormat {
                format: self.format,
            });
        }
        let mut builder = Group::builder(self.currency, self.name).language(self.language);
        if let Some(description) = self.description {
            builder = builder.description(description);
        }
        let mut group = builder.build()?;
        group.id = self.id;
        for member in self.members {
            group.add_member(member)?;
        }
        for category in self.categories {
            group.add_category(category)?;
        }
        for expense in self.expenses {
            group.add_expense(expense)?;
        }
        for transfer in self.transfers {
            group.add_transfer(transfer)?;
        }
        for member in self.archived {
            group.archive_member(member)?;
        }
        Ok(group)
    }
}
