/// @file eranos/currency.hpp
/// The currencies of the ISO 4217 list, with what the library knows of each.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_CURRENCY_HPP
#define ERANOS_CURRENCY_HPP

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

[[nodiscard]] inline Currency currencyOf(EranosCurrencyDetails const &details);
[[nodiscard]] inline Currency listedCurrency(std::string_view const code);

} // namespace detail

/// A currency of the ISO 4217 list, with what the library knows of it: only the library's list makes one, so an amount
/// in it always formats.
class Currency final {
  public:
    /// The currency of the ISO 4217 code @p code: "EUR".
    ///
    /// @param code The ISO 4217 code
    /// @return The currency, or an error of the kind ErrorKind::MONEY for a code the library does not list, and ErrorKind::INVALID_TEXT for text that
    ///   is not UTF-8
    [[nodiscard]] static std::expected<Currency, Error> of(std::string_view const code);

    /// Every currency the library lists, by code.
    ///
    /// @return The currencies
    [[nodiscard]] static std::vector<Currency> all();

    /// The ISO 4217 code: "EUR".
    ///
    /// @return The code
    [[nodiscard]] std::string const &code() const noexcept { return fields.code; }

    /// The number of decimal places of its amounts: 2 for the euro, 0 for the yen.
    ///
    /// @return The decimal places
    [[nodiscard]] std::int32_t decimalPlaces() const noexcept { return fields.decimalPlaces; }

    /// The English name: "Euro".
    ///
    /// @return The name
    [[nodiscard]] std::string const &name() const noexcept { return fields.name; }

    /// The ISO 4217 numeric code: 978 for the euro.
    ///
    /// @return The numeric code
    [[nodiscard]] std::int32_t numericCode() const noexcept { return fields.numericCode; }

    /// Whether two currencies have one code.
    ///
    /// @param left A currency
    /// @param right Another currency
    /// @return Whether they are the same currency
    [[nodiscard]] friend bool operator==(Currency const &left, Currency const &right) noexcept { return left.fields.code == right.fields.code; }

    /// How two currencies order, by code.
    ///
    /// @param left A currency
    /// @param right Another currency
    /// @return Their order
    [[nodiscard]] friend std::strong_ordering operator<=>(Currency const &left, Currency const &right) noexcept {
        return left.fields.code <=> right.fields.code;
    }

  private:
    friend Currency detail::currencyOf(detail::EranosCurrencyDetails const &details);

    /// What the library knows of the currency.
    struct Fields final {
        std::string code;
        std::int32_t decimalPlaces;
        std::string name;
        std::int32_t numericCode;
    };

    explicit Currency(Fields fields) noexcept : fields(std::move(fields)) {}

    Fields fields;
};

inline std::expected<Currency, Error> Currency::of(std::string_view const code) {
    return detail::expectedOf(detail::EranosCurrencyDetails::of(code)).transform([](std::unique_ptr<detail::EranosCurrencyDetails> const &details) {
        return detail::currencyOf(*details);
    });
}

inline std::vector<Currency> Currency::all() {
    auto const list = detail::EranosCurrencyDetailsList::all();
    std::vector<Currency> result;
    result.reserve(list->len());
    for (std::size_t index = 0; index < list->len(); ++index) {
        result.push_back(detail::currencyOf(*list->get(index)));
    }
    return result;
}

} // namespace eranos

#endif // ERANOS_CURRENCY_HPP
