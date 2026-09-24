//! Where a member can be paid: bank accounts and payment-app usernames.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use.
#![doc(test(attr(deny(warnings))))]

mod error;
mod iban;

use std::fmt;
use std::str::FromStr;

use eranos_core::text;

pub use error::{AccountError, IbanError};
pub use iban::Iban;

/// A named place where a member can receive money, such as `ING` with an
/// IBAN or `Revolut` with a username.
///
/// The name is a label; the details say where the money goes.
///
/// # Examples
///
/// ```
/// use eranos::{Account, AccountKind, Iban, Username};
///
/// let bank = Account::new(Iban::parse("RO49AAAA1B31007593840000")?, "ING")?;
/// assert_eq!(bank.kind(), AccountKind::Iban);
///
/// let app = Account::new(Username::new("@ana")?, "Revolut")?;
/// assert_eq!(app.name(), "Revolut");
/// # Ok::<(), eranos::error::AccountError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Account {
    details: AccountDetails,
    name: String,
}

impl Account {
    /// Returns an account named `name`, trimmed.
    ///
    /// # Errors
    ///
    /// [`AccountError::EmptyName`] when the name is blank.
    pub fn new(
        details: impl Into<AccountDetails>,
        name: impl Into<String>,
    ) -> Result<Self, AccountError> {
        let name = text::required(name, AccountError::EmptyName)?;
        Ok(Self {
            details: details.into(),
            name,
        })
    }

    /// The account details.
    #[must_use]
    pub const fn details(&self) -> &AccountDetails {
        &self.details
    }

    /// Whether the details are an IBAN or a username.
    #[must_use]
    pub const fn kind(&self) -> AccountKind {
        self.details.kind()
    }

    /// The account's label, such as `"ING"` or `"Revolut"`.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// The details of an [`Account`]: where the money goes.
///
/// Serialised as `{"iban": "…"}` or `{"username": "…"}`, in clear text.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum AccountDetails {
    /// A bank account.
    Iban(Iban),
    /// A username in a payment app.
    Username(Username),
}

impl AccountDetails {
    /// Which kind of details these are.
    #[must_use]
    pub const fn kind(&self) -> AccountKind {
        match self {
            Self::Iban(_) => AccountKind::Iban,
            Self::Username(_) => AccountKind::Username,
        }
    }
}

impl From<Iban> for AccountDetails {
    fn from(iban: Iban) -> Self {
        Self::Iban(iban)
    }
}

impl From<Username> for AccountDetails {
    fn from(username: Username) -> Self {
        Self::Username(username)
    }
}

/// The kind of an [`Account`], without its details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum AccountKind {
    /// A bank account identified by an IBAN.
    Iban,
    /// A payment-app account identified by a username.
    Username,
}

/// A username in a payment app, such as `@ana`.
///
/// [`as_str`](Self::as_str) returns it and `Display` prints it, trimmed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Username(String);

impl Username {
    /// Returns the username `text`, trimmed.
    ///
    /// # Errors
    ///
    /// [`AccountError::EmptyUsername`] when the text is blank.
    pub fn new(text: &str) -> Result<Self, AccountError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(AccountError::EmptyUsername);
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// The username.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Username {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for Username {
    type Err = AccountError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::new(text)
    }
}

/// The serialised form of an [`Account`], validated on the way in.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct AccountRecord {
    details: AccountDetails,
    name: String,
}

// Written out rather than derived with `serde(try_from)`: its `TryFrom` impl
// from the private record would appear on the facade's page for `Account`.
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Account {
    fn deserialize<D: serde::Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        let record = AccountRecord::deserialize(deserialiser)?;
        Self::new(record.details, record.name).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Username {
    fn serialize<S: serde::Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
        serialiser.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Username {
    fn deserialize<D: serde::Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        eranos_core::serde_text::deserialise(deserialiser, "a non-empty username")
    }
}

#[cfg(test)]
mod tests;
