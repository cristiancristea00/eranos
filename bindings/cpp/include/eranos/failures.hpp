/// @file eranos/failures.hpp
/// What a failure reports: the module that refused a call, the failure within the module, and why an IBAN was refused.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_FAILURES_HPP
#define ERANOS_FAILURES_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/detail/declarations.hpp"

#include <cstdint>

namespace eranos {

/// The module of the library that refused a call, or the core's own case.
enum class ErrorKind : std::uint8_t {
    /// A failure of the account module.
    ACCOUNT,
    /// A failure of the calendar module.
    CALENDAR,
    /// The call was cancelled through its cancellation, before it changed the group.
    CANCELLED,
    /// A failure of the category module.
    CATEGORY,
    /// A failure of the contact module.
    CONTACT,
    /// A failure of the expense module.
    EXPENSE,
    /// A failure of the group module.
    GROUP,
    /// A failure of the id module.
    ID,
    /// An entry to add has an identifier, which the group gives it.
    IDENTIFIER_GIVEN,
    /// An entry to replace has no identifier, so the group cannot tell which entry it replaces.
    IDENTIFIER_MISSING,
    /// Text that is not a saved group; the error holds the line and the column where it stops being one.
    INVALID_JSON,
    /// Text that is not UTF-8, which the C and C++ APIs refuse before the library sees it.
    INVALID_TEXT,
    /// A failure of the language module.
    LANGUAGE,
    /// A failure of the member module.
    MEMBER,
    /// A failure of the money module.
    MONEY,
    /// A read through a read handle after its release.
    RELEASED,
    /// A failure of the settlement module.
    SETTLEMENT,
    /// A failure of the transfer module.
    TRANSFER,
};

/// The failure within its module, named after the module and the failure.
enum class Failure : std::uint8_t {
    /// An account whose name is blank.
    ACCOUNT_EMPTY_NAME,
    /// A username that is blank.
    ACCOUNT_EMPTY_USERNAME,
    /// An IBAN that failed validation, and why.
    ACCOUNT_INVALID_IBAN,
    /// A calendar date that does not exist, is outside years 1 to 9999, or is not ISO 8601 text.
    CALENDAR_INVALID_DATE,
    /// A time of day that does not exist, has seconds, or is not ISO 8601 text.
    CALENDAR_INVALID_TIME,
    /// A name that is blank.
    CATEGORY_EMPTY_NAME,
    /// An email address that breaks the RFC 5321 and 5322 rules: no local part or domain around an @, a malformed one, one too long, or whitespace or
    /// an invisible character.
    CONTACT_INVALID_EMAIL,
    /// A phone number that does not start with +, holds a letter or an extension, or is not valid in its country's numbering plan.
    CONTACT_INVALID_PHONE_NUMBER,
    /// The amounts of a split do not add up to the amount being split.
    EXPENSE_AMOUNTS_DO_NOT_ADD_UP,
    /// An amount inside the expense is in a different currency.
    EXPENSE_CURRENCY_MISMATCH,
    /// A member appears more than once in one split.
    EXPENSE_DUPLICATE_MEMBER,
    /// An item whose name is blank.
    EXPENSE_EMPTY_ITEM_NAME,
    /// An expense whose name is blank.
    EXPENSE_EMPTY_NAME,
    /// A split lists no member.
    EXPENSE_EMPTY_SPLIT,
    /// The item line totals do not add up to the expense amount.
    EXPENSE_ITEMS_DO_NOT_ADD_UP,
    /// A fixed tip below zero.
    EXPENSE_NEGATIVE_TIP,
    /// An itemised expense without items.
    EXPENSE_NO_ITEMS,
    /// The amount or unit price is zero or negative.
    EXPENSE_NON_POSITIVE_AMOUNT,
    /// A member's percentage or amount is zero or negative.
    EXPENSE_NON_POSITIVE_SHARE,
    /// A total that does not fit the range of an amount.
    EXPENSE_OVERFLOW,
    /// The percentages of a split do not add up to exactly 100 %.
    EXPENSE_PERCENTAGES_DO_NOT_ADD_UP,
    /// An item quantity of zero.
    EXPENSE_ZERO_QUANTITY,
    /// An archived member cannot join a new expense.
    GROUP_ARCHIVED_MEMBER,
    /// The built-in Uncategorised category cannot be renamed or removed.
    GROUP_BUILT_IN_CATEGORY,
    /// A shared group's lock was asked for without waiting, while another call held it.
    GROUP_BUSY,
    /// A category with this ID already exists.
    GROUP_DUPLICATE_CATEGORY,
    /// Another category already has this name, compared by Unicode canonical caseless matching.
    GROUP_DUPLICATE_CATEGORY_NAME,
    /// The same currency appears twice among the rates of a currency change.
    GROUP_DUPLICATE_EXCHANGE_RATE,
    /// An expense with this ID already exists.
    GROUP_DUPLICATE_EXPENSE,
    /// A member with this ID already exists.
    GROUP_DUPLICATE_MEMBER,
    /// A transfer with this ID already exists.
    GROUP_DUPLICATE_TRANSFER,
    /// A group name that is blank.
    GROUP_EMPTY_NAME,
    /// The member is still referenced and cannot be removed; archive it instead.
    GROUP_MEMBER_IN_USE,
    /// An entry in another currency than the group's has no exchange rate into the group currency: its cost is local, or a currency change gave no
    /// rate for its currency.
    GROUP_MISSING_EXCHANGE_RATE,
    /// A total would leave the range of an amount.
    GROUP_OVERFLOW,
    /// An entry in the group currency has a converted cost; only a local cost is in the group currency.
    GROUP_UNEXPECTED_EXCHANGE_RATE,
    /// No category with this ID.
    GROUP_UNKNOWN_CATEGORY,
    /// No expense with this ID.
    GROUP_UNKNOWN_EXPENSE,
    /// No member with this ID.
    GROUP_UNKNOWN_MEMBER,
    /// No transfer with this ID.
    GROUP_UNKNOWN_TRANSFER,
    /// A saved group in a format this version does not read: a newer one, whose fields it would drop, or an earlier one.
    GROUP_UNSUPPORTED_FORMAT,
    /// Text that is not a UUID.
    ID_INVALID_ID,
    /// A language tag that is not a Unicode BCP 47 locale identifier (UTS #35).
    LANGUAGE_INVALID_LANGUAGE,
    /// A first name that is blank.
    MEMBER_EMPTY_FIRST_NAME,
    /// Arithmetic between amounts of two different currencies.
    MONEY_CURRENCY_MISMATCH,
    /// An amount that is not a decimal number.
    MONEY_INVALID_AMOUNT,
    /// An exchange rate that is not a positive decimal number with at most 18 decimal places.
    MONEY_INVALID_EXCHANGE_RATE,
    /// A number of parts that is not a positive decimal number with at most two decimal places.
    MONEY_INVALID_PARTS,
    /// A percentage that is not a decimal number with at most two decimal places.
    MONEY_INVALID_PERCENTAGE,
    /// A number or a result that does not fit its range.
    MONEY_OVERFLOW,
    /// An amount with more decimal places than its currency has.
    MONEY_TOO_MANY_DECIMALS,
    /// A currency code that is not an active ISO 4217 code.
    MONEY_UNKNOWN_CURRENCY,
    /// The exact solver needs 2ⁿ bytes for n open balances, and that table does not fit in the address space or the allocator refused it.
    SETTLEMENT_OUT_OF_MEMORY,
    /// The amount is zero or negative.
    TRANSFER_NON_POSITIVE_AMOUNT,
    /// The sender and the recipient are the same member.
    TRANSFER_SAME_MEMBER,
};

/// Why an IBAN was refused.
enum class IbanReason : std::uint8_t {
    /// A character other than an ASCII letter, a digit, or whitespace (which is dropped).
    INVALID_CHARACTER,
    /// The check digits are 00, 01, or 99, which ISO 13616 never issues, or the ISO 7064 mod-97 check failed: the number contains a typo.
    INVALID_CHECKSUM,
    /// Not two letters followed by two check digits.
    INVALID_FORMAT,
    /// A letter where the country's account number has a digit, or a digit where it has a letter, by the BBAN structure of the SWIFT IBAN registry.
    INVALID_STRUCTURE,
    /// A country code without an entry in the SWIFT IBAN registry.
    UNKNOWN_COUNTRY,
    /// The wrong number of characters for the country.
    WRONG_LENGTH,
};

namespace detail {

[[nodiscard]] inline ErrorKind fromDetail(EranosErrorKind const value) noexcept;
[[nodiscard]] inline Failure fromDetail(EranosFailure const value) noexcept;
[[nodiscard]] inline IbanReason fromDetail(EranosIbanReason const value) noexcept;

} // namespace detail

} // namespace eranos

#endif // ERANOS_FAILURES_HPP
