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
    inline constexpr Money(std::int64_t const minorUnits, CurrencyCode currency) noexcept : minorUnits(minorUnits), currency(std::move(currency)) {}

    /// The amount written @p text in @p currency: "12.50", "12.5", or "-3".
    ///
    /// @param text The amount's text
    /// @param currency The currency
    /// @return The amount, or an error of the kind ErrorKind::MONEY for text that is not an amount of the currency, and ErrorKind::INVALID_TEXT for
    ///   text that is not UTF-8
    [[nodiscard]] static std::expected<Money, Error> parse(std::string_view const text, CurrencyCode const &currency) noexcept;

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

    /// The amount as the library writes it: "12.50 EUR".
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

// An amount's currency is one the library lists.
inline std::string Money::toString() const noexcept { return detail::known(detail::toDetail(*this)->format()); }

} // namespace eranos

#endif // ERANOS_MONEY_HPP
