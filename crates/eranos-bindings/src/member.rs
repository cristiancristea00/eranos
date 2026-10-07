//! Members, their accounts, and the group's categories, as the bindings
//! carry them.

use eranos::category as categories;
use eranos::member as library;
use eranos_payment_account::FromPaymentAccountDetails;

use crate::{EranosError, identifier};

/// Where a member can be paid, without its label.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PaymentAccountDetails {
    /// A bank account.
    Iban {
        /// The IBAN, without spaces, in capitals.
        iban: String,
    },
    /// A username in a payment app.
    Username {
        /// The username, without surrounding whitespace.
        username: String,
    },
}

impl PaymentAccountDetails {
    /// The library's details.
    ///
    /// # Errors
    ///
    /// [`PaymentAccountFailure::InvalidIban`](crate::PaymentAccountFailure::InvalidIban) for
    /// an IBAN the library refuses, and
    /// [`PaymentAccountFailure::EmptyUsername`](crate::PaymentAccountFailure::EmptyUsername)
    /// for a blank username.
    pub(crate) fn to_library(&self) -> Result<library::PaymentAccountDetails, EranosError> {
        Ok(match self {
            Self::Iban { iban } => library::Iban::parse(iban)?.into(),
            Self::Username { username } => library::Username::new(username)?.into(),
        })
    }
}

impl FromPaymentAccountDetails for PaymentAccountDetails {
    fn iban(iban: &library::Iban) -> Self {
        Self::Iban {
            iban: iban.as_str().to_owned(),
        }
    }

    fn username(username: &library::Username) -> Self {
        Self::Username {
            username: username.as_str().to_owned(),
        }
    }
}

/// A place to send a member money: a bank account or a payment-app
/// username, with a label.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaymentAccount {
    /// The label: "Main account".
    pub name: String,
    /// The IBAN or the username.
    pub details: PaymentAccountDetails,
}

impl PaymentAccount {
    /// The record of `account`.
    fn from_library(account: &library::PaymentAccount) -> Self {
        Self {
            name: account.name().to_owned(),
            details: PaymentAccountDetails::from_payment_account_details(account.details()),
        }
    }

    /// The library's account.
    ///
    /// # Errors
    ///
    /// The errors of [`PaymentAccountDetails::to_library`], and
    /// [`PaymentAccountFailure::EmptyName`](crate::PaymentAccountFailure::EmptyName) for a
    /// blank label.
    fn to_library(&self) -> Result<library::PaymentAccount, EranosError> {
        Ok(library::PaymentAccount::new(
            self.name.clone(),
            self.details.to_library()?,
        )?)
    }
}

/// A person in the group.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Member {
    /// The identifier, which the app gives: its own, or one
    /// [`identifier_generate`](crate::identifier_generate) returns.
    pub id: String,
    /// The first name.
    pub first_name: String,
    /// The last name.
    pub last_name: Option<String>,
    /// The email address, without surrounding whitespace.
    pub email: Option<String>,
    /// The phone number, starting with `+` and the country code; a member
    /// the group holds has it in E.164 form: "+40712345678".
    pub phone: Option<String>,
    /// Where the member can be paid, in the order they were given.
    pub accounts: Vec<PaymentAccount>,
}

impl Member {
    /// The record of `member`.
    pub(crate) fn from_library(member: &library::Member) -> Self {
        Self {
            id: member.id().to_string(),
            first_name: member.first_name().to_owned(),
            last_name: member.last_name().map(str::to_owned),
            email: member.email().map(|email| email.as_str().to_owned()),
            phone: member.phone().map(|phone| phone.as_str().to_owned()),
            accounts: member
                .accounts()
                .iter()
                .map(PaymentAccount::from_library)
                .collect(),
        }
    }

    /// The library's member, for a record to add, under the record's
    /// identifier.
    ///
    /// # Errors
    ///
    /// An [`EranosError::Id`] for an identifier the rules refuse, the errors
    /// of the fields' conversions, and
    /// [`MemberFailure::EmptyFirstName`](crate::MemberFailure::EmptyFirstName)
    /// for a blank first name.
    pub(crate) fn to_new(&self) -> Result<library::Member, EranosError> {
        self.filled(library::Member::builder_with_id(
            identifier(&self.id)?,
            self.first_name.clone(),
        ))
    }

    /// The library's member that replaces `existing`, keeping their
    /// identifier, with every field of the record.
    ///
    /// # Errors
    ///
    /// As for [`to_new`](Self::to_new), but for the identifier.
    pub(crate) fn replacing(
        &self,
        existing: &library::Member,
    ) -> Result<library::Member, EranosError> {
        self.filled(existing.to_builder().first_name(self.first_name.clone()))
    }

    /// The member `builder` builds with the record's last name, email,
    /// phone, and accounts, each cleared when the record has none.
    fn filled(&self, builder: library::MemberBuilder) -> Result<library::Member, EranosError> {
        let email = self
            .email
            .as_deref()
            .map(library::Email::parse)
            .transpose()?;
        let phone = self
            .phone
            .as_deref()
            .map(library::PhoneNumber::parse)
            .transpose()?;
        let accounts = self
            .accounts
            .iter()
            .map(PaymentAccount::to_library)
            .collect::<Result<Vec<_>, EranosError>>()?;
        Ok(builder
            .last_name(self.last_name.clone().unwrap_or_default())
            .email(email)
            .phone(phone)
            .accounts(accounts)
            .build()?)
    }
}

/// A label the group gives expenses.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Category {
    /// The identifier, which the app gives: its own, or one
    /// [`identifier_generate`](crate::identifier_generate) returns.
    pub id: String,
    /// The name, which no other category of the group shares.
    pub name: String,
}

impl Category {
    /// The record of `category`.
    pub(crate) fn from_library(category: &categories::Category) -> Self {
        Self {
            id: category.id().to_string(),
            name: category.name().to_owned(),
        }
    }

    /// The library's category, for a record to add, under the record's
    /// identifier.
    ///
    /// # Errors
    ///
    /// An [`EranosError::Id`] for an identifier the rules refuse, and
    /// [`CategoryFailure::EmptyName`](crate::CategoryFailure::EmptyName) for a
    /// blank name.
    pub(crate) fn to_new(&self) -> Result<categories::Category, EranosError> {
        Ok(categories::Category::with_id(
            identifier(&self.id)?,
            self.name.clone(),
        )?)
    }

    /// `existing` renamed to the record's name, keeping its identifier.
    ///
    /// # Errors
    ///
    /// [`CategoryFailure::EmptyName`](crate::CategoryFailure::EmptyName) for a
    /// blank name.
    pub(crate) fn replacing(
        &self,
        existing: &categories::Category,
    ) -> Result<categories::Category, EranosError> {
        Ok(existing.renamed(self.name.clone())?)
    }
}

#[cfg(test)]
mod tests;
