//! Where a member can be paid: bank accounts and payment-app usernames.

use std::fmt;
use std::str::FromStr;

use crate::error::{Field, ValueError};
use crate::member::Iban;
use crate::secret::Secret;
use crate::text;

/// A named place where a member can receive money, such as `ING` with an
/// IBAN or `Revolut` with a username.
///
/// The name is a label and is not secret; the account details are.
///
/// # Examples
///
/// ```
/// use eranos::{Account, AccountKind, Iban, Username};
///
/// let bank = Account::new("ING", Iban::parse("RO49AAAA1B31007593840000")?)?;
/// assert_eq!(bank.kind(), AccountKind::Iban);
///
/// let app = Account::new("Revolut", Username::new("@ana")?)?;
/// assert_eq!(app.name(), "Revolut");
/// # Ok::<(), eranos::error::ValueError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "AccountRecord")
)]
pub struct Account {
    details: AccountDetails,
    name: String,
}

impl Account {
    /// Returns an account named `name`, trimmed.
    ///
    /// # Errors
    ///
    /// [`ValueError::Empty`] when the name is blank.
    pub fn new(
        name: impl Into<String>,
        details: impl Into<AccountDetails>,
    ) -> Result<Self, ValueError> {
        let name = text::required(name, Field::AccountName)?;
        Ok(Self {
            details: details.into(),
            name,
        })
    }

    /// The secret account details.
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

/// The secret details of an [`Account`].
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

/// The kind of an [`Account`], without its secret details.
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

/// A username in a payment app, held as a secret.
///
/// The text is wiped from memory when dropped and redacted in `Debug`; it is
/// available through [`expose_secret`](Self::expose_secret).
#[derive(Clone, PartialEq, Eq)]
pub struct Username(Secret);

impl Username {
    /// Returns the username `text`, trimmed.
    ///
    /// # Errors
    ///
    /// [`ValueError::Empty`] when the text is blank.
    pub fn new(text: &str) -> Result<Self, ValueError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(ValueError::Empty {
                field: Field::Username,
            });
        }
        Ok(Self(Secret::new(trimmed.to_owned())))
    }

    /// The username.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        self.0.expose()
    }
}

impl fmt::Debug for Username {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Username").field(&self.0).finish()
    }
}

impl FromStr for Username {
    type Err = ValueError;

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

#[cfg(feature = "serde")]
impl TryFrom<AccountRecord> for Account {
    type Error = ValueError;

    fn try_from(record: AccountRecord) -> Result<Self, Self::Error> {
        Self::new(record.name, record.details)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Username {
    fn serialize<S: serde::Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
        serialiser.serialize_str(self.expose_secret())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Username {
    fn deserialize<D: serde::Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        crate::serde_text::deserialise(deserialiser, "a non-empty username")
    }
}

#[cfg(test)]
mod tests;
