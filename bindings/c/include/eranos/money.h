/// @file eranos/money.h
/// Amounts: whole numbers of minor units of a currency.

#ifndef ERANOS_MONEY_H
#define ERANOS_MONEY_H

#include "EranosMoney.h"
#include "eranos/currency_code.h"
#include "eranos/detail.h"
#include "eranos/text.h"

#include <stdint.h>

/// An amount: a whole number of minor units of a currency, cents for the euro.
typedef struct EranosMoney EranosMoney;

/// @p minorUnits minor units of @p currency, whose details come from the library: from eranosCurrencyDetailsOf(), the list of every currency, or
/// another amount, so the amount needs no lookup and no check of its own.
///
/// @param minorUnits The amount in minor units: cents for the euro, none for the yen
/// @param currency The currency, with what ISO 4217 says of it, which the amount copies
/// @return The money, which the caller destroys
[[nodiscard]] static inline EranosMoney *eranosMoneyCreate(int64_t const minorUnits, EranosCurrencyDetails const *const currency) {
    return EranosMoney_create(minorUnits, currency);
}

/// The amount in minor units: cents for the euro, none for the yen.
///
/// @param money The money
/// @return The amount in minor units: cents for the euro, none for the yen
[[nodiscard]] static inline int64_t eranosMoneyMinorUnits(EranosMoney const *const money) { return EranosMoney_minor_units(money); }

/// The currency, with what ISO 4217 says of it.
///
/// @param money The money
/// @return A copy of the currency details, which the caller destroys
[[nodiscard]] static inline EranosCurrencyDetails *eranosMoneyCurrency(EranosMoney const *const money) { return EranosMoney_currency(money); }

/// The amount written `text` in the currency `currency`: "12.50", "12.5", or "-3".
///
/// @param text The text, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param currency The currency, as an ISO 4217 code, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param money Receives the money on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for text that
///   is not an amount of the currency, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosMoneyParse(char const *const text, char const *const currency, EranosMoney **const money) {
    EranosMoney_parse_result const result = EranosMoney_parse(eranosDetailView(text), eranosDetailView(currency));
    *money = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Writes the amount as the library does, with the decimal places of its currency's details: "12.50 EUR".
///
/// @param money The money
/// @param text Receives the text, replacing what it held
static inline void eranosMoneyFormat(EranosMoney const *const money, EranosText *const text) {
    eranosDetailClear(text);
    EranosMoney_format(money, text);
    eranosDetailFinish(text);
}

/// This amount plus `other`.
///
/// @param money The money
/// @param other The other
/// @param sum Receives the sum on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for amounts
///   in two currencies, or a sum out of range
[[nodiscard]] static inline EranosError *eranosMoneyAdd(EranosMoney const *const money, EranosMoney const *const other, EranosMoney **const sum) {
    EranosMoney_add_result const result = EranosMoney_add(money, other);
    *sum = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// This amount minus `other`.
///
/// @param money The money
/// @param other The other
/// @param difference Receives the difference on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for amounts
///   in two currencies, or a difference out of range
[[nodiscard]] static inline EranosError *eranosMoneySubtract(EranosMoney const *const money, EranosMoney const *const other,
                                                             EranosMoney **const difference) {
    EranosMoney_subtract_result const result = EranosMoney_subtract(money, other);
    *difference = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p money; destroying nullptr does nothing, as free() does.
///
/// @param money The money, which no call uses after this one, or nullptr
static inline void eranosMoneyDestroy(EranosMoney *const money) {
    if (nullptr != money) {
        EranosMoney_destroy(money);
    }
}

#endif // ERANOS_MONEY_H
