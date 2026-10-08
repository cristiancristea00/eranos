/// @file eranos/currency_code.hpp
/// The currencies of the ISO 4217 list, with what the library knows of each.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_CURRENCY_CODE_HPP
#define ERANOS_CURRENCY_CODE_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/detail/declarations.hpp"
#include "eranos/error.hpp"

#include <compare>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <memory>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace eranos {

namespace detail {

[[nodiscard]] inline CurrencyCode currencyOf(std::unique_ptr<EranosCurrencyDetails> details) noexcept;
[[nodiscard]] inline std::unique_ptr<EranosMoney> toDetail(Money const &value) noexcept;

} // namespace detail

/// A currency of the ISO 4217 list, with what the library knows of it: only the library makes one, and it keeps the library's details, which an
/// amount in it crosses with, so no amount looks its currency up.
class CurrencyCode final {
  public:
    /// The currency of the ISO 4217 code @p code: "EUR".
    ///
    /// @param code The ISO 4217 code
    /// @return The currency, or an error of the kind ErrorKind::MONEY for a code the library does not list, and ErrorKind::INVALID_TEXT for text that
    ///   is not UTF-8
    [[nodiscard]] static std::expected<CurrencyCode, Error> of(std::string_view const code) noexcept;

    /// Every currency the library lists, by code.
    ///
    /// @return The currencies
    [[nodiscard]] static std::vector<CurrencyCode> all() noexcept;

    /// The ISO 4217 code: "EUR".
    ///
    /// @return The code
    [[nodiscard]] inline constexpr std::string const &code() const noexcept { return fields.code; }

    /// The English name: "Euro".
    ///
    /// @return The name
    [[nodiscard]] inline constexpr std::string const &name() const noexcept { return fields.name; }

    /// The ISO 4217 numeric code: 978 for the euro.
    ///
    /// @return The numeric code
    [[nodiscard]] inline constexpr std::int32_t numericCode() const noexcept { return fields.numericCode; }

    /// The number of decimal places of its amounts: 2 for the euro, 0 for the yen.
    ///
    /// @return The decimal places
    [[nodiscard]] inline constexpr std::uint8_t decimalPlaces() const noexcept { return fields.decimalPlaces; }

    /// Whether two currencies have one code.
    ///
    /// @param left A currency
    /// @param right Another currency
    /// @return Whether they are the same currency
    [[nodiscard]] friend inline constexpr bool operator==(CurrencyCode const &left, CurrencyCode const &right) noexcept {
        return left.fields.code == right.fields.code;
    }

    /// How two currencies order, by code.
    ///
    /// @param left A currency
    /// @param right Another currency
    /// @return Their order
    [[nodiscard]] friend inline constexpr std::strong_ordering operator<=>(CurrencyCode const &left, CurrencyCode const &right) noexcept {
        return left.fields.code <=> right.fields.code;
    }

  private:
    friend CurrencyCode detail::currencyOf(std::unique_ptr<detail::EranosCurrencyDetails> details) noexcept;
    friend std::unique_ptr<detail::EranosMoney> detail::toDetail(Money const &value) noexcept;

    /// What the library knows of the currency, and the library's details it read them from, which an amount in the currency crosses with.
    struct Fields final {
        std::string code;
        std::string name;
        std::int32_t numericCode;
        std::uint8_t decimalPlaces;
        std::shared_ptr<detail::EranosCurrencyDetails const> details;
    };

    inline explicit CurrencyCode(Fields fields) noexcept : fields(std::move(fields)) {}

    Fields fields;
};

inline std::expected<CurrencyCode, Error> CurrencyCode::of(std::string_view const code) noexcept {
    return detail::expectedOf(detail::EranosCurrencyDetails::of(code)).transform([](std::unique_ptr<detail::EranosCurrencyDetails> details) noexcept {
        return detail::currencyOf(std::move(details));
    });
}

inline std::vector<CurrencyCode> CurrencyCode::all() noexcept {
    auto const list = detail::EranosCurrencyDetailsList::all();
    std::vector<CurrencyCode> result;
    result.reserve(list->len());
    for (std::size_t index = 0; index < list->len(); ++index) {
        result.push_back(detail::currencyOf(list->get(index)));
    }
    return result;
}

} // namespace eranos

#endif // ERANOS_CURRENCY_CODE_HPP
