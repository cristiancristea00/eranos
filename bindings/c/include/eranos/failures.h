/// @file eranos/failures.h
/// What a failure reports: the module that refused a call, the failure within the module, and why an IBAN was refused.

#ifndef ERANOS_FAILURES_H
#define ERANOS_FAILURES_H

#include "EranosErrorKind.h"
#include "EranosFailure.h"
#include "EranosIbanReason.h"

/// Why an IBAN was refused. The values of EranosIbanReason, by the names of the C layer.
typedef enum EranosIbanReason EranosIbanReason;

/// The values of EranosIbanReason.
enum {
    /// A character other than an ASCII letter, a digit, or whitespace (which is dropped).
    ERANOS_IBAN_REASON_INVALID_CHARACTER = EranosIbanReason_InvalidCharacter,
    /// The check digits are 00, 01, or 99, which ISO 13616 never issues, or the ISO 7064 mod-97 check failed: the number contains a typo.
    ERANOS_IBAN_REASON_INVALID_CHECKSUM = EranosIbanReason_InvalidChecksum,
    /// Not two letters followed by two check digits.
    ERANOS_IBAN_REASON_INVALID_FORMAT = EranosIbanReason_InvalidFormat,
    /// A letter where the country's account number has a digit, or a digit where it has a letter, by the BBAN structure of the SWIFT IBAN registry.
    ERANOS_IBAN_REASON_INVALID_STRUCTURE = EranosIbanReason_InvalidStructure,
    /// A country code without an entry in the SWIFT IBAN registry.
    ERANOS_IBAN_REASON_UNKNOWN_COUNTRY = EranosIbanReason_UnknownCountry,
    /// The wrong number of characters for the country.
    ERANOS_IBAN_REASON_WRONG_LENGTH = EranosIbanReason_WrongLength,
};

/// The failure within its module, named after the module and the failure. The values of EranosFailure, by the names of the C layer.
typedef enum EranosFailure EranosFailure;

/// The values of EranosFailure.
enum {
    /// An account whose name is blank.
    ERANOS_FAILURE_ACCOUNT_EMPTY_NAME = EranosFailure_AccountEmptyName,
    /// A username that is blank.
    ERANOS_FAILURE_ACCOUNT_EMPTY_USERNAME = EranosFailure_AccountEmptyUsername,
    /// An IBAN that failed validation, and why.
    ERANOS_FAILURE_ACCOUNT_INVALID_IBAN = EranosFailure_AccountInvalidIban,
    /// A calendar date that does not exist, is outside years 1 to 9999, or is not ISO 8601 text.
    ERANOS_FAILURE_CALENDAR_INVALID_DATE = EranosFailure_CalendarInvalidDate,
    /// A time of day that does not exist, has seconds, or is not ISO 8601 text.
    ERANOS_FAILURE_CALENDAR_INVALID_TIME = EranosFailure_CalendarInvalidTime,
    /// A name that is blank.
    ERANOS_FAILURE_CATEGORY_EMPTY_NAME = EranosFailure_CategoryEmptyName,
    /// An email address that breaks the RFC 5321 and 5322 rules: no local part or domain around an `@`, a malformed one, one too long, or whitespace
    /// or an invisible character.
    ERANOS_FAILURE_CONTACT_INVALID_EMAIL = EranosFailure_ContactInvalidEmail,
    /// A phone number that does not start with `+`, holds a letter or an extension, or is not valid in its country's numbering plan.
    ERANOS_FAILURE_CONTACT_INVALID_PHONE_NUMBER = EranosFailure_ContactInvalidPhoneNumber,
    /// The amounts of a split do not add up to the amount being split.
    ERANOS_FAILURE_EXPENSE_AMOUNTS_DO_NOT_ADD_UP = EranosFailure_ExpenseAmountsDoNotAddUp,
    /// An amount inside the expense is in a different currency.
    ERANOS_FAILURE_EXPENSE_CURRENCY_MISMATCH = EranosFailure_ExpenseCurrencyMismatch,
    /// A member appears more than once in one split.
    ERANOS_FAILURE_EXPENSE_DUPLICATE_MEMBER = EranosFailure_ExpenseDuplicateMember,
    /// An item whose name is blank.
    ERANOS_FAILURE_EXPENSE_EMPTY_ITEM_NAME = EranosFailure_ExpenseEmptyItemName,
    /// An expense whose name is blank.
    ERANOS_FAILURE_EXPENSE_EMPTY_NAME = EranosFailure_ExpenseEmptyName,
    /// A split lists no member.
    ERANOS_FAILURE_EXPENSE_EMPTY_SPLIT = EranosFailure_ExpenseEmptySplit,
    /// The item line totals do not add up to the expense amount.
    ERANOS_FAILURE_EXPENSE_ITEMS_DO_NOT_ADD_UP = EranosFailure_ExpenseItemsDoNotAddUp,
    /// A fixed tip below zero.
    ERANOS_FAILURE_EXPENSE_NEGATIVE_TIP = EranosFailure_ExpenseNegativeTip,
    /// An itemised expense without items.
    ERANOS_FAILURE_EXPENSE_NO_ITEMS = EranosFailure_ExpenseNoItems,
    /// The amount or unit price is zero or negative.
    ERANOS_FAILURE_EXPENSE_NON_POSITIVE_AMOUNT = EranosFailure_ExpenseNonPositiveAmount,
    /// A member's percentage or amount is zero or negative.
    ERANOS_FAILURE_EXPENSE_NON_POSITIVE_SHARE = EranosFailure_ExpenseNonPositiveShare,
    /// A total that does not fit the range of an amount.
    ERANOS_FAILURE_EXPENSE_OVERFLOW = EranosFailure_ExpenseOverflow,
    /// The percentages of a split do not add up to exactly 100 %.
    ERANOS_FAILURE_EXPENSE_PERCENTAGES_DO_NOT_ADD_UP = EranosFailure_ExpensePercentagesDoNotAddUp,
    /// An item quantity of zero.
    ERANOS_FAILURE_EXPENSE_ZERO_QUANTITY = EranosFailure_ExpenseZeroQuantity,
    /// An archived member cannot join a new expense.
    ERANOS_FAILURE_GROUP_ARCHIVED_MEMBER = EranosFailure_GroupArchivedMember,
    /// The built-in Uncategorised category cannot be renamed or removed.
    ERANOS_FAILURE_GROUP_BUILT_IN_CATEGORY = EranosFailure_GroupBuiltInCategory,
    /// A shared group's lock was asked for without waiting, while another call held it.
    ERANOS_FAILURE_GROUP_BUSY = EranosFailure_GroupBusy,
    /// A category with this ID already exists.
    ERANOS_FAILURE_GROUP_DUPLICATE_CATEGORY = EranosFailure_GroupDuplicateCategory,
    /// Another category already has this name, compared by Unicode canonical caseless matching.
    ERANOS_FAILURE_GROUP_DUPLICATE_CATEGORY_NAME = EranosFailure_GroupDuplicateCategoryName,
    /// The same currency appears twice among the rates of a currency change.
    ERANOS_FAILURE_GROUP_DUPLICATE_EXCHANGE_RATE = EranosFailure_GroupDuplicateExchangeRate,
    /// An expense with this ID already exists.
    ERANOS_FAILURE_GROUP_DUPLICATE_EXPENSE = EranosFailure_GroupDuplicateExpense,
    /// A member with this ID already exists.
    ERANOS_FAILURE_GROUP_DUPLICATE_MEMBER = EranosFailure_GroupDuplicateMember,
    /// A transfer with this ID already exists.
    ERANOS_FAILURE_GROUP_DUPLICATE_TRANSFER = EranosFailure_GroupDuplicateTransfer,
    /// A group name that is blank.
    ERANOS_FAILURE_GROUP_EMPTY_NAME = EranosFailure_GroupEmptyName,
    /// The member is still referenced and cannot be removed; archive it instead.
    ERANOS_FAILURE_GROUP_MEMBER_IN_USE = EranosFailure_GroupMemberInUse,
    /// An entry in another currency than the group's has no exchange rate into the group currency: its cost is local, or a currency change gave no
    /// rate for its currency.
    ERANOS_FAILURE_GROUP_MISSING_EXCHANGE_RATE = EranosFailure_GroupMissingExchangeRate,
    /// A total would leave the range of an amount.
    ERANOS_FAILURE_GROUP_OVERFLOW = EranosFailure_GroupOverflow,
    /// An entry in the group currency has a converted cost; only a local cost is in the group currency.
    ERANOS_FAILURE_GROUP_UNEXPECTED_EXCHANGE_RATE = EranosFailure_GroupUnexpectedExchangeRate,
    /// No category with this ID.
    ERANOS_FAILURE_GROUP_UNKNOWN_CATEGORY = EranosFailure_GroupUnknownCategory,
    /// No expense with this ID.
    ERANOS_FAILURE_GROUP_UNKNOWN_EXPENSE = EranosFailure_GroupUnknownExpense,
    /// No member with this ID.
    ERANOS_FAILURE_GROUP_UNKNOWN_MEMBER = EranosFailure_GroupUnknownMember,
    /// No transfer with this ID.
    ERANOS_FAILURE_GROUP_UNKNOWN_TRANSFER = EranosFailure_GroupUnknownTransfer,
    /// A saved group in a format this version does not read: a newer one, whose fields it would drop, or an earlier one.
    ERANOS_FAILURE_GROUP_UNSUPPORTED_FORMAT = EranosFailure_GroupUnsupportedFormat,
    /// Text that is not a UUID.
    ERANOS_FAILURE_ID_INVALID_ID = EranosFailure_IdInvalidId,
    /// A language tag that is not a Unicode BCP 47 locale identifier (UTS #35).
    ERANOS_FAILURE_LANGUAGE_INVALID_LANGUAGE = EranosFailure_LanguageInvalidLanguage,
    /// A first name that is blank.
    ERANOS_FAILURE_MEMBER_EMPTY_FIRST_NAME = EranosFailure_MemberEmptyFirstName,
    /// Arithmetic between amounts of two different currencies.
    ERANOS_FAILURE_MONEY_CURRENCY_MISMATCH = EranosFailure_MoneyCurrencyMismatch,
    /// An amount that is not a decimal number.
    ERANOS_FAILURE_MONEY_INVALID_AMOUNT = EranosFailure_MoneyInvalidAmount,
    /// An exchange rate that is not a positive decimal number with at most 18 decimal places.
    ERANOS_FAILURE_MONEY_INVALID_EXCHANGE_RATE = EranosFailure_MoneyInvalidExchangeRate,
    /// A number of parts that is not a positive decimal number with at most two decimal places.
    ERANOS_FAILURE_MONEY_INVALID_PARTS = EranosFailure_MoneyInvalidParts,
    /// A percentage that is not a decimal number with at most two decimal places.
    ERANOS_FAILURE_MONEY_INVALID_PERCENTAGE = EranosFailure_MoneyInvalidPercentage,
    /// A number or a result that does not fit its range.
    ERANOS_FAILURE_MONEY_OVERFLOW = EranosFailure_MoneyOverflow,
    /// An amount with more decimal places than its currency has.
    ERANOS_FAILURE_MONEY_TOO_MANY_DECIMALS = EranosFailure_MoneyTooManyDecimals,
    /// A currency code that is not an active ISO 4217 code.
    ERANOS_FAILURE_MONEY_UNKNOWN_CURRENCY = EranosFailure_MoneyUnknownCurrency,
    /// The exact solver needs 2ⁿ bytes for `n` open balances, and that table does not fit in the address space or the allocator refused it.
    ERANOS_FAILURE_SETTLEMENT_OUT_OF_MEMORY = EranosFailure_SettlementOutOfMemory,
    /// The amount is zero or negative.
    ERANOS_FAILURE_TRANSFER_NON_POSITIVE_AMOUNT = EranosFailure_TransferNonPositiveAmount,
    /// The sender and the recipient are the same member.
    ERANOS_FAILURE_TRANSFER_SAME_MEMBER = EranosFailure_TransferSameMember,
};

/// The module of the library that refused a call, or the core's own case. The values of EranosErrorKind, by the names of the C layer.
typedef enum EranosErrorKind EranosErrorKind;

/// The values of EranosErrorKind.
enum {
    /// An `AccountError`, about a payment account: its IBAN, username, or label.
    ERANOS_ERROR_KIND_ACCOUNT = EranosErrorKind_Account,
    /// A `CalendarError`, about a date or a time of day.
    ERANOS_ERROR_KIND_CALENDAR = EranosErrorKind_Calendar,
    /// The call was cancelled through its `Cancellation`, before it changed anything.
    ERANOS_ERROR_KIND_CANCELLED = EranosErrorKind_Cancelled,
    /// A `CategoryError`, about a category.
    ERANOS_ERROR_KIND_CATEGORY = EranosErrorKind_Category,
    /// A `ContactError`, about an email address or a phone number.
    ERANOS_ERROR_KIND_CONTACT = EranosErrorKind_Contact,
    /// An `ExpenseError`, about an expense: its amount, split, items, or tip.
    ERANOS_ERROR_KIND_EXPENSE = EranosErrorKind_Expense,
    /// A `GroupError`, about how an entry fits the group, or the group's lock.
    ERANOS_ERROR_KIND_GROUP = EranosErrorKind_Group,
    /// An `IdError`, about an identifier.
    ERANOS_ERROR_KIND_ID = EranosErrorKind_Id,
    /// An entry to add carried an identifier; the group gives a new entry its own.
    ERANOS_ERROR_KIND_IDENTIFIER_GIVEN = EranosErrorKind_IdentifierGiven,
    /// An entry to replace carried no identifier to find it by.
    ERANOS_ERROR_KIND_IDENTIFIER_MISSING = EranosErrorKind_IdentifierMissing,
    /// A saved group that is not JSON of the saved form, at `line` and `column`.
    ERANOS_ERROR_KIND_INVALID_JSON = EranosErrorKind_InvalidJson,
    /// Text that is not UTF-8, which the C and C++ APIs refuse before the library sees it.
    ERANOS_ERROR_KIND_INVALID_TEXT = EranosErrorKind_InvalidText,
    /// A `LanguageError`, about a language tag.
    ERANOS_ERROR_KIND_LANGUAGE = EranosErrorKind_Language,
    /// A `MemberError`, about a member.
    ERANOS_ERROR_KIND_MEMBER = EranosErrorKind_Member,
    /// A `MoneyError`, about an amount, a currency, a rate, or a ratio.
    ERANOS_ERROR_KIND_MONEY = EranosErrorKind_Money,
    /// The read handle was released, so it reads the group no more.
    ERANOS_ERROR_KIND_RELEASED = EranosErrorKind_Released,
    /// A `SettlementError`, about a settlement.
    ERANOS_ERROR_KIND_SETTLEMENT = EranosErrorKind_Settlement,
    /// A `TransferError`, about a transfer.
    ERANOS_ERROR_KIND_TRANSFER = EranosErrorKind_Transfer,
};

#endif // ERANOS_FAILURES_H
