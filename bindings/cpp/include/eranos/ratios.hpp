/// @file eranos/ratios.hpp
/// Exchange rates, percentages, and parts: the ratios that convert and divide amounts.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_RATIOS_HPP
#define ERANOS_RATIOS_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/currency.hpp"
#include "eranos/detail/declarations.hpp"
#include "eranos/money.hpp"

#include <cstdint>
#include <string>
#include <string_view>
#include <utility>

namespace eranos {

namespace detail {

[[nodiscard]] inline ExchangeRate exchangeRateOf(std::string text) noexcept;

} // namespace detail

/// An exchange rate: how many units of one currency an amount of another is worth, a positive decimal.
class ExchangeRate final {
  public:
    /// The rate written @p text: "4.97".
    ///
    /// @param text The rate's text
    /// @return The rate
    /// @throws Error Of the kind ErrorKind::MONEY for text that is not a positive decimal
    [[nodiscard]] static ExchangeRate parse(std::string_view const text);

    /// @p amount converted at this rate into @p currency.
    ///
    /// @param amount The amount to convert
    /// @param currency The currency to convert it into
    /// @return The converted amount, rounded half away from zero
    /// @throws Error Of the kind ErrorKind::MONEY for an amount out of range
    [[nodiscard]] Money convert(Money const &amount, Currency const &currency) const;

    /// The rate as the library writes it: "4.97".
    ///
    /// @return The text
    [[nodiscard]] std::string const &toString() const noexcept { return text; }

    /// Whether two rates are written alike.
    ///
    /// @param left A rate
    /// @param right Another rate
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(ExchangeRate const &left, ExchangeRate const &right) noexcept { return left.text == right.text; }

    /// Whether two rates are written differently.
    ///
    /// @param left A rate
    /// @param right Another rate
    /// @return Whether they differ
    [[nodiscard]] friend bool operator!=(ExchangeRate const &left, ExchangeRate const &right) noexcept { return !(left == right); }

  private:
    friend ExchangeRate detail::exchangeRateOf(std::string text) noexcept;

    explicit ExchangeRate(std::string text) noexcept : text(std::move(text)) {}

    std::string text;
};

/// A percentage, in basis points: 1250 for 12.5 %. The library checks it when a call carries it.
struct Percentage final {
    /// The percentage written @p text: "12.5%".
    ///
    /// @param text The percentage's text
    /// @return The percentage
    /// @throws Error Of the kind ErrorKind::MONEY for text that is not a percentage
    [[nodiscard]] static Percentage parse(std::string_view const text);

    /// The basis points: 10000 for 100 %.
    std::int32_t basisPoints{};

    /// The percentage as the library writes it: "12.5%".
    ///
    /// @return The text
    /// @throws Error Of the kind ErrorKind::MONEY for a percentage out of range
    [[nodiscard]] std::string toString() const;

    /// Whether two percentages have one number of basis points.
    ///
    /// @param left A percentage
    /// @param right Another percentage
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(Percentage const left, Percentage const right) noexcept {
        return left.basisPoints == right.basisPoints;
    }

    /// Whether two percentages differ.
    ///
    /// @param left A percentage
    /// @param right Another percentage
    /// @return Whether they differ
    [[nodiscard]] friend constexpr bool operator!=(Percentage const left, Percentage const right) noexcept { return !(left == right); }
};

/// A number of parts, in hundredths of a part: 50 for half a share. The library checks them when a call carries them.
struct Parts final {
    /// The parts written @p text: "0.5".
    ///
    /// @param text The parts' text
    /// @return The parts
    /// @throws Error Of the kind ErrorKind::MONEY for text that is not a number of parts
    [[nodiscard]] static Parts parse(std::string_view const text);

    /// The hundredths: 100 for one part.
    std::int32_t hundredths{};

    /// The parts as the library writes them: "0.5".
    ///
    /// @return The text
    /// @throws Error Of the kind ErrorKind::MONEY for parts out of range
    [[nodiscard]] std::string toString() const;

    /// Whether two numbers of parts are equal.
    ///
    /// @param left Parts
    /// @param right Other parts
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(Parts const left, Parts const right) noexcept { return left.hundredths == right.hundredths; }

    /// Whether two numbers of parts differ.
    ///
    /// @param left Parts
    /// @param right Other parts
    /// @return Whether they differ
    [[nodiscard]] friend constexpr bool operator!=(Parts const left, Parts const right) noexcept { return !(left == right); }
};

inline ExchangeRate ExchangeRate::parse(std::string_view const text) { return ExchangeRate(detail::unwrap(detail::EranosExchangeRate::parse(text))); }

inline Money ExchangeRate::convert(Money const &amount, Currency const &currency) const {
    return detail::fromDetail(*detail::unwrap(detail::EranosExchangeRate::convert(text, *detail::toDetail(amount), currency.code())));
}

inline Percentage Percentage::parse(std::string_view const text) { return Percentage{detail::unwrap(detail::EranosPercentage::parse(text))}; }

inline std::string Percentage::toString() const { return detail::unwrap(detail::EranosPercentage::format(basisPoints)); }

inline Parts Parts::parse(std::string_view const text) { return Parts{detail::unwrap(detail::EranosParts::parse(text))}; }

inline std::string Parts::toString() const { return detail::unwrap(detail::EranosParts::format(hundredths)); }

} // namespace eranos

#endif // ERANOS_RATIOS_HPP
