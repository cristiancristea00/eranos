/// @file eranos/money.hpp
/// Amounts: whole numbers of minor units of a currency.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_MONEY_HPP
#define ERANOS_MONEY_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/currency.hpp"
#include "eranos/detail/declarations.hpp"

#include <cstdint>
#include <memory>
#include <string>
#include <string_view>
#include <utility>

namespace eranos {

/// An amount: a whole number of minor units of a currency, cents for the euro.
struct Money final {
    /// The amount of @p minorUnits minor units of @p currency.
    ///
    /// @param currency The currency
    /// @param minorUnits The amount in minor units: cents for the euro, none for the yen
    Money(Currency currency, std::int64_t const minorUnits) noexcept : currency(std::move(currency)), minorUnits(minorUnits) {}

    /// The amount written @p text in @p currency: "12.50", "12.5", or "-3".
    ///
    /// @param text The amount's text
    /// @param currency The currency
    /// @return The amount
    /// @throws Error Of the kind ErrorKind::MONEY for text that is not an amount of the currency
    [[nodiscard]] static Money parse(std::string_view const text, Currency const &currency);

    /// The currency.
    Currency currency;

    /// The amount in minor units: cents for the euro, none for the yen.
    std::int64_t minorUnits;

    /// This amount plus @p other.
    ///
    /// @param other Another amount in the same currency
    /// @return The sum
    /// @throws Error Of the kind ErrorKind::MONEY for amounts in two currencies, or a sum out of range
    [[nodiscard]] Money plus(Money const &other) const;

    /// This amount minus @p other.
    ///
    /// @param other Another amount in the same currency
    /// @return The difference
    /// @throws Error Of the kind ErrorKind::MONEY for amounts in two currencies, or a difference out of range
    [[nodiscard]] Money minus(Money const &other) const;

    /// The amount as the library writes it: "12.50 EUR".
    ///
    /// @return The text
    [[nodiscard]] std::string toString() const;

    /// Whether two amounts have one currency and one number of minor units.
    ///
    /// @param left An amount
    /// @param right Another amount
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Money const &left, Money const &right) noexcept {
        return left.currency == right.currency && left.minorUnits == right.minorUnits;
    }

    /// Whether two amounts differ in currency or in minor units.
    ///
    /// @param left An amount
    /// @param right Another amount
    /// @return Whether they differ
    [[nodiscard]] friend bool operator!=(Money const &left, Money const &right) noexcept { return !(left == right); }
};

namespace detail {

[[nodiscard]] inline Money fromDetail(EranosMoney const &value);
[[nodiscard]] inline std::unique_ptr<EranosMoney> toDetail(Money const &value);

} // namespace detail

inline Money Money::parse(std::string_view const text, Currency const &currency) {
    return detail::fromDetail(*detail::unwrap(detail::EranosMoney::parse(text, currency.code())));
}

inline Money Money::plus(Money const &other) const {
    return detail::fromDetail(*detail::unwrap(detail::toDetail(*this)->add(*detail::toDetail(other))));
}

inline Money Money::minus(Money const &other) const {
    return detail::fromDetail(*detail::unwrap(detail::toDetail(*this)->subtract(*detail::toDetail(other))));
}

inline std::string Money::toString() const { return detail::unwrap(detail::toDetail(*this)->format()); }

} // namespace eranos

#endif // ERANOS_MONEY_HPP
