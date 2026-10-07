/// @file eranos/failures.h
/// What a failure reports: the module that refused a call, the failure within the module, and why an IBAN was refused.

#ifndef ERANOS_FAILURES_H
#define ERANOS_FAILURES_H

#include "EranosErrorKind.h"
#include "EranosFailure.h"
#include "EranosIbanReason.h"

/// Why an IBAN was refused. The values of EranosIbanReason, by the names of the C layer.
typedef enum EranosIbanReason EranosIbanReason;

/// A character other than an ASCII letter, a digit, or whitespace (which is dropped).
static constexpr EranosIbanReason ERANOS_IBAN_REASON_INVALID_CHARACTER = EranosIbanReason_InvalidCharacter;
/// Not two letters followed by two check digits.
static constexpr EranosIbanReason ERANOS_IBAN_REASON_INVALID_FORMAT = EranosIbanReason_InvalidFormat;
/// A country code without an entry in the SWIFT IBAN registry.
static constexpr EranosIbanReason ERANOS_IBAN_REASON_UNKNOWN_COUNTRY = EranosIbanReason_UnknownCountry;
/// The wrong number of characters for the country.
static constexpr EranosIbanReason ERANOS_IBAN_REASON_WRONG_LENGTH = EranosIbanReason_WrongLength;
/// A letter where the country's account number has a digit, or a digit where it has a letter, by the BBAN structure of the SWIFT IBAN registry.
static constexpr EranosIbanReason ERANOS_IBAN_REASON_INVALID_STRUCTURE = EranosIbanReason_InvalidStructure;
/// The check digits are 00, 01, or 99, which ISO 13616 never issues, or the ISO 7064 mod-97 check failed: the number contains a typo.
static constexpr EranosIbanReason ERANOS_IBAN_REASON_INVALID_CHECKSUM = EranosIbanReason_InvalidChecksum;

/// The failure within its module, named after the module and the failure. The values of EranosFailure, by the names of the C layer.
typedef enum EranosFailure EranosFailure;

/// Text that is not a UUID.
static constexpr EranosFailure ERANOS_FAILURE_ID_INVALID_ID = EranosFailure_IdInvalidId;
/// An account whose name is blank.
static constexpr EranosFailure ERANOS_FAILURE_ACCOUNT_EMPTY_NAME = EranosFailure_AccountEmptyName;
/// An IBAN that failed validation, and why.
static constexpr EranosFailure ERANOS_FAILURE_ACCOUNT_INVALID_IBAN = EranosFailure_AccountInvalidIban;
/// A username that is blank.
static constexpr EranosFailure ERANOS_FAILURE_ACCOUNT_EMPTY_USERNAME = EranosFailure_AccountEmptyUsername;
/// A calendar date that does not exist, is outside years 1 to 9999, or is not ISO 8601 text.
static constexpr EranosFailure ERANOS_FAILURE_CALENDAR_INVALID_DATE = EranosFailure_CalendarInvalidDate;
/// A time of day that does not exist, has seconds, or is not ISO 8601 text.
static constexpr EranosFailure ERANOS_FAILURE_CALENDAR_INVALID_TIME = EranosFailure_CalendarInvalidTime;
/// A name that is blank.
static constexpr EranosFailure ERANOS_FAILURE_CATEGORY_EMPTY_NAME = EranosFailure_CategoryEmptyName;
/// An email address that breaks the RFC 5321 and 5322 rules: no local part or domain around an `@`, a malformed one, one too long, or whitespace
/// or an invisible character.
static constexpr EranosFailure ERANOS_FAILURE_CONTACT_INVALID_EMAIL = EranosFailure_ContactInvalidEmail;
/// A phone number that does not start with `+`, holds a letter or an extension, or is not valid in its country's numbering plan.
static constexpr EranosFailure ERANOS_FAILURE_CONTACT_INVALID_PHONE_NUMBER = EranosFailure_ContactInvalidPhoneNumber;
/// An amount that is not a decimal number.
static constexpr EranosFailure ERANOS_FAILURE_MONEY_INVALID_AMOUNT = EranosFailure_MoneyInvalidAmount;
/// An amount with more decimal places than its currency has.
static constexpr EranosFailure ERANOS_FAILURE_MONEY_TOO_MANY_DECIMALS = EranosFailure_MoneyTooManyDecimals;
/// Arithmetic between amounts of two different currencies.
static constexpr EranosFailure ERANOS_FAILURE_MONEY_CURRENCY_MISMATCH = EranosFailure_MoneyCurrencyMismatch;
/// A currency code that is not an active ISO 4217 code.
static constexpr EranosFailure ERANOS_FAILURE_MONEY_UNKNOWN_CURRENCY = EranosFailure_MoneyUnknownCurrency;
/// An exchange rate that is not a positive decimal number with at most 18 decimal places.
static constexpr EranosFailure ERANOS_FAILURE_MONEY_INVALID_EXCHANGE_RATE = EranosFailure_MoneyInvalidExchangeRate;
/// A number of parts that is not a positive decimal number with at most two decimal places.
static constexpr EranosFailure ERANOS_FAILURE_MONEY_INVALID_PARTS = EranosFailure_MoneyInvalidParts;
/// A percentage that is not a decimal number with at most two decimal places.
static constexpr EranosFailure ERANOS_FAILURE_MONEY_INVALID_PERCENTAGE = EranosFailure_MoneyInvalidPercentage;
/// A number or a result that does not fit its range.
static constexpr EranosFailure ERANOS_FAILURE_MONEY_OVERFLOW = EranosFailure_MoneyOverflow;
/// A first name that is blank.
static constexpr EranosFailure ERANOS_FAILURE_MEMBER_EMPTY_FIRST_NAME = EranosFailure_MemberEmptyFirstName;
/// An expense whose name is blank.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_EMPTY_NAME = EranosFailure_ExpenseEmptyName;
/// The amount or unit price is zero or negative.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_NON_POSITIVE_AMOUNT = EranosFailure_ExpenseNonPositiveAmount;
/// An amount inside the expense is in a different currency.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_CURRENCY_MISMATCH = EranosFailure_ExpenseCurrencyMismatch;
/// A split lists no member.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_EMPTY_SPLIT = EranosFailure_ExpenseEmptySplit;
/// A member appears more than once in one split.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_DUPLICATE_MEMBER = EranosFailure_ExpenseDuplicateMember;
/// A member's percentage or amount is zero or negative.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_NON_POSITIVE_SHARE = EranosFailure_ExpenseNonPositiveShare;
/// The amounts of a split do not add up to the amount being split.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_AMOUNTS_DO_NOT_ADD_UP = EranosFailure_ExpenseAmountsDoNotAddUp;
/// The percentages of a split do not add up to exactly 100 %.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_PERCENTAGES_DO_NOT_ADD_UP = EranosFailure_ExpensePercentagesDoNotAddUp;
/// An itemised expense without items.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_NO_ITEMS = EranosFailure_ExpenseNoItems;
/// An item whose name is blank.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_EMPTY_ITEM_NAME = EranosFailure_ExpenseEmptyItemName;
/// An item quantity of zero.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_ZERO_QUANTITY = EranosFailure_ExpenseZeroQuantity;
/// The item line totals do not add up to the expense amount.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_ITEMS_DO_NOT_ADD_UP = EranosFailure_ExpenseItemsDoNotAddUp;
/// A fixed tip below zero.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_NEGATIVE_TIP = EranosFailure_ExpenseNegativeTip;
/// A total that does not fit the range of an amount.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_OVERFLOW = EranosFailure_ExpenseOverflow;
/// The exact solver needs 2ⁿ bytes for `n` open balances, and that table does not fit in the address space or the allocator refused it.
static constexpr EranosFailure ERANOS_FAILURE_SETTLEMENT_OUT_OF_MEMORY = EranosFailure_SettlementOutOfMemory;
/// The sender and the recipient are the same member.
static constexpr EranosFailure ERANOS_FAILURE_TRANSFER_SAME_MEMBER = EranosFailure_TransferSameMember;
/// The amount is zero or negative.
static constexpr EranosFailure ERANOS_FAILURE_TRANSFER_NON_POSITIVE_AMOUNT = EranosFailure_TransferNonPositiveAmount;
/// A saved group in a format this version does not read: a newer one, whose fields it would drop, or an earlier one.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_UNSUPPORTED_FORMAT = EranosFailure_ExpenseGroupUnsupportedFormat;
/// A group name that is blank.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_EMPTY_NAME = EranosFailure_ExpenseGroupEmptyName;
/// An entry in another currency than the group's has no exchange rate into the group currency: its cost is local, or a currency change gave no
/// rate for its currency.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_MISSING_EXCHANGE_RATE = EranosFailure_ExpenseGroupMissingExchangeRate;
/// An entry in the group currency has a converted cost; only a local cost is in the group currency.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_UNEXPECTED_EXCHANGE_RATE = EranosFailure_ExpenseGroupUnexpectedExchangeRate;
/// The same currency appears twice among the rates of a currency change.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_EXCHANGE_RATE = EranosFailure_ExpenseGroupDuplicateExchangeRate;
/// A member with this ID already exists.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_MEMBER = EranosFailure_ExpenseGroupDuplicateMember;
/// No member with this ID.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_MEMBER = EranosFailure_ExpenseGroupUnknownMember;
/// An archived member cannot join a new expense.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_ARCHIVED_MEMBER = EranosFailure_ExpenseGroupArchivedMember;
/// The member is still referenced and cannot be removed; archive it instead.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_MEMBER_IN_USE = EranosFailure_ExpenseGroupMemberInUse;
/// A category with this ID already exists.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_CATEGORY = EranosFailure_ExpenseGroupDuplicateCategory;
/// Another category already has this name, compared by Unicode canonical caseless matching.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_CATEGORY_NAME = EranosFailure_ExpenseGroupDuplicateCategoryName;
/// No category with this ID.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_CATEGORY = EranosFailure_ExpenseGroupUnknownCategory;
/// The built-in Uncategorised category cannot be renamed or removed.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_BUILT_IN_CATEGORY = EranosFailure_ExpenseGroupBuiltInCategory;
/// An expense with this ID already exists.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_EXPENSE = EranosFailure_ExpenseGroupDuplicateExpense;
/// No expense with this ID.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_EXPENSE = EranosFailure_ExpenseGroupUnknownExpense;
/// A transfer with this ID already exists.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_TRANSFER = EranosFailure_ExpenseGroupDuplicateTransfer;
/// No transfer with this ID.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_TRANSFER = EranosFailure_ExpenseGroupUnknownTransfer;
/// A shared group's lock was asked for without waiting, while another call held it.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_BUSY = EranosFailure_ExpenseGroupBusy;
/// A total would leave the range of an amount.
static constexpr EranosFailure ERANOS_FAILURE_EXPENSE_GROUP_OVERFLOW = EranosFailure_ExpenseGroupOverflow;
/// A language tag that is not a Unicode BCP 47 locale identifier (UTS #35).
static constexpr EranosFailure ERANOS_FAILURE_LANGUAGE_INVALID_LANGUAGE = EranosFailure_LanguageInvalidLanguage;

/// The module of the library that refused a call, or the core's own case. The values of EranosErrorKind, by the names of the C layer.
typedef enum EranosErrorKind EranosErrorKind;

/// An `IdError`, about an identifier.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_ID = EranosErrorKind_Id;
/// An `AccountError`, about a payment account: its IBAN, username, or label.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_ACCOUNT = EranosErrorKind_Account;
/// A `CalendarError`, about a date or a time of day.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_CALENDAR = EranosErrorKind_Calendar;
/// A `CategoryError`, about a category.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_CATEGORY = EranosErrorKind_Category;
/// A `ContactError`, about an email address or a phone number.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_CONTACT = EranosErrorKind_Contact;
/// A `MoneyError`, about an amount, a currency, a rate, or a ratio.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_MONEY = EranosErrorKind_Money;
/// A `MemberError`, about a member.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_MEMBER = EranosErrorKind_Member;
/// An `ExpenseError`, about an expense: its amount, split, items, or tip.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_EXPENSE = EranosErrorKind_Expense;
/// A `SettlementError`, about a settlement.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_SETTLEMENT = EranosErrorKind_Settlement;
/// A `TransferError`, about a transfer.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_TRANSFER = EranosErrorKind_Transfer;
/// An `ExpenseGroupError`, about how an entry fits the group, or the group's lock.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_EXPENSE_GROUP = EranosErrorKind_ExpenseGroup;
/// A `LanguageError`, about a language tag.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_LANGUAGE = EranosErrorKind_Language;
/// An entry to add carried an identifier; the group gives a new entry its own.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_IDENTIFIER_GIVEN = EranosErrorKind_IdentifierGiven;
/// An entry to replace carried no identifier to find it by.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_IDENTIFIER_MISSING = EranosErrorKind_IdentifierMissing;
/// A saved group that is not JSON of the saved form, at `line` and `column`.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_INVALID_JSON = EranosErrorKind_InvalidJson;
/// Text that is not UTF-8, which the C and C++ APIs refuse before the library sees it.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_INVALID_TEXT = EranosErrorKind_InvalidText;
/// The call was cancelled through its `Cancellation`, before it changed anything.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_CANCELLED = EranosErrorKind_Cancelled;
/// The read handle was released, so it reads the group no more.
static constexpr EranosErrorKind ERANOS_ERROR_KIND_RELEASED = EranosErrorKind_Released;

#endif // ERANOS_FAILURES_H
