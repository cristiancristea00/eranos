/// @file eranos/error.h
/// The error that a call the library refuses returns.

#ifndef ERANOS_ERROR_H
#define ERANOS_ERROR_H

#include "EranosError.h"
#include "eranos/detail.h"
#include "eranos/failures.h"
#include "eranos/identifiers.h"
#include "eranos/money.h"
#include "eranos/text.h"

#include <stdint.h>

/// A failure a call reports: its kind, the failure within its module, the library's message, and what the failure holds, each getter present for the
/// failures that hold its field.
typedef struct EranosError EranosError;

/// The module that refused the call, or the core's own case.
///
/// @param error The error
/// @return The result
[[nodiscard]] static inline EranosErrorKind eranosErrorKind(EranosError const *const error) { return EranosError_kind(error); }

/// The failure within its module, absent for the core's own cases.
///
/// @param error The error
/// @param failure Receives the failure, when there is one
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorFailure(EranosError const *const error, EranosFailure *const failure) {
    EranosError_failure_result const result = EranosError_failure(error);
    return eranosDetailPresentFailure(result.is_ok, &result.ok, failure);
}

/// Writes the library's message: lowercase, without a full stop, and never the text it refused.
///
/// @param error The error
/// @param message Receives the message, replacing what it held
static inline void eranosErrorMessage(EranosError const *const error, EranosText *const message) {
    eranosDetailClear(message);
    EranosError_message(error, message);
    eranosDetailFinish(message);
}

/// What the amounts add up to. Present for the failures ERANOS_FAILURE_EXPENSE_AMOUNTS_DO_NOT_ADD_UP and ERANOS_FAILURE_EXPENSE_ITEMS_DO_NOT_ADD_UP.
///
/// @param error The error
/// @return A copy of what the failure holds, which the caller destroys, or nullptr when it holds none
[[nodiscard]] static inline EranosMoney *eranosErrorActual(EranosError const *const error) { return EranosError_actual(error); }

/// How many decimal places the currency has. Present for the failure ERANOS_FAILURE_MONEY_TOO_MANY_DECIMALS.
///
/// @param error The error
/// @param allowed Receives the allowed, when there is one
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorAllowed(EranosError const *const error, int64_t *const allowed) {
    EranosError_allowed_result const result = EranosError_allowed(error);
    return eranosDetailPresentInt64(result.is_ok, &result.ok, allowed);
}

/// How many open balances the plan has. Present for the failure ERANOS_FAILURE_SETTLEMENT_OUT_OF_MEMORY.
///
/// @param error The error
/// @param balances Receives the balances, when there is one
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorBalances(EranosError const *const error, int64_t *const balances) {
    EranosError_balances_result const result = EranosError_balances(error);
    return eranosDetailPresentInt64(result.is_ok, &result.ok, balances);
}

/// The identifier of the category. Present for the failures ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_CATEGORY and
/// ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_CATEGORY. Writes it, if present; absent otherwise.
///
/// @param error The error
/// @param category Receives the category, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorCategory(EranosError const *const error, EranosText *const category) {
    eranosDetailClear(category);
    bool const present = EranosError_category(error, category).is_ok;
    eranosDetailFinish(category);
    return present;
}

/// The column of the saved group where the text stops being one, from 1. Present for the kind ERANOS_ERROR_KIND_INVALID_JSON.
///
/// @param error The error
/// @param column Receives the column, when there is one
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorColumn(EranosError const *const error, int64_t *const column) {
    EranosError_column_result const result = EranosError_column(error);
    return eranosDetailPresentInt64(result.is_ok, &result.ok, column);
}

/// The ISO 4217 code of the currency. Present for the failures ERANOS_FAILURE_MONEY_TOO_MANY_DECIMALS,
/// ERANOS_FAILURE_EXPENSE_GROUP_MISSING_EXCHANGE_RATE, and ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_EXCHANGE_RATE. Writes it, if present; absent
/// otherwise.
///
/// @param error The error
/// @param currency Receives the currency, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorCurrency(EranosError const *const error, EranosText *const currency) {
    eranosDetailClear(currency);
    bool const present = EranosError_currency(error, currency).is_ok;
    eranosDetailFinish(currency);
    return present;
}

/// The category that has the name. Present for the failure ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_CATEGORY_NAME. Writes it, if present; absent
/// otherwise.
///
/// @param error The error
/// @param existing Receives the existing, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorExisting(EranosError const *const error, EranosText *const existing) {
    eranosDetailClear(existing);
    bool const present = EranosError_existing(error, existing).is_ok;
    eranosDetailFinish(existing);
    return present;
}

/// The amount being split. Present for the failures ERANOS_FAILURE_EXPENSE_AMOUNTS_DO_NOT_ADD_UP and ERANOS_FAILURE_EXPENSE_ITEMS_DO_NOT_ADD_UP.
///
/// @param error The error
/// @return A copy of what the failure holds, which the caller destroys, or nullptr when it holds none
[[nodiscard]] static inline EranosMoney *eranosErrorExpectedAmount(EranosError const *const error) { return EranosError_expected_amount(error); }

/// The expense's currency. Present for the failure ERANOS_FAILURE_EXPENSE_CURRENCY_MISMATCH. Writes it, if present; absent otherwise.
///
/// @param error The error
/// @param expectedCurrency Receives the expected currency, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorExpectedCurrency(EranosError const *const error, EranosText *const expectedCurrency) {
    eranosDetailClear(expectedCurrency);
    bool const present = EranosError_expected_currency(error, expectedCurrency).is_ok;
    eranosDetailFinish(expectedCurrency);
    return present;
}

/// How many characters the country's IBANs have. Present for the failure ERANOS_FAILURE_PAYMENT_ACCOUNT_INVALID_IBAN with the reason
/// ERANOS_IBAN_REASON_WRONG_LENGTH.
///
/// @param error The error
/// @param expectedLength Receives the expected length, when there is one
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorExpectedLength(EranosError const *const error, int64_t *const expectedLength) {
    EranosError_expected_length_result const result = EranosError_expected_length(error);
    return eranosDetailPresentInt64(result.is_ok, &result.ok, expectedLength);
}

/// The identifier of the expense. Present for the failures ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_EXPENSE and
/// ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_EXPENSE. Writes it, if present; absent otherwise.
///
/// @param error The error
/// @param expense Receives the expense, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorExpense(EranosError const *const error, EranosText *const expense) {
    eranosDetailClear(expense);
    bool const present = EranosError_expense(error, expense).is_ok;
    eranosDetailFinish(expense);
    return present;
}

/// The expenses that name the member. Present for the failure ERANOS_FAILURE_EXPENSE_GROUP_MEMBER_IN_USE.
///
/// @param error The error
/// @return A copy of what the failure holds, which the caller destroys, or nullptr when it holds none
[[nodiscard]] static inline EranosTextList *eranosErrorExpenses(EranosError const *const error) { return EranosError_expenses(error); }

/// The format the saved group names. Present for the failure ERANOS_FAILURE_EXPENSE_GROUP_UNSUPPORTED_FORMAT.
///
/// @param error The error
/// @param format Receives the format, when there is one
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorFormat(EranosError const *const error, int64_t *const format) {
    EranosError_format_result const result = EranosError_format(error);
    return eranosDetailPresentInt64(result.is_ok, &result.ok, format);
}

/// The currency of the amount inside it. Present for the failure ERANOS_FAILURE_EXPENSE_CURRENCY_MISMATCH. Writes it, if present; absent otherwise.
///
/// @param error The error
/// @param foundCurrency Receives the found currency, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorFoundCurrency(EranosError const *const error, EranosText *const foundCurrency) {
    eranosDetailClear(foundCurrency);
    bool const present = EranosError_found_currency(error, foundCurrency).is_ok;
    eranosDetailFinish(foundCurrency);
    return present;
}

/// How many characters the IBAN has. Present for the failure ERANOS_FAILURE_PAYMENT_ACCOUNT_INVALID_IBAN with the reason
/// ERANOS_IBAN_REASON_WRONG_LENGTH.
///
/// @param error The error
/// @param foundLength Receives the found length, when there is one
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorFoundLength(EranosError const *const error, int64_t *const foundLength) {
    EranosError_found_length_result const result = EranosError_found_length(error);
    return eranosDetailPresentInt64(result.is_ok, &result.ok, foundLength);
}

/// The currency of the first amount. Present for the failure ERANOS_FAILURE_MONEY_CURRENCY_MISMATCH. Writes it, if present; absent otherwise.
///
/// @param error The error
/// @param left Receives the left, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorLeft(EranosError const *const error, EranosText *const left) {
    eranosDetailClear(left);
    bool const present = EranosError_left(error, left).is_ok;
    eranosDetailFinish(left);
    return present;
}

/// The line of the saved group where the text stops being one, from 1. Present for the kind ERANOS_ERROR_KIND_INVALID_JSON.
///
/// @param error The error
/// @param line Receives the line, when there is one
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorLine(EranosError const *const error, int64_t *const line) {
    EranosError_line_result const result = EranosError_line(error);
    return eranosDetailPresentInt64(result.is_ok, &result.ok, line);
}

/// The identifier of the member. Present for the failures ERANOS_FAILURE_EXPENSE_DUPLICATE_MEMBER, ERANOS_FAILURE_EXPENSE_NON_POSITIVE_SHARE,
/// ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_MEMBER, ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_MEMBER, ERANOS_FAILURE_EXPENSE_GROUP_ARCHIVED_MEMBER, and
/// ERANOS_FAILURE_EXPENSE_GROUP_MEMBER_IN_USE. Writes it, if present; absent otherwise.
///
/// @param error The error
/// @param member Receives the member, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorMember(EranosError const *const error, EranosText *const member) {
    eranosDetailClear(member);
    bool const present = EranosError_member(error, member).is_ok;
    eranosDetailFinish(member);
    return present;
}

/// The check the IBAN failed. Present for the failure ERANOS_FAILURE_PAYMENT_ACCOUNT_INVALID_IBAN.
///
/// @param error The error
/// @param reason Receives the reason, when there is one
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorReason(EranosError const *const error, EranosIbanReason *const reason) {
    EranosError_reason_result const result = EranosError_reason(error);
    return eranosDetailPresentIbanReason(result.is_ok, &result.ok, reason);
}

/// The currency of the second amount. Present for the failure ERANOS_FAILURE_MONEY_CURRENCY_MISMATCH. Writes it, if present; absent otherwise.
///
/// @param error The error
/// @param right Receives the right, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorRight(EranosError const *const error, EranosText *const right) {
    eranosDetailClear(right);
    bool const present = EranosError_right(error, right).is_ok;
    eranosDetailFinish(right);
    return present;
}

/// What the percentages add up to, in basis points. Present for the failure ERANOS_FAILURE_EXPENSE_PERCENTAGES_DO_NOT_ADD_UP.
///
/// @param error The error
/// @param totalBasisPoints Receives the total basis points, when there is one
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorTotalBasisPoints(EranosError const *const error, int64_t *const totalBasisPoints) {
    EranosError_total_basis_points_result const result = EranosError_total_basis_points(error);
    return eranosDetailPresentInt64(result.is_ok, &result.ok, totalBasisPoints);
}

/// The identifier of the transfer. Present for the failures ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_TRANSFER and
/// ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_TRANSFER. Writes it, if present; absent otherwise.
///
/// @param error The error
/// @param transfer Receives the transfer, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosErrorTransfer(EranosError const *const error, EranosText *const transfer) {
    eranosDetailClear(transfer);
    bool const present = EranosError_transfer(error, transfer).is_ok;
    eranosDetailFinish(transfer);
    return present;
}

/// The transfers that name the member. Present for the failure ERANOS_FAILURE_EXPENSE_GROUP_MEMBER_IN_USE.
///
/// @param error The error
/// @return A copy of what the failure holds, which the caller destroys, or nullptr when it holds none
[[nodiscard]] static inline EranosTextList *eranosErrorTransfers(EranosError const *const error) { return EranosError_transfers(error); }

/// Destroys @p error; destroying nullptr does nothing, as free() does.
///
/// @param error The error, which no call uses after this one, or nullptr
static inline void eranosErrorDestroy(EranosError *const error) {
    if (nullptr != error) {
        EranosError_destroy(error);
    }
}

#endif // ERANOS_ERROR_H
