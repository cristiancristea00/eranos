/// @file eranos/error.hpp
/// The error that a call the library refuses returns.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_ERROR_HPP
#define ERANOS_ERROR_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/detail/declarations.hpp"
#include "eranos/failures.hpp"
#include "eranos/identifiers.hpp"

#include <cstdint>
#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <vector>

namespace eranos {

namespace detail {

[[nodiscard]] inline Error errorOf(EranosError const &detail) noexcept;

} // namespace detail

/// A failure a call reports, which the call returns in its std::expected: the module that refused it, the library's message, the failure within
/// the module, and what the failure holds, each present for the failures that hold it. Copying one copies no payload, so it cannot throw. What it
/// holds is defined with the conversions, after every part, since it holds amounts, whose calls return errors.
class Error final {
  public:
    /// The module that refused the call, or the core's own case.
    ///
    /// @return The kind
    [[nodiscard]] ErrorKind kind() const noexcept;

    /// The library's message: lowercase, without a full stop, and never the text it refused.
    ///
    /// @return The message
    [[nodiscard]] std::string const &message() const noexcept;

    /// The failure within its module, absent for the core's own cases.
    ///
    /// @return The failure
    [[nodiscard]] std::optional<Failure> failure() const noexcept;

    /// What the amounts or the items add up to. Present for the failures that hold it.
    ///
    /// @return The actual
    [[nodiscard]] std::optional<Money> const &actual() const noexcept;

    /// How many decimal places the currency has. Present for the failures that hold it.
    ///
    /// @return The allowed
    [[nodiscard]] std::optional<std::int64_t> const &allowed() const noexcept;

    /// How many open balances the plan has. Present for the failures that hold it.
    ///
    /// @return The balances
    [[nodiscard]] std::optional<std::int64_t> const &balances() const noexcept;

    /// The identifier of the category. Present for the failures that hold it.
    ///
    /// @return The category
    [[nodiscard]] std::optional<CategoryId> const &category() const noexcept;

    /// The column of the saved group where the text stops being one, from 1. Present for the failures that hold it.
    ///
    /// @return The column
    [[nodiscard]] std::optional<std::int64_t> const &column() const noexcept;

    /// The ISO 4217 code of the currency. Present for the failures that hold it.
    ///
    /// @return The currency
    [[nodiscard]] std::optional<std::string> const &currency() const noexcept;

    /// The category that has the name. Present for the failures that hold it.
    ///
    /// @return The existing
    [[nodiscard]] std::optional<CategoryId> const &existing() const noexcept;

    /// The amount being split, or the expense's amount. Present for the failures that hold it.
    ///
    /// @return The expected amount
    [[nodiscard]] std::optional<Money> const &expectedAmount() const noexcept;

    /// The ISO 4217 code of the expense's currency. Present for the failures that hold it.
    ///
    /// @return The expected currency
    [[nodiscard]] std::optional<std::string> const &expectedCurrency() const noexcept;

    /// How many characters the country's IBANs have. Present for the failures that hold it.
    ///
    /// @return The expected length
    [[nodiscard]] std::optional<std::int64_t> const &expectedLength() const noexcept;

    /// The identifier of the expense. Present for the failures that hold it.
    ///
    /// @return The expense
    [[nodiscard]] std::optional<ExpenseId> const &expense() const noexcept;

    /// The expenses that name the member. Present for the failures that hold it.
    ///
    /// @return The expenses
    [[nodiscard]] std::optional<std::vector<ExpenseId>> const &expenses() const noexcept;

    /// The format the saved group names. Present for the failures that hold it.
    ///
    /// @return The format
    [[nodiscard]] std::optional<std::int64_t> const &format() const noexcept;

    /// The ISO 4217 code of the currency of the amount inside the expense. Present for the failures that hold it.
    ///
    /// @return The found currency
    [[nodiscard]] std::optional<std::string> const &foundCurrency() const noexcept;

    /// How many characters the IBAN has. Present for the failures that hold it.
    ///
    /// @return The found length
    [[nodiscard]] std::optional<std::int64_t> const &foundLength() const noexcept;

    /// The ISO 4217 code of the first amount's currency. Present for the failures that hold it.
    ///
    /// @return The left
    [[nodiscard]] std::optional<std::string> const &left() const noexcept;

    /// The line of the saved group where the text stops being one, from 1. Present for the failures that hold it.
    ///
    /// @return The line
    [[nodiscard]] std::optional<std::int64_t> const &line() const noexcept;

    /// The identifier of the member. Present for the failures that hold it.
    ///
    /// @return The member
    [[nodiscard]] std::optional<MemberId> const &member() const noexcept;

    /// Why the IBAN was refused. Present for the failures that hold it.
    ///
    /// @return The reason
    [[nodiscard]] std::optional<IbanReason> const &reason() const noexcept;

    /// The ISO 4217 code of the second amount's currency. Present for the failures that hold it.
    ///
    /// @return The right
    [[nodiscard]] std::optional<std::string> const &right() const noexcept;

    /// What the percentages add up to, in basis points. Present for the failures that hold it.
    ///
    /// @return The total basis points
    [[nodiscard]] std::optional<std::int64_t> const &totalBasisPoints() const noexcept;

    /// The identifier of the transfer. Present for the failures that hold it.
    ///
    /// @return The transfer
    [[nodiscard]] std::optional<TransferId> const &transfer() const noexcept;

    /// The transfers that name the member. Present for the failures that hold it.
    ///
    /// @return The transfers
    [[nodiscard]] std::optional<std::vector<TransferId>> const &transfers() const noexcept;

  private:
    friend Error detail::errorOf(detail::EranosError const &detail) noexcept;

    /// What an error holds.
    struct Fields;

    inline explicit Error(std::shared_ptr<Fields const> fields) noexcept : fields(std::move(fields)) {}

    std::shared_ptr<Fields const> fields;
};

} // namespace eranos

#endif // ERANOS_ERROR_HPP
