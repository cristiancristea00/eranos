//! The binding core's error, mirrored for `uniffi` as its one error type,
//! with the reason of a refused IBAN that one of its cases holds.

use eranos_bindings::{EranosError, IbanReason, Money};

#[uniffi::remote(Error)]
pub enum EranosError {
    AccountEmptyName {
        detail: String,
    },
    AmountsDoNotAddUp {
        actual: Money,
        detail: String,
        expected: Money,
    },
    ArchivedMember {
        detail: String,
        member: String,
    },
    BuiltInCategory {
        detail: String,
    },
    Busy {
        detail: String,
    },
    Cancelled {
        detail: String,
    },
    CategoryEmptyName {
        detail: String,
    },
    DuplicateCategory {
        category: String,
        detail: String,
    },
    DuplicateCategoryName {
        detail: String,
        existing: String,
    },
    DuplicateExchangeRate {
        currency: String,
        detail: String,
    },
    DuplicateExpense {
        detail: String,
        expense: String,
    },
    DuplicateTransfer {
        detail: String,
        transfer: String,
    },
    EmptyFirstName {
        detail: String,
    },
    EmptyItemName {
        detail: String,
    },
    EmptySplit {
        detail: String,
    },
    EmptyUsername {
        detail: String,
    },
    ExpenseCurrencyMismatch {
        detail: String,
        expected: String,
        found: String,
    },
    ExpenseDuplicateMember {
        detail: String,
        member: String,
    },
    ExpenseEmptyName {
        detail: String,
    },
    ExpenseNonPositiveAmount {
        detail: String,
    },
    ExpenseOverflow {
        detail: String,
    },
    GroupDuplicateMember {
        detail: String,
        member: String,
    },
    GroupEmptyName {
        detail: String,
    },
    GroupOverflow {
        detail: String,
    },
    IdentifierGiven {
        detail: String,
    },
    IdentifierMissing {
        detail: String,
    },
    InvalidAmount {
        detail: String,
    },
    InvalidDate {
        detail: String,
    },
    InvalidEmail {
        detail: String,
    },
    InvalidExchangeRate {
        detail: String,
    },
    InvalidIban {
        detail: String,
        reason: IbanReason,
    },
    InvalidId {
        detail: String,
    },
    InvalidJson {
        column: i64,
        detail: String,
        line: i64,
    },
    InvalidLanguage {
        detail: String,
    },
    InvalidParts {
        detail: String,
    },
    InvalidPercentage {
        detail: String,
    },
    InvalidPhoneNumber {
        detail: String,
    },
    InvalidTime {
        detail: String,
    },
    ItemsDoNotAddUp {
        actual: Money,
        detail: String,
        expected: Money,
    },
    MemberInUse {
        detail: String,
        expenses: Vec<String>,
        member: String,
        transfers: Vec<String>,
    },
    MissingExchangeRate {
        currency: String,
        detail: String,
    },
    MoneyCurrencyMismatch {
        detail: String,
        left: String,
        right: String,
    },
    MoneyOverflow {
        detail: String,
    },
    NegativeTip {
        detail: String,
    },
    NoItems {
        detail: String,
    },
    NonPositiveShare {
        detail: String,
        member: String,
    },
    OutOfMemory {
        balances: i64,
        detail: String,
    },
    PercentagesDoNotAddUp {
        detail: String,
        total_basis_points: i64,
    },
    Released {
        detail: String,
    },
    SameMember {
        detail: String,
    },
    TooManyDecimals {
        allowed: i64,
        currency: String,
        detail: String,
    },
    TransferNonPositiveAmount {
        detail: String,
    },
    UnexpectedExchangeRate {
        detail: String,
    },
    UnknownCategory {
        category: String,
        detail: String,
    },
    UnknownCurrency {
        detail: String,
    },
    UnknownExpense {
        detail: String,
        expense: String,
    },
    UnknownMember {
        detail: String,
        member: String,
    },
    UnknownTransfer {
        detail: String,
        transfer: String,
    },
    UnsupportedFormat {
        detail: String,
        format: i64,
    },
    ZeroQuantity {
        detail: String,
    },
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
