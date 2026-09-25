/// @file eranos/error.hpp
/// The error that a call the library refuses throws.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_ERROR_HPP
#define ERANOS_ERROR_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/detail/declarations.hpp"
#include "eranos/failures.hpp"
#include "eranos/identifiers.hpp"
#include "eranos/money.hpp"

#include <cstdint>
#include <memory>
#include <optional>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace eranos {

namespace detail {

[[nodiscard]] inline Error errorOf(EranosError const &detail);

} // namespace detail

/// A failure a call reports: the module that refused it, the failure within the module, the library's message, and what the failure holds, each
/// present for the failures that hold it. Copying one copies no payload, so it cannot throw.
class Error final : public std::runtime_error {
  public:
    /// The module that refused the call, or the core's own case.
    ///
    /// @return The kind
    [[nodiscard]] ErrorKind kind() const noexcept { return fields->kind; }

    /// The failure within its module, absent for the core's own cases.
    ///
    /// @return The failure
    [[nodiscard]] std::optional<Failure> failure() const noexcept { return fields->failure; }

    /// What the amounts or the items add up to. Present for the failures that hold it.
    ///
    /// @return The actual
    [[nodiscard]] std::optional<Money> const &actual() const noexcept { return fields->actual; }

    /// How many decimal places the currency has. Present for the failures that hold it.
    ///
    /// @return The allowed
    [[nodiscard]] std::optional<std::int64_t> const &allowed() const noexcept { return fields->allowed; }

    /// How many open balances the plan has. Present for the failures that hold it.
    ///
    /// @return The balances
    [[nodiscard]] std::optional<std::int64_t> const &balances() const noexcept { return fields->balances; }

    /// The identifier of the category. Present for the failures that hold it.
    ///
    /// @return The category
    [[nodiscard]] std::optional<CategoryId> const &category() const noexcept { return fields->category; }

    /// The column of the saved group where the text stops being one, from 1. Present for the failures that hold it.
    ///
    /// @return The column
    [[nodiscard]] std::optional<std::int64_t> const &column() const noexcept { return fields->column; }

    /// The ISO 4217 code of the currency. Present for the failures that hold it.
    ///
    /// @return The currency
    [[nodiscard]] std::optional<std::string> const &currency() const noexcept { return fields->currency; }

    /// The category that has the name. Present for the failures that hold it.
    ///
    /// @return The existing
    [[nodiscard]] std::optional<CategoryId> const &existing() const noexcept { return fields->existing; }

    /// The amount being split, or the expense's amount. Present for the failures that hold it.
    ///
    /// @return The expected amount
    [[nodiscard]] std::optional<Money> const &expectedAmount() const noexcept { return fields->expectedAmount; }

    /// The ISO 4217 code of the expense's currency. Present for the failures that hold it.
    ///
    /// @return The expected currency
    [[nodiscard]] std::optional<std::string> const &expectedCurrency() const noexcept { return fields->expectedCurrency; }

    /// How many characters the country's IBANs have. Present for the failures that hold it.
    ///
    /// @return The expected length
    [[nodiscard]] std::optional<std::int64_t> const &expectedLength() const noexcept { return fields->expectedLength; }

    /// The identifier of the expense. Present for the failures that hold it.
    ///
    /// @return The expense
    [[nodiscard]] std::optional<ExpenseId> const &expense() const noexcept { return fields->expense; }

    /// The expenses that name the member. Present for the failures that hold it.
    ///
    /// @return The expenses
    [[nodiscard]] std::optional<std::vector<ExpenseId>> const &expenses() const noexcept { return fields->expenses; }

    /// The format the saved group names. Present for the failures that hold it.
    ///
    /// @return The format
    [[nodiscard]] std::optional<std::int64_t> const &format() const noexcept { return fields->format; }

    /// The ISO 4217 code of the currency of the amount inside the expense. Present for the failures that hold it.
    ///
    /// @return The found currency
    [[nodiscard]] std::optional<std::string> const &foundCurrency() const noexcept { return fields->foundCurrency; }

    /// How many characters the IBAN has. Present for the failures that hold it.
    ///
    /// @return The found length
    [[nodiscard]] std::optional<std::int64_t> const &foundLength() const noexcept { return fields->foundLength; }

    /// The ISO 4217 code of the first amount's currency. Present for the failures that hold it.
    ///
    /// @return The left
    [[nodiscard]] std::optional<std::string> const &left() const noexcept { return fields->left; }

    /// The line of the saved group where the text stops being one, from 1. Present for the failures that hold it.
    ///
    /// @return The line
    [[nodiscard]] std::optional<std::int64_t> const &line() const noexcept { return fields->line; }

    /// The identifier of the member. Present for the failures that hold it.
    ///
    /// @return The member
    [[nodiscard]] std::optional<MemberId> const &member() const noexcept { return fields->member; }

    /// Why the IBAN was refused. Present for the failures that hold it.
    ///
    /// @return The reason
    [[nodiscard]] std::optional<IbanReason> const &reason() const noexcept { return fields->reason; }

    /// The ISO 4217 code of the second amount's currency. Present for the failures that hold it.
    ///
    /// @return The right
    [[nodiscard]] std::optional<std::string> const &right() const noexcept { return fields->right; }

    /// What the percentages add up to, in basis points. Present for the failures that hold it.
    ///
    /// @return The total basis points
    [[nodiscard]] std::optional<std::int64_t> const &totalBasisPoints() const noexcept { return fields->totalBasisPoints; }

    /// The identifier of the transfer. Present for the failures that hold it.
    ///
    /// @return The transfer
    [[nodiscard]] std::optional<TransferId> const &transfer() const noexcept { return fields->transfer; }

    /// The transfers that name the member. Present for the failures that hold it.
    ///
    /// @return The transfers
    [[nodiscard]] std::optional<std::vector<TransferId>> const &transfers() const noexcept { return fields->transfers; }

  private:
    friend Error detail::errorOf(detail::EranosError const &detail);

    /// What an error holds besides its message.
    struct Fields final {
        ErrorKind kind;
        std::optional<Failure> failure;
        std::optional<Money> actual;
        std::optional<std::int64_t> allowed;
        std::optional<std::int64_t> balances;
        std::optional<CategoryId> category;
        std::optional<std::int64_t> column;
        std::optional<std::string> currency;
        std::optional<CategoryId> existing;
        std::optional<Money> expectedAmount;
        std::optional<std::string> expectedCurrency;
        std::optional<std::int64_t> expectedLength;
        std::optional<ExpenseId> expense;
        std::optional<std::vector<ExpenseId>> expenses;
        std::optional<std::int64_t> format;
        std::optional<std::string> foundCurrency;
        std::optional<std::int64_t> foundLength;
        std::optional<std::string> left;
        std::optional<std::int64_t> line;
        std::optional<MemberId> member;
        std::optional<IbanReason> reason;
        std::optional<std::string> right;
        std::optional<std::int64_t> totalBasisPoints;
        std::optional<TransferId> transfer;
        std::optional<std::vector<TransferId>> transfers;
    };

    Error(std::string const &message, std::shared_ptr<Fields const> fields) : std::runtime_error(message), fields(std::move(fields)) {}

    std::shared_ptr<Fields const> fields;
};

} // namespace eranos

#endif // ERANOS_ERROR_HPP
