//! The serialized form of a group: its details and entries, never the
//! derived totals.
//!
//! Deserializing replays every entry through the same checks as the insert
//! methods, so a hand-edited or corrupted file cannot produce an
//! inconsistent group. Members are replayed unarchived and archived at the
//! end, because an expense may reference a member who was archived after it
//! was added.

use std::collections::BTreeSet;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Group;
use crate::category::Category;
use crate::error::{GroupError, ValueError};
use crate::expense::Expense;
use crate::group::Language;
use crate::id::{CategoryId, ExpenseId, GroupId, MemberId, TransferId};
use crate::member::Member;
use crate::money::Currency;
use crate::transfer::Transfer;

/// The version of the serialized form. A newer one is rejected rather than
/// read with fields silently dropped.
const FORMAT: u32 = 1;

#[derive(Serialize)]
struct GroupView<'group> {
    format: u32,
    id: GroupId,
    name: &'group str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'group str>,
    currency: Currency,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<&'group Language>,
    members: Vec<MemberView<'group>>,
    categories: Vec<Entry<CategoryId, &'group Category>>,
    expenses: Vec<Entry<ExpenseId, &'group Expense>>,
    transfers: Vec<Entry<TransferId, &'group Transfer>>,
}

#[derive(Serialize)]
struct MemberView<'group> {
    id: MemberId,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    archived: bool,
    member: &'group Member,
}

#[derive(Serialize, Deserialize)]
struct Entry<Id, Value> {
    id: Id,
    #[serde(rename = "entry")]
    value: Value,
}

#[derive(Deserialize)]
struct GroupRecord {
    format: u32,
    id: GroupId,
    name: String,
    #[serde(default)]
    description: Option<String>,
    currency: Currency,
    #[serde(default)]
    language: Option<Language>,
    #[serde(default)]
    members: Vec<MemberRecord>,
    #[serde(default)]
    categories: Vec<Entry<CategoryId, Category>>,
    #[serde(default)]
    expenses: Vec<Entry<ExpenseId, Expense>>,
    #[serde(default)]
    transfers: Vec<Entry<TransferId, Transfer>>,
}

#[derive(Deserialize)]
struct MemberRecord {
    id: MemberId,
    #[serde(default)]
    archived: bool,
    member: Member,
}

impl Serialize for Group {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        GroupView {
            format: FORMAT,
            id: self.id,
            name: &self.name,
            description: self.description.as_deref(),
            currency: self.currency,
            language: self.language.as_ref(),
            members: self
                .members
                .iter()
                .map(|(&id, record)| MemberView {
                    id,
                    archived: record.archived,
                    member: &record.member,
                })
                .collect(),
            categories: self
                .categories
                .iter()
                .map(|(&id, value)| Entry { id, value })
                .collect(),
            expenses: self
                .expenses
                .iter()
                .map(|(&id, record)| Entry {
                    id,
                    value: &record.expense,
                })
                .collect(),
            transfers: self
                .transfers
                .iter()
                .map(|(&id, record)| Entry {
                    id,
                    value: &record.transfer,
                })
                .collect(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Group {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        GroupRecord::deserialize(deserializer)?
            .replay()
            .map_err(D::Error::custom)
    }
}

/// Why a well-formed file does not make a valid group.
#[derive(Debug, thiserror::Error)]
enum ReplayError {
    /// A format this version cannot read.
    #[error("unsupported group format {0}: this version reads format {FORMAT}")]
    Format(u32),
    /// Invalid group details.
    #[error(transparent)]
    Details(#[from] ValueError),
    /// An entry the group refuses.
    #[error(transparent)]
    Entry(#[from] GroupError),
}

impl GroupRecord {
    /// Rebuilds the group through the same checks as the insert methods.
    ///
    /// Not generic over the deserializer, so it is compiled once rather
    /// than for every format the apps read.
    fn replay(self) -> Result<Group, ReplayError> {
        if self.format != FORMAT {
            return Err(ReplayError::Format(self.format));
        }
        let mut builder = Group::builder(self.name, self.currency)
            .id(self.id)
            .language(self.language);
        if let Some(description) = self.description {
            builder = builder.description(description);
        }
        let mut group = builder.build()?;
        let archived: BTreeSet<MemberId> = self
            .members
            .iter()
            .filter(|member| member.archived)
            .map(|member| member.id)
            .collect();
        for member in self.members {
            group.insert_member(member.id, member.member)?;
        }
        for Entry { id, value } in self.categories {
            group.insert_category(id, value)?;
        }
        for Entry { id, value } in self.expenses {
            group.insert_expense(id, value)?;
        }
        for Entry { id, value } in self.transfers {
            group.insert_transfer(id, value)?;
        }
        // Every ID in `archived` is one of the members inserted above.
        for (id, member) in &mut group.members {
            member.archived = archived.contains(id);
        }
        Ok(group)
    }
}
