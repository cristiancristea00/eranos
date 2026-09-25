//! The binding core's error, mirrored for `uniffi` as its one error type:
//! a case per module of the library, each holding that module's failure,
//! and the reason of a refused IBAN that an account's failure holds.

use eranos_bindings::{
    AccountFailure, CalendarFailure, CategoryFailure, ContactFailure, EranosError, ExpenseFailure,
    GroupFailure, IbanReason, IdFailure, LanguageFailure, MemberFailure, Money, MoneyFailure,
    SettlementFailure, TransferFailure,
};

#[uniffi::remote(Error)]
pub enum EranosError {
    Account {
        detail: String,
        failure: AccountFailure,
    },
    Calendar {
        detail: String,
        failure: CalendarFailure,
    },
    Cancelled {
        detail: String,
    },
    Category {
        detail: String,
        failure: CategoryFailure,
    },
    Contact {
        detail: String,
        failure: ContactFailure,
    },
    Expense {
        detail: String,
        failure: ExpenseFailure,
    },
    Group {
        detail: String,
        failure: GroupFailure,
    },
    Id {
        detail: String,
        failure: IdFailure,
    },
    IdentifierGiven {
        detail: String,
    },
    IdentifierMissing {
        detail: String,
    },
    InvalidJson {
        column: i64,
        detail: String,
        line: i64,
    },
    Language {
        detail: String,
        failure: LanguageFailure,
    },
    Member {
        detail: String,
        failure: MemberFailure,
    },
    Money {
        detail: String,
        failure: MoneyFailure,
    },
    Released {
        detail: String,
    },
    Settlement {
        detail: String,
        failure: SettlementFailure,
    },
    Transfer {
        detail: String,
        failure: TransferFailure,
    },
}

#[uniffi::remote(Enum)]
pub enum AccountFailure {
    EmptyName,
    EmptyUsername,
    InvalidIban { reason: IbanReason },
}

#[uniffi::remote(Enum)]
pub enum CalendarFailure {
    InvalidDate,
    InvalidTime,
}

#[uniffi::remote(Enum)]
pub enum CategoryFailure {
    EmptyName,
}

#[uniffi::remote(Enum)]
pub enum ContactFailure {
    InvalidEmail,
    InvalidPhoneNumber,
}

#[uniffi::remote(Enum)]
pub enum ExpenseFailure {
    AmountsDoNotAddUp { actual: Money, expected: Money },
    CurrencyMismatch { expected: String, found: String },
    DuplicateMember { member: String },
    EmptyItemName,
    EmptyName,
    EmptySplit,
    ItemsDoNotAddUp { actual: Money, expected: Money },
    NegativeTip,
    NoItems,
    NonPositiveAmount,
    NonPositiveShare { member: String },
    Overflow,
    PercentagesDoNotAddUp { total_basis_points: i64 },
    ZeroQuantity,
}

#[uniffi::remote(Enum)]
pub enum GroupFailure {
    ArchivedMember {
        member: String,
    },
    BuiltInCategory,
    Busy,
    DuplicateCategory {
        category: String,
    },
    DuplicateCategoryName {
        existing: String,
    },
    DuplicateExchangeRate {
        currency: String,
    },
    DuplicateExpense {
        expense: String,
    },
    DuplicateMember {
        member: String,
    },
    DuplicateTransfer {
        transfer: String,
    },
    EmptyName,
    MemberInUse {
        expenses: Vec<String>,
        member: String,
        transfers: Vec<String>,
    },
    MissingExchangeRate {
        currency: String,
    },
    Overflow,
    UnexpectedExchangeRate,
    UnknownCategory {
        category: String,
    },
    UnknownExpense {
        expense: String,
    },
    UnknownMember {
        member: String,
    },
    UnknownTransfer {
        transfer: String,
    },
    UnsupportedFormat {
        format: i64,
    },
}

#[uniffi::remote(Enum)]
pub enum IdFailure {
    InvalidId,
}

#[uniffi::remote(Enum)]
pub enum LanguageFailure {
    InvalidLanguage,
}

#[uniffi::remote(Enum)]
pub enum MemberFailure {
    EmptyFirstName,
}

#[uniffi::remote(Enum)]
pub enum MoneyFailure {
    CurrencyMismatch { left: String, right: String },
    InvalidAmount,
    InvalidExchangeRate,
    InvalidParts,
    InvalidPercentage,
    Overflow,
    TooManyDecimals { allowed: i64, currency: String },
    UnknownCurrency,
}

#[uniffi::remote(Enum)]
pub enum SettlementFailure {
    OutOfMemory { balances: i64 },
}

#[uniffi::remote(Enum)]
pub enum TransferFailure {
    NonPositiveAmount,
    SameMember,
}

#[uniffi::remote(Enum)]
pub enum IbanReason {
    InvalidCharacter,
    InvalidChecksum,
    InvalidFormat,
    InvalidStructure,
    UnknownCountry,
    WrongLength { expected: i64, found: i64 },
}
