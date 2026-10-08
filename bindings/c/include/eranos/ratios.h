/// @file eranos/ratios.h
/// Exchange rates, percentages, and parts: the ratios that convert and divide amounts.

#ifndef ERANOS_RATIOS_H
#define ERANOS_RATIOS_H

#include "EranosExchangeRate.h"
#include "EranosParts.h"
#include "EranosPercentage.h"
#include "eranos/detail.h"
#include "eranos/text.h"

#include <stdint.h>

/// Writes the exchange rate written `text` as the library writes it: "4.97".
///
/// @param text The text, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param exchangeRate Receives the exchange rate, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for text that
///   is not a positive decimal, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExchangeRateParse(char const *const text, EranosText *const exchangeRate) {
    eranosDetailClear(exchangeRate);
    EranosExchangeRate_parse_result const result = EranosExchangeRate_parse(eranosDetailView(text), exchangeRate);
    eranosDetailFinish(exchangeRate);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Writes the exchange rate of the double @p value as the library writes it: 4.9765 is "4.9765". The library writes the double in its shortest
/// decimal form, rounds it half away from zero to 18 decimal places, and parses it, so the rate keeps the digits of that form: 25.5 is "25.5".
///
/// @param value The rate, as a double
/// @param exchangeRate Receives the exchange rate, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for a value
///   that is NaN or infinite, or that is not a positive rate at 18 decimal places
[[nodiscard]] static inline EranosError *eranosExchangeRateFromDouble(double const value, EranosText *const exchangeRate) {
    eranosDetailClear(exchangeRate);
    EranosExchangeRate_from_double_result const result = EranosExchangeRate_from_double(value, exchangeRate);
    eranosDetailFinish(exchangeRate);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The amount `amount` converted at the rate `rate` into the currency `currency`.
///
/// @param rate The rate, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param amount The amount
/// @param currency The currency, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param converted Receives the converted on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for a rate
///   that is not one, a currency the library does not list, or an amount out of range, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that
///   is not UTF-8
[[nodiscard]] static inline EranosError *eranosExchangeRateConvert(char const *const rate, EranosMoney const *const amount,
                                                                   char const *const currency, EranosMoney **const converted) {
    EranosExchangeRate_convert_result const result = EranosExchangeRate_convert(eranosDetailView(rate), amount, eranosDetailView(currency));
    *converted = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The parts written `text`, in hundredths: "0.5" is 50.
///
/// @param text The text, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param hundredths Receives the hundredths on success
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for text that
///   is not a number of parts, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosPartsParse(char const *const text, int32_t *const hundredths) {
    EranosParts_parse_result const result = EranosParts_parse(eranosDetailView(text));
    *hundredths = result.is_ok ? result.ok : 0;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The parts of the double @p value, in hundredths: 1.005 is 101. The library writes the double in its shortest decimal form, rounds it half away
/// from zero to two decimal places, and parses it.
///
/// @param value The parts, as a double
/// @param hundredths Receives the hundredths on success
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for a value
///   that is NaN or infinite, rounds to zero or below, or is out of range
[[nodiscard]] static inline EranosError *eranosPartsFromDouble(double const value, int32_t *const hundredths) {
    EranosParts_from_double_result const result = EranosParts_from_double(value);
    *hundredths = result.is_ok ? result.ok : 0;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Writes the parts of `hundredths` hundredths as the library does: "0.5". It writes zero and a negative number, which eranosPartsCheck() refuses,
/// the same way: "0" and "-0.5".
///
/// @param hundredths The hundredths
/// @param text Receives the text, replacing what it held
static inline void eranosPartsFormat(int32_t const hundredths, EranosText *const text) {
    eranosDetailClear(text);
    EranosParts_format(hundredths, text);
    eranosDetailFinish(text);
}

/// Checks that `hundredths` is a number of parts, as a split of parts requires.
///
/// @param hundredths The hundredths
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for a number
///   that is not positive
[[nodiscard]] static inline EranosError *eranosPartsCheck(int32_t const hundredths) {
    EranosParts_check_result const result = EranosParts_check(hundredths);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The percentage written `text`, in basis points: "12.5%" is 1250.
///
/// @param text The text, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param basisPoints Receives the basis points on success
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for text that
///   is not a percentage, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosPercentageParse(char const *const text, int32_t *const basisPoints) {
    EranosPercentage_parse_result const result = EranosPercentage_parse(eranosDetailView(text));
    *basisPoints = result.is_ok ? result.ok : 0;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The percentage of the double @p value, a number of per cent, in basis points: 12.345 is 1235. The library writes the double in its shortest
/// decimal form, rounds it half away from zero to two decimal places, and parses it, so `0.1 + 0.2`, which is 0.30000000000000004 as a double, is
/// 30.
///
/// @param value The percentage, as a double
/// @param basisPoints Receives the basis points on success
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for a value
///   that is NaN or infinite, negative, or out of range
[[nodiscard]] static inline EranosError *eranosPercentageFromDouble(double const value, int32_t *const basisPoints) {
    EranosPercentage_from_double_result const result = EranosPercentage_from_double(value);
    *basisPoints = result.is_ok ? result.ok : 0;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Writes the percentage of `basisPoints` basis points as the library does: "12.5%". It writes a negative number, which eranosPercentageCheck()
/// refuses, the same way after a minus sign: "-12.5%".
///
/// @param basisPoints The basis points
/// @param text Receives the text, replacing what it held
static inline void eranosPercentageFormat(int32_t const basisPoints, EranosText *const text) {
    eranosDetailClear(text);
    EranosPercentage_format(basisPoints, text);
    eranosDetailFinish(text);
}

/// Checks that `basisPoints` is a percentage, as an entry that holds one requires.
///
/// @param basisPoints The basis points
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for a
///   negative number
[[nodiscard]] static inline EranosError *eranosPercentageCheck(int32_t const basisPoints) {
    EranosPercentage_check_result const result = EranosPercentage_check(basisPoints);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

#endif // ERANOS_RATIOS_H
