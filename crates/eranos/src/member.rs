//! The people in a group, their contact details, and where they can be paid.

mod account;
mod contact;
mod iban;

pub use account::{Account, AccountDetails, AccountKind, Username};
pub use contact::{Email, PhoneNumber};
pub use iban::Iban;

use crate::error::MemberError;
use crate::id::MemberId;
use crate::text;

/// A person who pays for or shares expenses.
///
/// Only the first name is required. A member gets its identifier when it is
/// built, and keeps it through every edit. Members are values: to change
/// one, edit a copy with [`to_builder`](Self::to_builder) and pass it to
/// [`Group::replace_member`](crate::Group::replace_member).
///
/// # Examples
///
/// ```
/// use eranos::{Account, Email, Iban, Member};
///
/// let ana = Member::builder("Ana")
///     .last_name("Popescu")
///     .email(Email::parse("ana@example.com")?)
///     .account(Account::new(Iban::parse("RO49AAAA1B31007593840000")?, "ING")?)
///     .build()?;
/// assert_eq!(ana.first_name(), "Ana");
/// assert_eq!(ana.accounts().len(), 1);
///
/// let renamed = ana.to_builder().first_name("Anna").build()?;
/// assert_eq!(renamed.last_name(), Some("Popescu"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "MemberRecord")
)]
pub struct Member {
    id: MemberId,
    first_name: String,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    accounts: Vec<Account>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    email: Option<Email>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    last_name: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    phone: Option<PhoneNumber>,
}

impl Member {
    /// Starts a member with the required first name.
    pub fn builder(first_name: impl Into<String>) -> MemberBuilder {
        MemberBuilder {
            id: None,
            first_name: first_name.into(),
            accounts: Vec::new(),
            email: None,
            last_name: None,
            phone: None,
        }
    }

    /// Starts a builder holding a copy of this member, to edit it; the copy
    /// keeps the identifier.
    pub fn to_builder(&self) -> MemberBuilder {
        MemberBuilder {
            id: Some(self.id),
            first_name: self.first_name.clone(),
            accounts: self.accounts.clone(),
            email: self.email.clone(),
            last_name: self.last_name.clone(),
            phone: self.phone.clone(),
        }
    }

    /// The identifier.
    #[must_use]
    pub const fn id(&self) -> MemberId {
        self.id
    }

    /// The first name, the only required field.
    #[must_use]
    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    /// The accounts the member can be paid to, in the order they were added.
    #[must_use]
    pub fn accounts(&self) -> &[Account] {
        &self.accounts
    }

    /// The email address, if any.
    #[must_use]
    pub const fn email(&self) -> Option<&Email> {
        self.email.as_ref()
    }

    /// The last name, if any.
    #[must_use]
    pub fn last_name(&self) -> Option<&str> {
        self.last_name.as_deref()
    }

    /// The phone number, if any.
    #[must_use]
    pub const fn phone(&self) -> Option<&PhoneNumber> {
        self.phone.as_ref()
    }
}

/// Builds a [`Member`]; created by [`Member::builder`] or
/// [`Member::to_builder`].
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct MemberBuilder {
    id: Option<MemberId>,
    first_name: String,
    accounts: Vec<Account>,
    email: Option<Email>,
    last_name: Option<String>,
    phone: Option<PhoneNumber>,
}

impl MemberBuilder {
    /// Replaces the first name.
    pub fn first_name(mut self, first_name: impl Into<String>) -> Self {
        self.first_name = first_name.into();
        self
    }

    /// Appends an account.
    pub fn account(mut self, account: Account) -> Self {
        self.accounts.push(account);
        self
    }

    /// Replaces every account.
    pub fn accounts(mut self, accounts: impl IntoIterator<Item = Account>) -> Self {
        self.accounts = accounts.into_iter().collect();
        self
    }

    /// Sets the email address, or clears it with `None`.
    pub fn email(mut self, email: impl Into<Option<Email>>) -> Self {
        self.email = email.into();
        self
    }

    /// Sets the last name; blank text clears it.
    pub fn last_name(mut self, last_name: impl Into<String>) -> Self {
        self.last_name = Some(last_name.into());
        self
    }

    /// Sets the phone number, or clears it with `None`.
    pub fn phone(mut self, phone: impl Into<Option<PhoneNumber>>) -> Self {
        self.phone = phone.into();
        self
    }

    /// Validates the names and returns the member, under a newly generated
    /// identifier unless the builder came from
    /// [`Member::to_builder`].
    ///
    /// # Errors
    ///
    /// [`MemberError::EmptyFirstName`] when the first name is blank.
    pub fn build(self) -> Result<Member, MemberError> {
        let first_name = text::required(self.first_name, MemberError::EmptyFirstName)?;
        Ok(Member {
            id: self.id.unwrap_or_else(MemberId::generate),
            first_name,
            accounts: self.accounts,
            email: self.email,
            last_name: self.last_name.and_then(text::optional),
            phone: self.phone,
        })
    }
}

/// The serialised form of a [`Member`], validated on the way in.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct MemberRecord {
    id: MemberId,
    first_name: String,
    #[serde(default)]
    accounts: Vec<Account>,
    #[serde(default)]
    email: Option<Email>,
    #[serde(default)]
    last_name: Option<String>,
    #[serde(default)]
    phone: Option<PhoneNumber>,
}

#[cfg(feature = "serde")]
impl TryFrom<MemberRecord> for Member {
    type Error = MemberError;

    fn try_from(record: MemberRecord) -> Result<Self, Self::Error> {
        let builder = MemberBuilder {
            id: Some(record.id),
            ..Self::builder(record.first_name)
        }
        .email(record.email)
        .phone(record.phone)
        .accounts(record.accounts);
        match record.last_name {
            Some(last_name) => builder.last_name(last_name).build(),
            None => builder.build(),
        }
    }
}

#[cfg(test)]
mod tests;
