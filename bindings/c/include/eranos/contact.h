/// @file eranos/contact.h
/// Email addresses, phone numbers, IBANs, usernames, and languages: text the library checks and makes canonical.

#ifndef ERANOS_CONTACT_H
#define ERANOS_CONTACT_H

#include "EranosEmail.h"
#include "EranosIban.h"
#include "EranosLanguage.h"
#include "EranosPhoneNumber.h"
#include "EranosUsername.h"
#include "eranos/detail.h"
#include "eranos/text.h"

/// Writes the email address written `text`, checked and trimmed.
///
/// @param text The text, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param email Receives the email, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_CONTACT for text
///   that is not an email address, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosEmailParse(char const *const text, EranosText *const email) {
    eranosDetailClear(email);
    EranosEmail_parse_result const result = EranosEmail_parse(eranosDetailView(text), email);
    eranosDetailFinish(email);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Writes the IBAN written `text` in its electronic form, without spaces.
///
/// @param text The text, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param iban Receives the iban, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_PAYMENT_ACCOUNT for
///   text that is not an IBAN, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosIbanParse(char const *const text, EranosText *const iban) {
    eranosDetailClear(iban);
    EranosIban_parse_result const result = EranosIban_parse(eranosDetailView(text), iban);
    eranosDetailFinish(iban);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Writes the country code of the IBAN written `text`: "RO".
///
/// @param text The text, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param countryCode Receives the country code, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_PAYMENT_ACCOUNT for
///   text that is not an IBAN, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosIbanCountryCode(char const *const text, EranosText *const countryCode) {
    eranosDetailClear(countryCode);
    EranosIban_country_code_result const result = EranosIban_country_code(eranosDetailView(text), countryCode);
    eranosDetailFinish(countryCode);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Writes the language written `text` as a canonical BCP 47 tag: "pt-PT".
///
/// @param text The text, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param language Receives the language, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_LANGUAGE for text
///   that is not a language tag, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosLanguageParse(char const *const text, EranosText *const language) {
    eranosDetailClear(language);
    EranosLanguage_parse_result const result = EranosLanguage_parse(eranosDetailView(text), language);
    eranosDetailFinish(language);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Writes the phone number written `text` in the E.164 form: "+40712345678".
///
/// @param text The text, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param phoneNumber Receives the phone number, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_CONTACT for text
///   that is not a phone number, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosPhoneNumberParse(char const *const text, EranosText *const phoneNumber) {
    eranosDetailClear(phoneNumber);
    EranosPhoneNumber_parse_result const result = EranosPhoneNumber_parse(eranosDetailView(text), phoneNumber);
    eranosDetailFinish(phoneNumber);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Writes the phone number written `text` in the international form: "+40 712 345 678".
///
/// @param text The text, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param international Receives the international, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_CONTACT for text
///   that is not a phone number, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosPhoneNumberInternational(char const *const text, EranosText *const international) {
    eranosDetailClear(international);
    EranosPhoneNumber_international_result const result = EranosPhoneNumber_international(eranosDetailView(text), international);
    eranosDetailFinish(international);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Writes the username written `text`, trimmed.
///
/// @param text The text, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param username Receives the username, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_PAYMENT_ACCOUNT for
///   a blank username, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosUsernameParse(char const *const text, EranosText *const username) {
    eranosDetailClear(username);
    EranosUsername_parse_result const result = EranosUsername_parse(eranosDetailView(text), username);
    eranosDetailFinish(username);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

#endif // ERANOS_CONTACT_H
