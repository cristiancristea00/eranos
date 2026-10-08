/// @file eranos/money.hpp
/// Amounts: whole numbers of minor units of a currency.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_MONEY_HPP
#define ERANOS_MONEY_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/currency_code.hpp"
#include "eranos/detail/declarations.hpp"
#include "eranos/error.hpp"

#include <cstdint>
#include <expected>
#include <memory>
#include <string>
#include <string_view>
#include <utility>

namespace eranos {

/// An amount: a whole number of minor units of a currency, cents for the euro.
struct Money final {
    /// The amount of @p minorUnits minor units of @p currency.
    ///
    /// @param minorUnits The amount in minor units: cents for the euro, none for the yen
    /// @param currency The currency
    inline Money(std::int64_t const minorUnits, CurrencyCode currency) noexcept : minorUnits(minorUnits), currency(std::move(currency)) {}

    /// The amount written @p text in @p currency: "12.50", "12.5", or "-3".
    ///
    /// @param text The amount's text
    /// @param currency The currency
    /// @return The amount, or an error of the kind ErrorKind::MONEY for text that is not an amount of the currency, and ErrorKind::INVALID_TEXT for
    ///   text that is not UTF-8
    [[nodiscard]] static std::expected<Money, Error> parse(std::string_view const text, CurrencyCode const &currency) noexcept;

    /// The amount of the double @p value in @p currency: 1.005 in EUR is 1.01 EUR. The library writes the double in its shortest decimal form,
    /// rounds it half away from zero to the currency's decimal places, and parses it, so every binding gets the same amount from the same double.
    ///
    /// @param value The amount, as a double
    /// @param currency The currency
    /// @return The amount, or an error of the kind ErrorKind::MONEY for a value that is NaN or infinite, or an amount out of range
    [[nodiscard]] static std::expected<Money, Error> fromDouble(double const value, CurrencyCode const &currency) noexcept;

    /// The amount in minor units: cents for the euro, none for the yen.
    std::int64_t minorUnits;

    /// The currency.
    CurrencyCode currency;

    /// This amount plus @p other.
    ///
    /// @param other Another amount in the same currency
    /// @return The sum, or an error of the kind ErrorKind::MONEY for amounts in two currencies, or a sum out of range
    [[nodiscard]] std::expected<Money, Error> plus(Money const &other) const noexcept;

    /// This amount minus @p other.
    ///
    /// @param other Another amount in the same currency
    /// @return The difference, or an error of the kind ErrorKind::MONEY for amounts in two currencies, or a difference out of range
    [[nodiscard]] std::expected<Money, Error> minus(Money const &other) const noexcept;

    /// The amount as the library writes it, with the decimal places of its currency: "12.50 EUR".
    ///
    /// @return The text
    [[nodiscard]] std::string toString() const noexcept;

    /// Whether two amounts have one number of minor units and one currency.
    ///
    /// @param left An amount
    /// @param right Another amount
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(Money const &left, Money const &right) noexcept = default;
};

namespace detail {

[[nodiscard]] inline Money fromDetail(EranosMoney const &value) noexcept;
[[nodiscard]] inline std::unique_ptr<EranosMoney> toDetail(Money const &value) noexcept;

} // namespace detail

inline std::expected<Money, Error> Money::parse(std::string_view const text, CurrencyCode const &currency) noexcept {
    return detail::expectedOf(detail::EranosMoney::parse(text, currency.code())).transform([](auto const &amount) noexcept {
        return detail::fromDetail(*amount);
    });
}

inline std::expected<Money, Error> Money::fromDouble(double const value, CurrencyCode const &currency) noexcept {
    return detail::expectedOf(detail::EranosMoney::create_from_double(value, detail::toDetail(currency))).transform([](auto const &amount) noexcept {
        return detail::fromDetail(*amount);
    });
}

inline std::expected<Money, Error> Money::plus(Money const &other) const noexcept {
    return detail::expectedOf(detail::toDetail(*this)->add(*detail::toDetail(other))).transform([](auto const &sum) noexcept {
        return detail::fromDetail(*sum);
    });
}

inline std::expected<Money, Error> Money::minus(Money const &other) const noexcept {
    return detail::expectedOf(detail::toDetail(*this)->subtract(*detail::toDetail(other))).transform([](auto const &difference) noexcept {
        return detail::fromDetail(*difference);
    });
}

inline std::string Money::toString() const noexcept { return detail::toDetail(*this)->format(); }

} // namespace eranos

#endif // ERANOS_MONEY_HPP
