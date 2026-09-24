//! The serialised form of a group: its details and entries, never the
//! derived totals.
//!
//! Deserialising replays every entry through the same checks as the `add_*`
//! methods, so a hand-edited or corrupted file cannot produce an
//! inconsistent group. Members are replayed unarchived and archived at the
//! end, because an expense may reference a member who was archived after it
//! was added.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Group;
use crate::category::Category;
use crate::error::{GroupError, ValueError};
use crate::expense::Expense;
use crate::group::Language;
use crate::id::{CategoryId, GroupId, MemberId};
use crate::member::Member;
use crate::money::Currency;
use crate::transfer::Transfer;

/// The version of the serialised form. Every other version is refused: a
/// newer one would be read with fields silently dropped, and no earlier one
/// is read.
const FORMAT: u32 = 2;

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

#[derive(Deserialize)]
struct GroupRecord {
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
        GroupRecord::deserialize(deserialiser)?
            .replay()
            .map_err(D::Error::custom)
    }
}

/// Why a well-formed file does not make a valid group.
#[derive(Debug, thiserror::Error)]
enum ReplayError {
    /// Invalid group details.
    #[error(transparent)]
    Details(#[from] ValueError),
    /// An entry the group refuses.
    #[error(transparent)]
    Entry(#[from] GroupError),
    /// A format this version cannot read.
    #[error("unsupported group format {0}: this version reads format {FORMAT}")]
    Format(u32),
}

impl GroupRecord {
    /// Rebuilds the group through the same checks as the `add_*` methods.
    ///
    /// Not generic over the deserialiser, so it is compiled once rather
    /// than for every format the apps read.
    fn replay(self) -> Result<Group, ReplayError> {
        if self.format != FORMAT {
            return Err(ReplayError::Format(self.format));
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
