//! Members, their contact details and accounts, and the group's
//! categories, as the bindings carry them.

use eranos::category as categories;
use eranos::member as library;
use eranos_payment_account::FromPaymentAccountDetails;

use crate::{EranosError, MismatchedField, identifier};

/// A phone number, in the two forms the library writes it in.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhoneNumber {
    /// The number in E.164 form: "+40712345678".
    pub e164: String,
    /// The number in international form, grouped as its country groups it:
    /// "+40 712 345 678".
    pub international: String,
}

impl PhoneNumber {
    /// The record of `phone`, with the international form the library
    /// writes for it.
    pub(crate) fn from_library(phone: &library::PhoneNumber) -> Self {
        Self {
            e164: phone.as_str().to_owned(),
            international: phone.international(),
        }
    }

    /// The library's phone number of the record's E.164 text, which it
    /// parses as it parses any phone number.
    ///
    /// # Errors
    ///
    /// [`ContactFailure::InvalidPhoneNumber`](crate::ContactFailure::InvalidPhoneNumber)
    /// for text that is not a phone number, and
    /// [`EranosError::MismatchedRecord`] for an international form other
    /// than the one the library writes for the number: such a record names
    /// no number as the library writes it.
    pub(crate) fn to_library(&self) -> Result<library::PhoneNumber, EranosError> {
        let phone = library::PhoneNumber::parse(&self.e164)?;
        if phone.international() == self.international {
            Ok(phone)
        } else {
            Err(EranosError::mismatched_record(
                MismatchedField::PhoneInternational,
            ))
        }
    }
}

/// An IBAN, with the country code it starts with.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Iban {
    /// The IBAN, without spaces, in capitals: "RO49AAAA1B31007593840000".
    pub iban: String,
    /// The two-letter code of the IBAN's country: "RO".
    pub country_code: String,
}

impl Iban {
    /// The record of `iban`, with its country code.
    pub(crate) fn from_library(iban: &library::Iban) -> Self {
        Self {
            iban: iban.as_str().to_owned(),
            country_code: iban.country_code().to_owned(),
        }
    }
}

/// Where a member can be paid, without its label.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PaymentAccountDetails {
    /// A bank account.
    Iban {
        /// The IBAN, without spaces, in capitals: "RO49AAAA1B31007593840000".
        iban: String,
        /// The two-letter code of the IBAN's country: "RO".
        country_code: String,
    },
    /// A username in a payment app.
    Username {
        /// The username, without surrounding whitespace.
        username: String,
    },
}

impl PaymentAccountDetails {
    /// The library's details. An IBAN is parsed as the library parses any
    /// IBAN, so it may hold spaces and lower case.
    ///
    /// # Errors
    ///
    /// [`PaymentAccountFailure::InvalidIban`](crate::PaymentAccountFailure::InvalidIban) for
    /// an IBAN the library refuses, with the reason;
    /// [`EranosError::MismatchedRecord`] for a country code other than the
    /// IBAN's: such details name no account of the IBAN registry; and
    /// [`PaymentAccountFailure::EmptyUsername`](crate::PaymentAccountFailure::EmptyUsername)
    /// for a blank username.
    pub(crate) fn to_library(&self) -> Result<library::PaymentAccountDetails, EranosError> {
        Ok(match self {
            Self::Iban { iban, country_code } => {
                let iban = library::Iban::parse(iban)?;
                if iban.country_code() != country_code {
                    return Err(EranosError::mismatched_record(
                        MismatchedField::IbanCountryCode,
                    ));
                }
                iban.into()
            }
            Self::Username { username } => library::Username::new(username)?.into(),
        })
    }
}

impl From<Iban> for PaymentAccountDetails {
    fn from(Iban { iban, country_code }: Iban) -> Self {
        Self::Iban { iban, country_code }
    }
}

impl FromPaymentAccountDetails for PaymentAccountDetails {
    fn iban(iban: &library::Iban) -> Self {
        Iban::from_library(iban).into()
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
    /// The phone number, in E.164 form and in international form.
    pub phone: Option<PhoneNumber>,
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
            phone: member.phone().map(PhoneNumber::from_library),
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
            .as_ref()
            .map(PhoneNumber::to_library)
            .transpose()?;
        let accounts = self
            .accounts
            .iter()
            .map(PaymentAccount::to_library)
            .collect::<Result<Vec<_>, EranosError>>()?;
        let builder = match &self.last_name {
            Some(last_name) => builder.last_name(last_name.as_str()),
            None => builder.clear_last_name(),
        };
        Ok(builder
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
