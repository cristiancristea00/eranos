/// @file eranos/contact.hpp
/// Email addresses, phone numbers, IBANs, usernames, and languages: text the library checks and makes canonical.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_CONTACT_HPP
#define ERANOS_CONTACT_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/detail/declarations.hpp"
#include "eranos/error.hpp"

#include <expected>
#include <string>
#include <string_view>
#include <utility>

namespace eranos {

namespace detail {

[[nodiscard]] inline constexpr Email emailOf(std::string text) noexcept;
[[nodiscard]] inline constexpr PhoneNumber phoneNumberOf(std::string text) noexcept;
[[nodiscard]] inline constexpr Iban ibanOf(std::string text) noexcept;
[[nodiscard]] inline constexpr Username usernameOf(std::string text) noexcept;
[[nodiscard]] inline constexpr Language languageOf(std::string text) noexcept;

} // namespace detail

/// An email address, checked by the library.
class Email final {
  public:
    /// The email address written @p text, checked and made canonical by the library.
    ///
    /// @param text The text
    /// @return The email address, or an error of the kind ErrorKind::CONTACT for text that is not one, and ErrorKind::INVALID_TEXT for text that is
    ///   not UTF-8
    [[nodiscard]] static std::expected<Email, Error> parse(std::string_view const text) noexcept;

    /// The email address as the library writes it: "ana@example.com".
    ///
    /// @return The text
    [[nodiscard]] inline constexpr std::string const &toString() const noexcept { return text; }

    /// Whether two values are written alike.
    ///
    /// @param left A value
    /// @param right Another value
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(Email const &left, Email const &right) noexcept = default;

  private:
    friend constexpr Email detail::emailOf(std::string text) noexcept;

    inline constexpr explicit Email(std::string text) noexcept : text(std::move(text)) {}

    std::string text;
};

/// A phone number, which the library keeps in the E.164 form.
class PhoneNumber final {
  public:
    /// The phone number written @p text, checked and made canonical by the library.
    ///
    /// @param text The text
    /// @return The phone number, or an error of the kind ErrorKind::CONTACT for text that is not one, and ErrorKind::INVALID_TEXT for text that is
    ///   not UTF-8
    [[nodiscard]] static std::expected<PhoneNumber, Error> parse(std::string_view const text) noexcept;

    /// The phone number as the library writes it: "+40712345678".
    ///
    /// @return The text
    [[nodiscard]] inline constexpr std::string const &toString() const noexcept { return text; }

    /// The number in the international form: "+40 712 345 678".
    ///
    /// @return The text
    [[nodiscard]] std::string international() const noexcept;

    /// Whether two values are written alike.
    ///
    /// @param left A value
    /// @param right Another value
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(PhoneNumber const &left, PhoneNumber const &right) noexcept = default;

  private:
    friend constexpr PhoneNumber detail::phoneNumberOf(std::string text) noexcept;

    inline constexpr explicit PhoneNumber(std::string text) noexcept : text(std::move(text)) {}

    std::string text;
};

/// An IBAN, which the library checks and keeps in its electronic form.
class Iban final {
  public:
    /// The IBAN written @p text, checked and made canonical by the library.
    ///
    /// @param text The text
    /// @return The IBAN, or an error of the kind ErrorKind::PAYMENT_ACCOUNT for text that is not one, and ErrorKind::INVALID_TEXT for text that is
    /// not UTF-8
    [[nodiscard]] static std::expected<Iban, Error> parse(std::string_view const text) noexcept;

    /// The IBAN as the library writes it: "RO49AAAA1B31007593840000".
    ///
    /// @return The text
    [[nodiscard]] inline constexpr std::string const &toString() const noexcept { return text; }

    /// The IBAN's country code: "RO".
    ///
    /// @return The country code
    [[nodiscard]] std::string countryCode() const noexcept;

    /// Whether two values are written alike.
    ///
    /// @param left A value
    /// @param right Another value
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(Iban const &left, Iban const &right) noexcept = default;

  private:
    friend constexpr Iban detail::ibanOf(std::string text) noexcept;

    inline constexpr explicit Iban(std::string text) noexcept : text(std::move(text)) {}

    std::string text;
};

/// A username in a payment app, trimmed by the library.
class Username final {
  public:
    /// The username written @p text, checked and made canonical by the library.
    ///
    /// @param text The text
    /// @return The username, or an error of the kind ErrorKind::PAYMENT_ACCOUNT for text that is not one, and ErrorKind::INVALID_TEXT for text that
    /// is not
    ///   UTF-8
    [[nodiscard]] static std::expected<Username, Error> parse(std::string_view const text) noexcept;

    /// The username as the library writes it: "ana.pays".
    ///
    /// @return The text
    [[nodiscard]] inline constexpr std::string const &toString() const noexcept { return text; }

    /// Whether two values are written alike.
    ///
    /// @param left A value
    /// @param right Another value
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(Username const &left, Username const &right) noexcept = default;

  private:
    friend constexpr Username detail::usernameOf(std::string text) noexcept;

    inline constexpr explicit Username(std::string text) noexcept : text(std::move(text)) {}

    std::string text;
};

/// A language, as a canonical BCP 47 tag.
class Language final {
  public:
    /// The language written @p text, checked and made canonical by the library.
    ///
    /// @param text The text
    /// @return The language, or an error of the kind ErrorKind::LANGUAGE for text that is not one, and ErrorKind::INVALID_TEXT for text that is not
    ///   UTF-8
    [[nodiscard]] static std::expected<Language, Error> parse(std::string_view const text) noexcept;

    /// The language as the library writes it: "pt-PT".
    ///
    /// @return The text
    [[nodiscard]] inline constexpr std::string const &toString() const noexcept { return text; }

    /// Whether two values are written alike.
    ///
    /// @param left A value
    /// @param right Another value
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(Language const &left, Language const &right) noexcept = default;

  private:
    friend constexpr Language detail::languageOf(std::string text) noexcept;

    inline constexpr explicit Language(std::string text) noexcept : text(std::move(text)) {}

    std::string text;
};

inline std::expected<Email, Error> Email::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosEmail::parse(text)).transform(detail::emailOf);
}

inline std::expected<PhoneNumber, Error> PhoneNumber::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosPhoneNumber::parse(text)).transform(detail::phoneNumberOf);
}

// A phone number is one the library checked.
inline std::string PhoneNumber::international() const noexcept { return detail::known(detail::EranosPhoneNumber::international(text)); }

inline std::expected<Iban, Error> Iban::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosIban::parse(text)).transform(detail::ibanOf);
}

// An IBAN is one the library checked.
inline std::string Iban::countryCode() const noexcept { return detail::known(detail::EranosIban::country_code(text)); }

inline std::expected<Username, Error> Username::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosUsername::parse(text)).transform(detail::usernameOf);
}

inline std::expected<Language, Error> Language::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosLanguage::parse(text)).transform(detail::languageOf);
}

} // namespace eranos

#endif // ERANOS_CONTACT_HPP
