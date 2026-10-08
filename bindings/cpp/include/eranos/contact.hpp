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
#include <memory>
#include <string>
#include <string_view>
#include <utility>

namespace eranos {

namespace detail {

[[nodiscard]] inline constexpr Email emailOf(std::string text) noexcept;
[[nodiscard]] inline PhoneNumber fromDetail(EranosPhoneNumber const &value) noexcept;
[[nodiscard]] inline Iban fromDetail(EranosIban const &value) noexcept;
[[nodiscard]] inline Iban ibanOf(EranosPaymentAccountDetails const &value) noexcept;
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

/// A phone number, which the library keeps in the E.164 form, with the international form it derives from it. Only the library makes one, so the
/// two forms always belong together.
class PhoneNumber final {
  public:
    /// The phone number written @p text, checked and made canonical by the library, which derives its international form.
    ///
    /// @param text The text
    /// @return The phone number, or an error of the kind ErrorKind::CONTACT for text that is not one, and ErrorKind::INVALID_TEXT for text that is
    ///   not UTF-8
    [[nodiscard]] static std::expected<PhoneNumber, Error> parse(std::string_view const text) noexcept;

    /// The phone number as the library writes it, in the E.164 form: "+40712345678".
    ///
    /// @return The text
    [[nodiscard]] inline constexpr std::string const &toString() const noexcept { return fields.e164; }

    /// The number in the international form, grouped as its country groups it: "+40 712 345 678".
    ///
    /// @return The text
    [[nodiscard]] inline constexpr std::string const &international() const noexcept { return fields.international; }

    /// Whether two values are written alike.
    ///
    /// @param left A value
    /// @param right Another value
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(PhoneNumber const &left, PhoneNumber const &right) noexcept = default;

  private:
    friend PhoneNumber detail::fromDetail(detail::EranosPhoneNumber const &value) noexcept;

    /// The E.164 form and the international form.
    struct Fields final {
        std::string e164;
        std::string international;

        [[nodiscard]] friend constexpr bool operator==(Fields const &left, Fields const &right) noexcept = default;
    };

    inline constexpr explicit PhoneNumber(Fields fields) noexcept : fields(std::move(fields)) {}

    Fields fields;
};

/// An IBAN, which the library checks and keeps in its electronic form, with the country code it starts with. Only the library makes one, so the
/// two always belong together.
class Iban final {
  public:
    /// The IBAN written @p text, checked and made canonical by the library, which derives its country code.
    ///
    /// @param text The text
    /// @return The IBAN, or an error of the kind ErrorKind::PAYMENT_ACCOUNT for text that is not one, and ErrorKind::INVALID_TEXT for text that is
    ///   not UTF-8
    [[nodiscard]] static std::expected<Iban, Error> parse(std::string_view const text) noexcept;

    /// The IBAN as the library writes it, without spaces, in capitals: "RO49AAAA1B31007593840000".
    ///
    /// @return The text
    [[nodiscard]] inline constexpr std::string const &toString() const noexcept { return fields.iban; }

    /// The two-letter code of the IBAN's country: "RO".
    ///
    /// @return The country code
    [[nodiscard]] inline constexpr std::string const &countryCode() const noexcept { return fields.countryCode; }

    /// Whether two values are written alike.
    ///
    /// @param left A value
    /// @param right Another value
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(Iban const &left, Iban const &right) noexcept = default;

  private:
    friend Iban detail::fromDetail(detail::EranosIban const &value) noexcept;
    friend Iban detail::ibanOf(detail::EranosPaymentAccountDetails const &value) noexcept;

    /// The IBAN and its country code.
    struct Fields final {
        std::string iban;
        std::string countryCode;

        [[nodiscard]] friend constexpr bool operator==(Fields const &left, Fields const &right) noexcept = default;
    };

    inline constexpr explicit Iban(Fields fields) noexcept : fields(std::move(fields)) {}

    Fields fields;
};

/// A username in a payment app, trimmed by the library.
class Username final {
  public:
    /// The username written @p text, checked and made canonical by the library.
    ///
    /// @param text The text
    /// @return The username, or an error of the kind ErrorKind::PAYMENT_ACCOUNT for text that is not one, and ErrorKind::INVALID_TEXT for text that
    ///   is not UTF-8
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
    return detail::expectedOf(detail::EranosPhoneNumber::parse(text)).transform([](std::unique_ptr<detail::EranosPhoneNumber> const &value) noexcept {
        return detail::fromDetail(*value);
    });
}

inline std::expected<Iban, Error> Iban::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosIban::parse(text)).transform([](std::unique_ptr<detail::EranosIban> const &value) noexcept {
        return detail::fromDetail(*value);
    });
}

inline std::expected<Username, Error> Username::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosUsername::parse(text)).transform(detail::usernameOf);
}

inline std::expected<Language, Error> Language::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosLanguage::parse(text)).transform(detail::languageOf);
}

} // namespace eranos

#endif // ERANOS_CONTACT_HPP
