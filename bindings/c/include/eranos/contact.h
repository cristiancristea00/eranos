/// @file eranos/contact.h
/// Email addresses, phone numbers, IBANs, usernames, and languages: text the library checks and makes canonical, and for a phone number and an IBAN,
/// objects that hold the text the library derives from them.

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

/// An IBAN, with the country code it starts with: an object only eranosIbanParse() makes, so it holds no text the library did not derive.
typedef struct EranosIban EranosIban;

/// The IBAN written `text`, without spaces, in capitals, with its country code.
///
/// @param text The text, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param iban Receives the IBAN on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_PAYMENT_ACCOUNT for
///   text that is not an IBAN, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosIbanParse(char const *const text, EranosIban **const iban) {
    EranosIban_parse_result const result = EranosIban_parse(eranosDetailView(text));
    *iban = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The IBAN, without spaces, in capitals: "RO49AAAA1B31007593840000".
///
/// @param iban The IBAN
/// @param text Receives the IBAN's text, replacing what it held
static inline void eranosIbanIban(EranosIban const *const iban, EranosText *const text) {
    eranosDetailClear(text);
    EranosIban_iban(iban, text);
    eranosDetailFinish(text);
}

/// The two-letter code of the IBAN's country: "RO".
///
/// @param iban The IBAN
/// @param countryCode Receives the country code, replacing what it held
static inline void eranosIbanCountryCode(EranosIban const *const iban, EranosText *const countryCode) {
    eranosDetailClear(countryCode);
    EranosIban_country_code(iban, countryCode);
    eranosDetailFinish(countryCode);
}

/// Destroys @p iban; destroying nullptr does nothing, as free() does.
///
/// @param iban The IBAN, which no call uses after this one, or nullptr
static inline void eranosIbanDestroy(EranosIban *const iban) {
    if (nullptr != iban) {
        EranosIban_destroy(iban);
    }
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

/// A phone number, in the two forms the library writes it in: an object only eranosPhoneNumberParse() and eranosMemberPhone() make, so it holds no
/// text the library did not derive.
typedef struct EranosPhoneNumber EranosPhoneNumber;

/// The phone number written `text`, in E.164 form and in international form.
///
/// @param text The text, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param phoneNumber Receives the phone number on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_CONTACT for text
///   that is not a phone number, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosPhoneNumberParse(char const *const text, EranosPhoneNumber **const phoneNumber) {
    EranosPhoneNumber_parse_result const result = EranosPhoneNumber_parse(eranosDetailView(text));
    *phoneNumber = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The number in E.164 form: "+40712345678".
///
/// @param phoneNumber The phone number
/// @param e164 Receives the number in E.164 form, replacing what it held
static inline void eranosPhoneNumberE164(EranosPhoneNumber const *const phoneNumber, EranosText *const e164) {
    eranosDetailClear(e164);
    EranosPhoneNumber_e164(phoneNumber, e164);
    eranosDetailFinish(e164);
}

/// The number in international form, grouped as its country groups it: "+40 712 345 678".
///
/// @param phoneNumber The phone number
/// @param international Receives the number in international form, replacing what it held
static inline void eranosPhoneNumberInternational(EranosPhoneNumber const *const phoneNumber, EranosText *const international) {
    eranosDetailClear(international);
    EranosPhoneNumber_international(phoneNumber, international);
    eranosDetailFinish(international);
}

/// Destroys @p phoneNumber; destroying nullptr does nothing, as free() does.
///
/// @param phoneNumber The phone number, which no call uses after this one, or nullptr
static inline void eranosPhoneNumberDestroy(EranosPhoneNumber *const phoneNumber) {
    if (nullptr != phoneNumber) {
        EranosPhoneNumber_destroy(phoneNumber);
    }
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
