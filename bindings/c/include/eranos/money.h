/// @file eranos/money.h
/// Amounts: whole numbers of minor units of a currency.

#ifndef ERANOS_MONEY_H
#define ERANOS_MONEY_H

#include "EranosMoney.h"
#include "eranos/detail.h"
#include "eranos/text.h"

#include <stddef.h>
#include <stdint.h>

/// An amount: a whole number of minor units of a currency, cents for the euro.
typedef struct EranosMoney EranosMoney;

/// A new money of its required fields; every other field starts absent, empty, or at its default.
///
/// @param currency The currency, as an ISO 4217 code, as UTF-8 ended by a NUL; NULL reads as empty text
/// @param minorUnits The amount in minor units: cents for the euro, none for the yen
/// @param money Receives the money on success, which the caller destroys, and NULL on failure
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
static inline EranosError *eranosMoneyCreate(char const *const currency, int64_t const minorUnits, EranosMoney **const money) {
    EranosMoney_create_result const result = EranosMoney_create(eranosDetailView(currency), minorUnits);
    *money = result.is_ok ? result.ok : NULL;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The currency, as an ISO 4217 code.
///
/// @param money The money
/// @param currency Receives the currency, as an ISO 4217 code, replacing what it held
static inline void eranosMoneyCurrency(EranosMoney const *const money, EranosText *const currency) {
    eranosDetailClear(currency);
    EranosMoney_currency(money, currency);
    eranosDetailFinish(currency);
}

/// The amount in minor units: cents for the euro, none for the yen.
///
/// @param money The money
/// @return The amount in minor units: cents for the euro, none for the yen
static inline int64_t eranosMoneyMinorUnits(EranosMoney const *const money) { return EranosMoney_minor_units(money); }

/// The amount written `text` in the currency `currency`: "12.50", "12.5", or "-3".
///
/// @param text The text, as UTF-8 ended by a NUL; NULL reads as empty text
/// @param currency The currency, as an ISO 4217 code, as UTF-8 ended by a NUL; NULL reads as empty text
/// @param money Receives the money on success, which the caller destroys, and NULL on failure
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for text that
///   is not an amount of the currency, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
static inline EranosError *eranosMoneyParse(char const *const text, char const *const currency, EranosMoney **const money) {
    EranosMoney_parse_result const result = EranosMoney_parse(eranosDetailView(text), eranosDetailView(currency));
    *money = result.is_ok ? result.ok : NULL;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Writes the amount as the library does: "12.50 EUR".
///
/// @param money The money
/// @param text Receives the text, replacing what it held
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for a currency
///   the library does not list
static inline EranosError *eranosMoneyFormat(EranosMoney const *const money, EranosText *const text) {
    eranosDetailClear(text);
    EranosMoney_format_result const result = EranosMoney_format(money, text);
    eranosDetailFinish(text);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// This amount plus `other`.
///
/// @param money The money
/// @param other The other
/// @param sum Receives the sum on success, which the caller destroys, and NULL on failure
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for amounts in
///   two currencies, or a sum out of range
static inline EranosError *eranosMoneyAdd(EranosMoney const *const money, EranosMoney const *const other, EranosMoney **const sum) {
    EranosMoney_add_result const result = EranosMoney_add(money, other);
    *sum = result.is_ok ? result.ok : NULL;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// This amount minus `other`.
///
/// @param money The money
/// @param other The other
/// @param difference Receives the difference on success, which the caller destroys, and NULL on failure
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for amounts in
///   two currencies, or a difference out of range
static inline EranosError *eranosMoneySubtract(EranosMoney const *const money, EranosMoney const *const other, EranosMoney **const difference) {
    EranosMoney_subtract_result const result = EranosMoney_subtract(money, other);
    *difference = result.is_ok ? result.ok : NULL;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p money; destroying NULL does nothing, as free() does.
///
/// @param money The money, which no call uses after this one, or NULL
static inline void eranosMoneyDestroy(EranosMoney *const money) {
    if (NULL != money) {
        EranosMoney_destroy(money);
    }
}

#endif // ERANOS_MONEY_H
