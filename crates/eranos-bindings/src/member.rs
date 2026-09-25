//! Members, their accounts, and the group's categories, as the bindings
//! carry them.

use eranos::category as categories;
use eranos::member as library;
use eranos::{CategoryId, MemberId};
use eranos_account::FromAccountDetails;

use crate::{EranosError, identifier};

/// Where a member can be paid, without its label.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccountDetails {
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

impl AccountDetails {
    /// The library's details.
    ///
    /// # Errors
    ///
    /// [`EranosError::InvalidIban`] for an IBAN the library refuses, and
    /// [`EranosError::EmptyUsername`] for a blank username.
    pub(crate) fn to_library(&self) -> Result<library::AccountDetails, EranosError> {
        Ok(match self {
            Self::Iban { iban } => library::Iban::parse(iban)?.into(),
            Self::Username { username } => library::Username::new(username)?.into(),
        })
    }
}

impl FromAccountDetails for AccountDetails {
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Account {
    /// The IBAN or the username.
    pub details: AccountDetails,
    /// The label: "Main account".
    pub name: String,
}

impl Account {
    /// The record of `account`.
    fn from_library(account: &library::Account) -> Self {
        Self {
            details: AccountDetails::from_account_details(account.details()),
            name: account.name().to_owned(),
        }
    }

    /// The library's account.
    ///
    /// # Errors
    ///
    /// The errors of [`AccountDetails::to_library`], and
    /// [`EranosError::AccountEmptyName`] for a blank label.
    fn to_library(&self) -> Result<library::Account, EranosError> {
        Ok(library::Account::new(
            self.details.to_library()?,
            self.name.clone(),
        )?)
    }
}

/// A person in the group.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Member {
    /// The identifier: absent for a member to add, present for one the
    /// group holds or one to replace.
    pub id: Option<String>,
    /// The first name.
    pub first_name: String,
    /// Where the member can be paid, in the order they were given.
    pub accounts: Vec<Account>,
    /// The email address, without surrounding whitespace.
    pub email: Option<String>,
    /// The last name.
    pub last_name: Option<String>,
    /// The phone number, starting with `+` and the country code; a member
    /// the group holds has it in E.164 form: "+40712345678".
    pub phone: Option<String>,
}

impl Member {
    /// The record of `member`.
    pub(crate) fn from_library(member: &library::Member) -> Self {
        Self {
            id: Some(member.id().to_string()),
            first_name: member.first_name().to_owned(),
            accounts: member
                .accounts()
                .iter()
                .map(Account::from_library)
                .collect(),
            email: member.email().map(|email| email.as_str().to_owned()),
            last_name: member.last_name().map(str::to_owned),
            phone: member.phone().map(|phone| phone.as_str().to_owned()),
        }
    }

    /// The library's member, for a record to add: they get a new
    /// identifier.
    ///
    /// # Errors
    ///
    /// [`EranosError::IdentifierGiven`] when the record has an identifier,
    /// the errors of the fields' conversions, and
    /// [`EranosError::EmptyFirstName`] for a blank first name.
    pub(crate) fn to_new(&self) -> Result<library::Member, EranosError> {
        if self.id.is_some() {
            return Err(EranosError::identifier_given());
        }
        self.filled(library::Member::builder(self.first_name.clone()))
    }

    /// The identifier of a record to replace.
    ///
    /// # Errors
    ///
    /// [`EranosError::IdentifierMissing`] when the record has none, and
    /// [`EranosError::InvalidId`] when it is not an identifier.
    pub(crate) fn id_to_replace(&self) -> Result<MemberId, EranosError> {
        identifier(
            self.id
                .as_deref()
                .ok_or_else(EranosError::identifier_missing)?,
        )
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

    /// The member `builder` builds with the record's accounts, email, last
    /// name, and phone, each cleared when the record has none.
    fn filled(&self, builder: library::MemberBuilder) -> Result<library::Member, EranosError> {
        let accounts = self
            .accounts
            .iter()
            .map(Account::to_library)
            .collect::<Result<Vec<_>, EranosError>>()?;
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
        Ok(builder
            .accounts(accounts)
            .email(email)
            .last_name(self.last_name.clone().unwrap_or_default())
            .phone(phone)
            .build()?)
    }
}

/// A label the group gives expenses.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Category {
    /// The identifier: absent for a category to add, present for one the
    /// group holds or one to rename.
    pub id: Option<String>,
    /// The name, which no other category of the group shares.
    pub name: String,
}

impl Category {
    /// The record of `category`.
    pub(crate) fn from_library(category: &categories::Category) -> Self {
        Self {
            id: Some(category.id().to_string()),
            name: category.name().to_owned(),
        }
    }

    /// The library's category, for a record to add: it gets a new
    /// identifier.
    ///
    /// # Errors
    ///
    /// [`EranosError::IdentifierGiven`] when the record has an identifier,
    /// and [`EranosError::CategoryEmptyName`] for a blank name.
    pub(crate) fn to_new(&self) -> Result<categories::Category, EranosError> {
        if self.id.is_some() {
            return Err(EranosError::identifier_given());
        }
        Ok(categories::Category::new(self.name.clone())?)
    }

    /// The identifier of a record to rename.
    ///
    /// # Errors
    ///
    /// [`EranosError::IdentifierMissing`] when the record has none, and
    /// [`EranosError::InvalidId`] when it is not an identifier.
    pub(crate) fn id_to_replace(&self) -> Result<CategoryId, EranosError> {
        identifier(
            self.id
                .as_deref()
                .ok_or_else(EranosError::identifier_missing)?,
        )
    }

    /// `existing` renamed to the record's name, keeping its identifier.
    ///
    /// # Errors
    ///
    /// [`EranosError::CategoryEmptyName`] for a blank name.
    pub(crate) fn replacing(
        &self,
        existing: &categories::Category,
    ) -> Result<categories::Category, EranosError> {
        Ok(existing.renamed(self.name.clone())?)
    }
}

#[cfg(test)]
mod tests;
