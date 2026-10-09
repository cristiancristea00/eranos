/// @file eranos/ratios.hpp
/// Exchange rates, percentages, parts, and quantities: the decimals that convert, divide, and multiply amounts.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_RATIOS_HPP
#define ERANOS_RATIOS_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/currency_code.hpp"
#include "eranos/detail/declarations.hpp"
#include "eranos/error.hpp"
#include "eranos/money.hpp"

#include <cstdint>
#include <expected>
#include <memory>
#include <span>
#include <string>
#include <string_view>
#include <utility>

namespace eranos {

namespace detail {

[[nodiscard]] inline constexpr ExchangeRate exchangeRateOf(std::string text) noexcept;

} // namespace detail

/// An exchange rate: how many units of one currency an amount of another is worth, a positive decimal.
class ExchangeRate final {
  public:
    /// The rate written @p text: "4.97".
    ///
    /// @param text The rate's text
    /// @return The rate, or an error of the kind ErrorKind::MONEY for text that is not a positive decimal, and ErrorKind::INVALID_TEXT for text that
    ///   is not UTF-8
    [[nodiscard]] static std::expected<ExchangeRate, Error> parse(std::string_view const text) noexcept;

    /// The rate of the double @p value: 4.9765 is "4.9765". The library writes the double in its shortest decimal form, rounds it half away from
    /// zero to 18 decimal places, and parses it, so the rate keeps the digits of that form: 25.5 is "25.5".
    ///
    /// @param value The rate, as a double
    /// @return The rate, or an error of the kind ErrorKind::MONEY for a value that is NaN or infinite, or that is not a positive rate at 18 decimal
    ///   places
    [[nodiscard]] static std::expected<ExchangeRate, Error> fromDouble(double const value) noexcept;

    /// @p amount converted at this rate into @p currency.
    ///
    /// @param amount The amount to convert
    /// @param currency The currency to convert it into
    /// @return The converted amount, rounded half away from zero, or an error of the kind ErrorKind::MONEY for an amount out of range
    [[nodiscard]] std::expected<Money, Error> convert(Money const &amount, CurrencyCode const &currency) const noexcept;

    /// The rate as the library writes it: "4.97".
    ///
    /// @return The text
    [[nodiscard]] inline constexpr std::string const &toString() const noexcept { return text; }

    /// Whether two rates are written alike.
    ///
    /// @param left A rate
    /// @param right Another rate
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(ExchangeRate const &left, ExchangeRate const &right) noexcept = default;

  private:
    friend constexpr ExchangeRate detail::exchangeRateOf(std::string text) noexcept;

    inline constexpr explicit ExchangeRate(std::string text) noexcept : text(std::move(text)) {}

    std::string text;
};

/// A percentage, in basis points: 1250 for 12.5 %. The library checks it when a call carries it.
struct Percentage final {
    /// The percentage written @p text: "12.5%".
    ///
    /// @param text The percentage's text
    /// @return The percentage, or an error of the kind ErrorKind::MONEY for text that is not a percentage, and ErrorKind::INVALID_TEXT for text that
    ///   is not UTF-8
    [[nodiscard]] static std::expected<Percentage, Error> parse(std::string_view const text) noexcept;

    /// The percentage of the double @p value, a number of per cent: 12.345 is 12.35 %. The library writes the double in its shortest decimal form,
    /// rounds it half away from zero to two decimal places, and parses it, so 0.1 + 0.2, which is 0.30000000000000004 as a double, is 0.3 %.
    ///
    /// @param value The percentage, as a double
    /// @return The percentage, or an error of the kind ErrorKind::MONEY for a value that is NaN or infinite, negative, or out of range
    [[nodiscard]] static std::expected<Percentage, Error> fromDouble(double const value) noexcept;

    /// The basis points: 10000 for 100 %.
    std::int32_t basisPoints{};

    /// The percentage as the library writes it: "12.5%". A negative number of basis points, which the library refuses when a call carries it, is
    /// written the same way after a minus sign: "-12.5%".
    ///
    /// @return The text
    [[nodiscard]] std::string toString() const noexcept;

    /// Whether two percentages have one number of basis points.
    ///
    /// @param left A percentage
    /// @param right Another percentage
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(Percentage const &left, Percentage const &right) noexcept = default;
};

/// A number of parts, in hundredths of a part: 50 for half a share. The library checks them when a call carries them.
struct Parts final {
    /// The parts written @p text: "0.5".
    ///
    /// @param text The parts' text
    /// @return The parts, or an error of the kind ErrorKind::MONEY for text that is not a number of parts, and ErrorKind::INVALID_TEXT for text that
    ///   is not UTF-8
    [[nodiscard]] static std::expected<Parts, Error> parse(std::string_view const text) noexcept;

    /// The parts of the double @p value: 1.005 is 1.01 parts, 101 hundredths. The library writes the double in its shortest decimal form, rounds it
    /// half away from zero to two decimal places, and parses it.
    ///
    /// @param value The parts, as a double
    /// @return The parts, or an error of the kind ErrorKind::MONEY for a value that is NaN or infinite, rounds to zero or below, or is out of range
    [[nodiscard]] static std::expected<Parts, Error> fromDouble(double const value) noexcept;

    /// The hundredths: 100 for one part.
    std::int32_t hundredths{};

    /// The parts as the library writes them: "0.5". Zero and a negative number of hundredths, which the library refuses when a call carries them,
    /// are written the same way: "0" and "-0.5".
    ///
    /// @return The text
    [[nodiscard]] std::string toString() const noexcept;

    /// Whether two numbers of parts are equal.
    ///
    /// @param left Parts
    /// @param right Other parts
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(Parts const &left, Parts const &right) noexcept = default;
};

/// A quantity of an item, in ten-thousandths of a unit: 10000 for one unit, 4520 for 0.452. The library checks it when a call carries it.
struct Quantity final {
    /// The quantity written @p text: "0.452".
    ///
    /// @param text The quantity's text
    /// @return The quantity, or an error of the kind ErrorKind::MONEY for text that is not a positive decimal with at most four decimal places, or a
    ///   quantity out of range, and ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] static std::expected<Quantity, Error> parse(std::string_view const text) noexcept;

    /// The quantity of @p units whole units: 3 is 30000 ten-thousandths.
    ///
    /// @param units The units
    /// @return The quantity, or an error of the kind ErrorKind::MONEY for a number that is not positive, or out of range
    [[nodiscard]] static std::expected<Quantity, Error> whole(std::int64_t const units) noexcept;

    /// The quantity of the double @p value: 0.45205 is 0.4521, 4521 ten-thousandths. The library writes the double in its shortest decimal form,
    /// rounds it half away from zero to four decimal places, and parses it.
    ///
    /// @param value The quantity, as a double
    /// @return The quantity, or an error of the kind ErrorKind::MONEY for a value that is NaN or infinite, rounds to zero or below, or is out of
    ///   range
    [[nodiscard]] static std::expected<Quantity, Error> fromDouble(double const value) noexcept;

    /// The ten-thousandths: 10000, one unit, until it is given another.
    std::int64_t tenThousandths{10'000};

    /// The quantity as the library writes it: "0.452". Zero and a negative number of ten-thousandths, which the library refuses when a call carries
    /// them, are written the same way: "0" and "-0.452".
    ///
    /// @return The text
    [[nodiscard]] std::string toString() const noexcept;

    /// Whether two quantities have one number of ten-thousandths.
    ///
    /// @param left A quantity
    /// @param right Another quantity
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(Quantity const &left, Quantity const &right) noexcept = default;
};

inline std::expected<ExchangeRate, Error> ExchangeRate::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosExchangeRate::parse(text)).transform(detail::exchangeRateOf);
}

inline std::expected<ExchangeRate, Error> ExchangeRate::fromDouble(double const value) noexcept {
    return detail::expectedOf(detail::EranosExchangeRate::from_double(value)).transform(detail::exchangeRateOf);
}

inline std::expected<Money, Error> ExchangeRate::convert(Money const &amount, CurrencyCode const &currency) const noexcept {
    return detail::expectedOf(detail::EranosExchangeRate::convert(text, *detail::toDetail(amount), currency.code()))
        .transform([](auto const &converted) noexcept { return detail::fromDetail(*converted); });
}

inline std::expected<Percentage, Error> Percentage::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosPercentage::parse(text)).transform([](std::int32_t const basisPoints) noexcept {
        return Percentage{basisPoints};
    });
}

inline std::expected<Percentage, Error> Percentage::fromDouble(double const value) noexcept {
    return detail::expectedOf(detail::EranosPercentage::from_double(value)).transform([](std::int32_t const basisPoints) noexcept {
        return Percentage{basisPoints};
    });
}

inline std::string Percentage::toString() const noexcept { return detail::EranosPercentage::format(basisPoints); }

inline std::expected<Parts, Error> Parts::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosParts::parse(text)).transform([](std::int32_t const hundredths) noexcept { return Parts{hundredths}; });
}

inline std::expected<Parts, Error> Parts::fromDouble(double const value) noexcept {
    return detail::expectedOf(detail::EranosParts::from_double(value)).transform([](std::int32_t const hundredths) noexcept {
        return Parts{hundredths};
    });
}

inline std::string Parts::toString() const noexcept { return detail::EranosParts::format(hundredths); }

inline std::expected<Quantity, Error> Quantity::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosQuantity::parse(text)).transform([](std::int64_t const tenThousandths) noexcept {
        return Quantity{tenThousandths};
    });
}

inline std::expected<Quantity, Error> Quantity::whole(std::int64_t const units) noexcept {
    return detail::expectedOf(detail::EranosQuantity::from_whole(units)).transform([](std::int64_t const tenThousandths) noexcept {
        return Quantity{tenThousandths};
    });
}

inline std::expected<Quantity, Error> Quantity::fromDouble(double const value) noexcept {
    return detail::expectedOf(detail::EranosQuantity::from_double(value)).transform([](std::int64_t const tenThousandths) noexcept {
        return Quantity{tenThousandths};
    });
}

inline std::string Quantity::toString() const noexcept { return detail::EranosQuantity::format(tenThousandths); }

/// The rate into a new group currency of an amount in another.
struct CurrencyRate final {
    /// The record of its fields.
    ///
    /// @param currency The ISO 4217 code of the currency the rate converts from.
    /// @param rate The rate, as a decimal.
    inline CurrencyRate(CurrencyCode currency, ExchangeRate rate) noexcept : currency(std::move(currency)), rate(std::move(rate)) {}

    /// The ISO 4217 code of the currency the rate converts from.
    CurrencyCode currency;

    /// The rate, as a decimal.
    ExchangeRate rate;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(CurrencyRate const &left, CurrencyRate const &right) noexcept = default;
};

namespace detail {

[[nodiscard]] inline std::unique_ptr<EranosCurrencyRate> toDetail(CurrencyRate const &value) noexcept;
[[nodiscard]] inline std::unique_ptr<EranosCurrencyRateList> toDetail(std::span<CurrencyRate const> const records) noexcept;

} // namespace detail

} // namespace eranos

#endif // ERANOS_RATIOS_HPP
