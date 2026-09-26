/// @file eranos/currency.h
/// The currencies of the ISO 4217 list, with what the library knows of each.

#ifndef ERANOS_CURRENCY_H
#define ERANOS_CURRENCY_H

#include "EranosCurrencyDetails.h"
#include "EranosCurrencyDetailsList.h"
#include "eranos/detail.h"
#include "eranos/text.h"

#include <stddef.h>
#include <stdint.h>

/// A currency of the ISO 4217 list, with what the library knows of it.
typedef struct EranosCurrencyDetails EranosCurrencyDetails;

/// The ISO 4217 code.
///
/// @param details The currency details
/// @param code Receives the ISO 4217 code, replacing what it held
static inline void eranosCurrencyDetailsCode(EranosCurrencyDetails const *const details, EranosText *const code) {
    eranosDetailClear(code);
    EranosCurrencyDetails_code(details, code);
    eranosDetailFinish(code);
}

/// How many decimal places its amounts have.
///
/// @param details The currency details
/// @return How many decimal places its amounts have
[[nodiscard]] static inline int32_t eranosCurrencyDetailsDecimalPlaces(EranosCurrencyDetails const *const details) {
    return EranosCurrencyDetails_decimal_places(details);
}

/// The English name.
///
/// @param details The currency details
/// @param name Receives the English name, replacing what it held
static inline void eranosCurrencyDetailsName(EranosCurrencyDetails const *const details, EranosText *const name) {
    eranosDetailClear(name);
    EranosCurrencyDetails_name(details, name);
    eranosDetailFinish(name);
}

/// The ISO 4217 numeric code.
///
/// @param details The currency details
/// @return The ISO 4217 numeric code
[[nodiscard]] static inline int32_t eranosCurrencyDetailsNumericCode(EranosCurrencyDetails const *const details) {
    return EranosCurrencyDetails_numeric_code(details);
}

/// The currency of the ISO 4217 code `code`.
///
/// @param code The ISO 4217 code, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param details Receives the details on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for a code
/// the
///   library does not list, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosCurrencyDetailsOf(char const *const code, EranosCurrencyDetails **const details) {
    EranosCurrencyDetails_of_result const result = EranosCurrencyDetails_of(eranosDetailView(code));
    *details = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p details; destroying nullptr does nothing, as free() does.
///
/// @param details The currency details, which no call uses after this one, or nullptr
static inline void eranosCurrencyDetailsDestroy(EranosCurrencyDetails *const details) {
    if (nullptr != details) {
        EranosCurrencyDetails_destroy(details);
    }
}

/// A list of currency details records.
typedef struct EranosCurrencyDetailsList EranosCurrencyDetailsList;

/// How many records the list holds.
///
/// @param list The currency details list
/// @return The result
[[nodiscard]] static inline size_t eranosCurrencyDetailsListLength(EranosCurrencyDetailsList const *const list) {
    return EranosCurrencyDetailsList_len(list);
}

/// Whether the list holds no record.
///
/// @param list The currency details list
/// @return Whether it holds none
[[nodiscard]] static inline bool eranosCurrencyDetailsListIsEmpty(EranosCurrencyDetailsList const *const list) {
    return EranosCurrencyDetailsList_is_empty(list);
}

/// A copy of the record at `index`, absent past the end.
///
/// @param list The currency details list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or nullptr past the end
[[nodiscard]] static inline EranosCurrencyDetails *eranosCurrencyDetailsListGet(EranosCurrencyDetailsList const *const list, size_t const index) {
    return EranosCurrencyDetailsList_get(list, index);
}

/// Every currency of the ISO 4217 list that the library knows, by code.
///
/// @return The currency details list, which the caller destroys
[[nodiscard]] static inline EranosCurrencyDetailsList *eranosCurrencyDetailsListAll(void) { return EranosCurrencyDetailsList_all(); }

/// Destroys @p list; destroying nullptr does nothing, as free() does.
///
/// @param list The currency details list, which no call uses after this one, or nullptr
static inline void eranosCurrencyDetailsListDestroy(EranosCurrencyDetailsList *const list) {
    if (nullptr != list) {
        EranosCurrencyDetailsList_destroy(list);
    }
}

#endif // ERANOS_CURRENCY_H
