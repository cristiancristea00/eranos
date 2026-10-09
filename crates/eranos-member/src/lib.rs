//! The people in a group, with their contact details and the accounts they
//! can be paid to.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use. The facade does not re-export [`FromMemberError`],
//! through which a crate that tells every failure apart converts one without
//! the wildcard arm a `match` outside this crate would need.
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

mod error;

use eranos_contact::{Email, PhoneNumber};
use eranos_core::{MemberId, text};
use eranos_payment_account::PaymentAccount;
use std::fmt;

pub use error::{FromMemberError, MemberError};

/// A person who pays for or shares expenses.
///
/// Only the first name is required. A member has an identifier from the
/// moment its builder starts: the one given to
/// [`builder_with_id`](Self::builder_with_id), or one that
/// [`builder`](Self::builder) generates, and keeps it through every edit.
/// Members are values: to change one, edit a copy with
/// [`to_builder`](Self::to_builder) and pass it to
/// `ExpenseGroup::replace_member`.
///
/// # Examples
///
/// ```
/// use eranos::{PaymentAccount, Email, Iban, Member};
///
/// let ana = Member::builder("Ana")
///     .last_name("Popescu")
///     .email(Email::parse("ana@example.com")?)
///     .account(PaymentAccount::new("ING", Iban::parse("RO49AAAA1B31007593840000")?)?)
///     .build()?;
/// assert_eq!(ana.first_name(), "Ana");
/// assert_eq!(ana.accounts().len(), 1);
///
/// let renamed = ana.to_builder().first_name("Anna").build()?;
/// assert_eq!(renamed.last_name(), Some("Popescu"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Member {
    id: MemberId,
    first_name: String,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    last_name: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    email: Option<Email>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    phone: Option<PhoneNumber>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    accounts: Vec<PaymentAccount>,
}

/// Prints the identifier and not the names, the contact details, or the
/// accounts, which a log must not hold.
impl fmt::Debug for Member {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Member")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl Member {
    /// Starts a member with the required first name, under an identifier
    /// generated when it is called; [`builder_with_id`](Self::builder_with_id)
    /// takes one instead.
    pub fn builder(first_name: impl Into<String>) -> MemberBuilder {
        Self::builder_with_id(MemberId::generate(), first_name)
    }

    /// Starts a member with the required first name under the identifier
    /// `id`, such as the key an app's own store gives the member;
    /// [`builder`](Self::builder) generates one instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{Member, MemberId};
    ///
    /// let ana = Member::builder_with_id(MemberId::new("ana")?, "Ana").build()?;
    /// assert_eq!(ana.id().as_str(), "ana");
    /// assert_eq!(ana.to_builder().first_name("Anna").build()?.id(), ana.id());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn builder_with_id(id: MemberId, first_name: impl Into<String>) -> MemberBuilder {
        MemberBuilder {
            id,
            first_name: first_name.into(),
            last_name: None,
            email: None,
            phone: None,
            accounts: Vec::new(),
        }
    }

    /// Starts a builder holding a copy of this member, to edit it; the copy
    /// keeps the identifier.
    pub fn to_builder(&self) -> MemberBuilder {
        MemberBuilder {
            id: self.id,
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            email: self.email.clone(),
            phone: self.phone.clone(),
            accounts: self.accounts.clone(),
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

    /// The last name, if any.
    #[must_use]
    pub fn last_name(&self) -> Option<&str> {
        self.last_name.as_deref()
    }

    /// The email address, if any.
    #[must_use]
    pub const fn email(&self) -> Option<&Email> {
        self.email.as_ref()
    }

    /// The phone number, if any.
    #[must_use]
    pub const fn phone(&self) -> Option<&PhoneNumber> {
        self.phone.as_ref()
    }

    /// The accounts the member can be paid to, in the order they were added.
    #[must_use]
    pub fn accounts(&self) -> &[PaymentAccount] {
        &self.accounts
    }
}

/// Builds a [`Member`]; created by [`Member::builder`],
/// [`Member::builder_with_id`], or [`Member::to_builder`].
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct MemberBuilder {
    id: MemberId,
    first_name: String,
    last_name: Option<String>,
    email: Option<Email>,
    phone: Option<PhoneNumber>,
    accounts: Vec<PaymentAccount>,
}

impl MemberBuilder {
    /// Replaces the identifier.
    pub const fn id(mut self, id: MemberId) -> Self {
        self.id = id;
        self
    }

    /// Replaces the first name.
    pub fn first_name(mut self, first_name: impl Into<String>) -> Self {
        self.first_name = first_name.into();
        self
    }

    /// Sets the last name, trimmed; blank text leaves it absent, as
    /// [`clear_last_name`](Self::clear_last_name) does.
    pub fn last_name(mut self, last_name: impl Into<String>) -> Self {
        self.last_name = Some(last_name.into());
        self
    }

    /// Clears the last name, which [`last_name`](Self::last_name) sets.
    pub fn clear_last_name(mut self) -> Self {
        self.last_name = None;
        self
    }

    /// Sets the email address, or clears it with `None`.
    pub fn email(mut self, email: impl Into<Option<Email>>) -> Self {
        self.email = email.into();
        self
    }

    /// Sets the phone number, or clears it with `None`.
    pub fn phone(mut self, phone: impl Into<Option<PhoneNumber>>) -> Self {
        self.phone = phone.into();
        self
    }

    /// Replaces every account.
    pub fn accounts(mut self, accounts: impl IntoIterator<Item = PaymentAccount>) -> Self {
        self.accounts = accounts.into_iter().collect();
        self
    }

    /// Appends an account.
    pub fn account(mut self, account: PaymentAccount) -> Self {
        self.accounts.push(account);
        self
    }

    /// Validates the names and returns the member, under the identifier the
    /// builder holds.
    ///
    /// # Errors
    ///
    /// [`MemberError::EmptyFirstName`] when the first name is blank.
    pub fn build(self) -> Result<Member, MemberError> {
        let first_name = text::required(self.first_name, MemberError::EmptyFirstName)?;
        Ok(Member {
            id: self.id,
            first_name,
            last_name: self.last_name.and_then(text::optional),
            email: self.email,
            phone: self.phone,
            accounts: self.accounts,
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
    last_name: Option<String>,
    #[serde(default)]
    email: Option<Email>,
    #[serde(default)]
    phone: Option<PhoneNumber>,
    #[serde(default)]
    accounts: Vec<PaymentAccount>,
}

/// Builds the member `record` describes, through the checks of the builder.
#[cfg(feature = "serde")]
fn from_record(record: MemberRecord) -> Result<Member, MemberError> {
    let builder = Member::builder_with_id(record.id, record.first_name)
        .email(record.email)
        .phone(record.phone)
        .accounts(record.accounts);
    match record.last_name {
        Some(last_name) => builder.last_name(last_name).build(),
        None => builder.build(),
    }
}

// Written out rather than derived with `serde(try_from)`: its `TryFrom` impl
// from the private record would appear on the facade's page for `Member`.
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Member {
    fn deserialize<D: serde::Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        from_record(MemberRecord::deserialize(deserialiser)?).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests;
