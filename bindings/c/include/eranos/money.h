/// @file eranos/money.h
/// Amounts: whole numbers of minor units of a currency.

#ifndef ERANOS_MONEY_H
#define ERANOS_MONEY_H

#include "EranosMoney.h"
#include "eranos/detail.h"
#include "eranos/text.h"

#include <stdint.h>

/// An amount: a whole number of minor units of a currency, cents for the euro.
typedef struct EranosMoney EranosMoney;

/// A new amount of its required fields; every other field starts absent, empty, or at its default.
///
/// @param minorUnits The amount in minor units: cents for the euro, none for the yen
/// @param currency The currency, as an ISO 4217 code, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param money Receives the money on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosMoneyCreate(int64_t const minorUnits, char const *const currency, EranosMoney **const money) {
    EranosMoney_create_result const result = EranosMoney_create(minorUnits, eranosDetailView(currency));
    *money = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The amount in minor units: cents for the euro, none for the yen.
///
/// @param money The money
/// @return The amount in minor units: cents for the euro, none for the yen
[[nodiscard]] static inline int64_t eranosMoneyMinorUnits(EranosMoney const *const money) { return EranosMoney_minor_units(money); }

/// The currency, as an ISO 4217 code.
///
/// @param money The money
/// @param currency Receives the currency, as an ISO 4217 code, replacing what it held
static inline void eranosMoneyCurrency(EranosMoney const *const money, EranosText *const currency) {
    eranosDetailClear(currency);
    EranosMoney_currency(money, currency);
    eranosDetailFinish(currency);
}

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

/// Writes the amount as the library does: "12.50 EUR".
///
/// @param money The money
/// @param text Receives the text, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for a
///   currency the library does not list
[[nodiscard]] static inline EranosError *eranosMoneyFormat(EranosMoney const *const money, EranosText *const text) {
    eranosDetailClear(text);
    EranosMoney_format_result const result = EranosMoney_format(money, text);
    eranosDetailFinish(text);
    return eranosDetailErrorOf(result.is_ok, &result.err);
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
