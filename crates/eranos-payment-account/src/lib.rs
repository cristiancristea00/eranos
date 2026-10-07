//! Where a member can be paid: bank accounts and payment-app usernames.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use. The facade does not re-export the conversion traits
//! [`FromPaymentAccountDetails`], [`FromPaymentAccountError`], and [`FromIbanError`],
//! through which a crate that tells every variant apart converts account
//! details or a failure without the wildcard arm a `match` outside this crate
//! would need.
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

mod error;
mod iban;

use std::fmt;
use std::str::FromStr;

use eranos_core::text;

pub use error::{FromIbanError, FromPaymentAccountError, IbanError, PaymentAccountError};
pub use iban::Iban;

/// A named place where a member can receive money, such as `ING` with an
/// IBAN or `Revolut` with a username.
///
/// The name is a label; the details say where the money goes.
///
/// # Examples
///
/// ```
/// use eranos::{PaymentAccount, PaymentAccountKind, Iban, Username};
///
/// let bank = PaymentAccount::new("ING", Iban::parse("RO49AAAA1B31007593840000")?)?;
/// assert_eq!(bank.kind(), PaymentAccountKind::Iban);
///
/// let app = PaymentAccount::new("Revolut", Username::new("@ana")?)?;
/// assert_eq!(app.name(), "Revolut");
/// # Ok::<(), eranos::error::PaymentAccountError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PaymentAccount {
    name: String,
    details: PaymentAccountDetails,
}

impl PaymentAccount {
    /// Returns an account named `name`, trimmed.
    ///
    /// # Errors
    ///
    /// [`PaymentAccountError::EmptyName`] when the name is blank.
    pub fn new(
        name: impl Into<String>,
        details: impl Into<PaymentAccountDetails>,
    ) -> Result<Self, PaymentAccountError> {
        let name = text::required(name, PaymentAccountError::EmptyName)?;
        Ok(Self {
            name,
            details: details.into(),
        })
    }

    /// The account's label, such as `"ING"` or `"Revolut"`.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The account details.
    #[must_use]
    pub const fn details(&self) -> &PaymentAccountDetails {
        &self.details
    }

    /// Whether the details are an IBAN or a username.
    #[must_use]
    pub const fn kind(&self) -> PaymentAccountKind {
        self.details.kind()
    }
}

/// The details of an [`PaymentAccount`]: where the money goes.
///
/// Serialised as `{"iban": "…"}` or `{"username": "…"}`, in clear text.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum PaymentAccountDetails {
    /// A bank account.
    Iban(Iban),
    /// A username in a payment app.
    Username(Username),
}

impl PaymentAccountDetails {
    /// Which kind of details these are.
    #[must_use]
    pub const fn kind(&self) -> PaymentAccountKind {
        match self {
            Self::Iban(_) => PaymentAccountKind::Iban,
            Self::Username(_) => PaymentAccountKind::Username,
        }
    }
}

impl From<Iban> for PaymentAccountDetails {
    fn from(iban: Iban) -> Self {
        Self::Iban(iban)
    }
}

impl From<Username> for PaymentAccountDetails {
    fn from(username: Username) -> Self {
        Self::Username(username)
    }
}

/// Builds a value from [`PaymentAccountDetails`], one method per kind of account.
pub trait FromPaymentAccountDetails: Sized {
    /// The value for a bank account's `iban`.
    fn iban(iban: &Iban) -> Self;

    /// The value for a payment app's `username`.
    fn username(username: &Username) -> Self;

    /// The value for `details`, from the method of its variant.
    #[must_use]
    fn from_payment_account_details(details: &PaymentAccountDetails) -> Self {
        match details {
            PaymentAccountDetails::Iban(iban) => Self::iban(iban),
            PaymentAccountDetails::Username(username) => Self::username(username),
        }
    }
}

/// The kind of an [`PaymentAccount`], without its details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum PaymentAccountKind {
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
    /// [`PaymentAccountError::EmptyUsername`] when the text is blank.
    pub fn new(text: &str) -> Result<Self, PaymentAccountError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(PaymentAccountError::EmptyUsername);
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
    type Err = PaymentAccountError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::new(text)
    }
}

/// The serialised form of an [`PaymentAccount`], validated on the way in.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct PaymentAccountRecord {
    name: String,
    details: PaymentAccountDetails,
}

// Written out rather than derived with `serde(try_from)`: its `TryFrom` impl
// from the private record would appear on the facade's page for `PaymentAccount`.
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for PaymentAccount {
    fn deserialize<D: serde::Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        let record = PaymentAccountRecord::deserialize(deserialiser)?;
        Self::new(record.name, record.details).map_err(serde::de::Error::custom)
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
