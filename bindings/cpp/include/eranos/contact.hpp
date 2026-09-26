/// @file eranos/contact.hpp
/// Email addresses, phone numbers, IBANs, usernames, and languages: text the library checks and makes canonical.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_CONTACT_HPP
#define ERANOS_CONTACT_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/detail/declarations.hpp"

#include <string>
#include <string_view>
#include <utility>

namespace eranos {

namespace detail {

[[nodiscard]] inline Email emailOf(std::string text) noexcept;
[[nodiscard]] inline PhoneNumber phoneNumberOf(std::string text) noexcept;
[[nodiscard]] inline Iban ibanOf(std::string text) noexcept;
[[nodiscard]] inline Username usernameOf(std::string text) noexcept;
[[nodiscard]] inline Language languageOf(std::string text) noexcept;

} // namespace detail

/// An email address, checked by the library.
class Email final {
  public:
    /// The email address written @p text, checked and made canonical by the library.
    ///
    /// @param text The text
    /// @return The email address
    /// @throws Error Of the kind ErrorKind::CONTACT for text that is not one, and ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] static Email parse(std::string_view const text);

    /// The email address as the library writes it: "ana@example.com".
    ///
    /// @return The text
    [[nodiscard]] std::string const &toString() const noexcept { return text; }

    /// Whether two values are written alike.
    ///
    /// @param left A value
    /// @param right Another value
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Email const &left, Email const &right) noexcept = default;

  private:
    friend Email detail::emailOf(std::string text) noexcept;

    explicit Email(std::string text) noexcept : text(std::move(text)) {}

    std::string text;
};

/// A phone number, which the library keeps in the E.164 form.
class PhoneNumber final {
  public:
    /// The phone number written @p text, checked and made canonical by the library.
    ///
    /// @param text The text
    /// @return The phone number
    /// @throws Error Of the kind ErrorKind::CONTACT for text that is not one, and ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] static PhoneNumber parse(std::string_view const text);

    /// The phone number as the library writes it: "+40712345678".
    ///
    /// @return The text
    [[nodiscard]] std::string const &toString() const noexcept { return text; }

    /// The number in the international form: "+40 712 345 678".
    ///
    /// @return The text
    [[nodiscard]] std::string international() const;

    /// Whether two values are written alike.
    ///
    /// @param left A value
    /// @param right Another value
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(PhoneNumber const &left, PhoneNumber const &right) noexcept = default;

  private:
    friend PhoneNumber detail::phoneNumberOf(std::string text) noexcept;

    explicit PhoneNumber(std::string text) noexcept : text(std::move(text)) {}

    std::string text;
};

/// An IBAN, which the library checks and keeps in its electronic form.
class Iban final {
  public:
    /// The IBAN written @p text, checked and made canonical by the library.
    ///
    /// @param text The text
    /// @return The IBAN
    /// @throws Error Of the kind ErrorKind::ACCOUNT for text that is not one, and ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] static Iban parse(std::string_view const text);

    /// The IBAN as the library writes it: "RO49AAAA1B31007593840000".
    ///
    /// @return The text
    [[nodiscard]] std::string const &toString() const noexcept { return text; }

    /// The IBAN's country code: "RO".
    ///
    /// @return The country code
    [[nodiscard]] std::string countryCode() const;

    /// Whether two values are written alike.
    ///
    /// @param left A value
    /// @param right Another value
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Iban const &left, Iban const &right) noexcept = default;

  private:
    friend Iban detail::ibanOf(std::string text) noexcept;

    explicit Iban(std::string text) noexcept : text(std::move(text)) {}

    std::string text;
};

/// A username in a payment app, trimmed by the library.
class Username final {
  public:
    /// The username written @p text, checked and made canonical by the library.
    ///
    /// @param text The text
    /// @return The username
    /// @throws Error Of the kind ErrorKind::ACCOUNT for text that is not one, and ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] static Username parse(std::string_view const text);

    /// The username as the library writes it: "ana.pays".
    ///
    /// @return The text
    [[nodiscard]] std::string const &toString() const noexcept { return text; }

    /// Whether two values are written alike.
    ///
    /// @param left A value
    /// @param right Another value
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Username const &left, Username const &right) noexcept = default;

  private:
    friend Username detail::usernameOf(std::string text) noexcept;

    explicit Username(std::string text) noexcept : text(std::move(text)) {}

    std::string text;
};

/// A language, as a canonical BCP 47 tag.
class Language final {
  public:
    /// The language written @p text, checked and made canonical by the library.
    ///
    /// @param text The text
    /// @return The language
    /// @throws Error Of the kind ErrorKind::LANGUAGE for text that is not one, and ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] static Language parse(std::string_view const text);

    /// The language as the library writes it: "pt-PT".
    ///
    /// @return The text
    [[nodiscard]] std::string const &toString() const noexcept { return text; }

    /// Whether two values are written alike.
    ///
    /// @param left A value
    /// @param right Another value
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Language const &left, Language const &right) noexcept = default;

  private:
    friend Language detail::languageOf(std::string text) noexcept;

    explicit Language(std::string text) noexcept : text(std::move(text)) {}

    std::string text;
};

inline Email Email::parse(std::string_view const text) { return Email(detail::unwrap(detail::EranosEmail::parse(text))); }

inline PhoneNumber PhoneNumber::parse(std::string_view const text) { return PhoneNumber(detail::unwrap(detail::EranosPhoneNumber::parse(text))); }

inline std::string PhoneNumber::international() const { return detail::unwrap(detail::EranosPhoneNumber::international(text)); }

inline Iban Iban::parse(std::string_view const text) { return Iban(detail::unwrap(detail::EranosIban::parse(text))); }

inline std::string Iban::countryCode() const { return detail::unwrap(detail::EranosIban::country_code(text)); }

inline Username Username::parse(std::string_view const text) { return Username(detail::unwrap(detail::EranosUsername::parse(text))); }

inline Language Language::parse(std::string_view const text) { return Language(detail::unwrap(detail::EranosLanguage::parse(text))); }

} // namespace eranos

#endif // ERANOS_CONTACT_HPP
