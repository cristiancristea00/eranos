/// @file eranos/expense.hpp
/// Expenses: what was bought, what it cost, who paid, and how it divides, with their costs, items, and tips.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_EXPENSE_HPP
#define ERANOS_EXPENSE_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/calendar.hpp"
#include "eranos/detail/declarations.hpp"
#include "eranos/error.hpp"
#include "eranos/identifiers.hpp"
#include "eranos/money.hpp"
#include "eranos/ratios.hpp"
#include "eranos/split.hpp"

#include <cstdint>
#include <expected>
#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <variant>
#include <vector>

namespace eranos {

/// How a tip is shared among the participants.
enum class TipDistribution : std::uint8_t {
    /// In proportion to each participant's share of the amount.
    PROPORTIONAL,
    /// The same for every participant.
    EQUAL,
};

/// A line of an itemised bill: `quantity` units of `name` at `unit_price`, which `split` divides.
struct ExpenseItem final {
    /// The record of its required fields; the others start absent, empty, or at their default.
    ///
    /// @param name What was bought.
    /// @param unitPrice The price of one unit, in the expense's currency.
    /// @param split How the line's total divides.
    ExpenseItem(std::string name, Money unitPrice, Split split) noexcept
        : name(std::move(name)), unitPrice(std::move(unitPrice)), split(std::move(split)) {}

    /// What was bought.
    std::string name;

    /// The price of one unit, in the expense's currency.
    Money unitPrice;

    /// How the line's total divides.
    Split split;

    /// How many units, from 1.
    std::int32_t quantity{1};

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(ExpenseItem const &left, ExpenseItem const &right) noexcept = default;
};

/// A percentage of the expense's amount.
struct PercentageTipValue final {
    /// The tip's share of the amount, in basis points.
    Percentage percentage;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(PercentageTipValue const &left, PercentageTipValue const &right) noexcept = default;
};

/// A fixed amount in the expense's currency.
struct FixedTipValue final {
    /// The tip.
    Money amount;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(FixedTipValue const &left, FixedTipValue const &right) noexcept = default;
};

/// How a tip is expressed. One of its alternatives: PercentageTipValue, FixedTipValue.
using TipValue = std::variant<PercentageTipValue, FixedTipValue>;

/// A tip on top of an expense's amount.
struct TipRule final {
    /// The record of its fields.
    ///
    /// @param value How the tip is expressed.
    /// @param distribution How the tip is shared among the participants.
    TipRule(TipValue value, TipDistribution const distribution) noexcept : value(std::move(value)), distribution(distribution) {}

    /// How the tip is expressed.
    TipValue value;

    /// How the tip is shared among the participants.
    TipDistribution distribution;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(TipRule const &left, TipRule const &right) noexcept = default;
};

/// An amount in the group currency.
struct LocalCost final {
    /// The amount.
    Money amount;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(LocalCost const &left, LocalCost const &right) noexcept = default;
};

/// An amount in another currency than the group's, converted at `rate`.
struct ConvertedCost final {
    /// The amount, in its own currency.
    Money amount;
    /// The rate into the group currency, as a decimal.
    ExchangeRate rate;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(ConvertedCost const &left, ConvertedCost const &right) noexcept = default;
};

/// What an entry cost: an amount in the group currency, or one in another, converted at a rate. One of its alternatives: LocalCost, ConvertedCost.
using Cost = std::variant<LocalCost, ConvertedCost>;

/// The whole amount is divided by one split.
struct WholeExpenseSplit final {
    /// How the amount divides.
    Split split;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(WholeExpenseSplit const &left, WholeExpenseSplit const &right) noexcept = default;
};

/// Every line is divided by its own split; the lines add up to the amount.
struct ItemisedExpenseSplit final {
    /// The lines of the bill.
    std::vector<ExpenseItem> items;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(ItemisedExpenseSplit const &left, ItemisedExpenseSplit const &right) noexcept = default;
};

/// How an expense's amount is shared. One of its alternatives: WholeExpenseSplit, ItemisedExpenseSplit.
using ExpenseSplit = std::variant<WholeExpenseSplit, ItemisedExpenseSplit>;

/// An expense: what was bought, what it cost, who paid, and how it divides.
struct Expense final {
    /// The record of its required fields; the others start absent, empty, or at their default.
    ///
    /// @param name What was bought.
    /// @param cost What it cost.
    /// @param payers Who paid the total, amount and tip, and how much each.
    /// @param split How the amount is shared.
    /// @param date The day it was bought.
    Expense(std::string name, Cost cost, Split payers, ExpenseSplit split, CalendarDate const date) noexcept
        : name(std::move(name)), cost(std::move(cost)), payers(std::move(payers)), split(std::move(split)), date(date) {}

    /// The identifier: absent for an expense to add, present for one the group holds.
    std::optional<ExpenseId> id;

    /// What was bought.
    std::string name;

    /// What it cost.
    Cost cost;

    /// Who paid the total, amount and tip, and how much each.
    Split payers;

    /// How the amount is shared.
    ExpenseSplit split;

    /// The day it was bought.
    CalendarDate date;

    /// The tip on top of the amount.
    std::optional<TipRule> tip;

    /// The time of day it was bought.
    std::optional<TimeOfDay> time;

    /// The category's identifier; absent for the built-in one.
    std::optional<CategoryId> category;

    /// A longer description.
    std::optional<std::string> description;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Expense const &left, Expense const &right) noexcept = default;
};

/// The tip of an expense in its currency, zero without one.
///
/// @param expense The expense
/// @return The tip, or an error of the kind ErrorKind::EXPENSE for a tip the expense cannot have
[[nodiscard]] std::expected<Money, Error> tipAmount(Expense const &expense);

/// The amount of an expense with its tip, what its payers paid.
///
/// @param expense The expense
/// @return The total, or an error of the kind ErrorKind::EXPENSE for a tip the expense cannot have
[[nodiscard]] std::expected<Money, Error> total(Expense const &expense);

/// Every member an expense names, payers and participants, each once and in the order of their identifiers.
///
/// @param expense The expense
/// @return The members, or an error of the kind ErrorKind::ID for an identifier that is not one
[[nodiscard]] std::expected<std::vector<MemberId>, Error> participants(Expense const &expense);

namespace detail {

[[nodiscard]] inline ExpenseItem fromDetail(EranosExpenseItem const &value);
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpenseItem>, Error> toDetail(ExpenseItem const &value);
[[nodiscard]] inline TipValue fromDetail(EranosTipValue const &value);
[[nodiscard]] inline std::unique_ptr<EranosTipValue> toDetail(PercentageTipValue const &value);
[[nodiscard]] inline std::unique_ptr<EranosTipValue> toDetail(FixedTipValue const &value);
[[nodiscard]] inline std::unique_ptr<EranosTipValue> toDetail(TipValue const &value);
[[nodiscard]] inline TipRule fromDetail(EranosTipRule const &value);
[[nodiscard]] inline std::unique_ptr<EranosTipRule> toDetail(TipRule const &value);
[[nodiscard]] inline Cost fromDetail(EranosCost const &value);
[[nodiscard]] inline std::unique_ptr<EranosCost> toDetail(LocalCost const &value);
[[nodiscard]] inline std::unique_ptr<EranosCost> toDetail(ConvertedCost const &value);
[[nodiscard]] inline std::unique_ptr<EranosCost> toDetail(Cost const &value);
[[nodiscard]] inline ExpenseSplit fromDetail(EranosExpenseSplit const &value);
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpenseSplit>, Error> toDetail(WholeExpenseSplit const &value);
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpenseSplit>, Error> toDetail(ItemisedExpenseSplit const &value);
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpenseSplit>, Error> toDetail(ExpenseSplit const &value);
[[nodiscard]] inline Expense fromDetail(EranosExpense const &value);
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpense>, Error> toDetail(Expense const &value);
[[nodiscard]] inline std::vector<Expense> fromDetail(EranosExpenseList const &list);
[[nodiscard]] inline std::vector<ExpenseItem> fromDetail(EranosExpenseItemList const &list);
[[nodiscard]] constexpr TipDistribution fromDetail(EranosTipDistribution const value) noexcept;
[[nodiscard]] constexpr EranosTipDistribution toDetail(TipDistribution const value) noexcept;

} // namespace detail

inline std::expected<Money, Error> tipAmount(Expense const &expense) {
    return detail::toDetail(expense)
        .and_then([](std::unique_ptr<detail::EranosExpense> const &converted) { return detail::expectedOf(converted->tip_amount()); })
        .transform([](std::unique_ptr<detail::EranosMoney> const &amount) { return detail::fromDetail(*amount); });
}

inline std::expected<Money, Error> total(Expense const &expense) {
    return detail::toDetail(expense)
        .and_then([](std::unique_ptr<detail::EranosExpense> const &converted) { return detail::expectedOf(converted->total()); })
        .transform([](std::unique_ptr<detail::EranosMoney> const &amount) { return detail::fromDetail(*amount); });
}

inline std::expected<std::vector<MemberId>, Error> participants(Expense const &expense) {
    return detail::toDetail(expense)
        .and_then([](std::unique_ptr<detail::EranosExpense> const &converted) { return detail::expectedOf(converted->participants()); })
        .transform([](std::unique_ptr<detail::EranosTextList> const &members) { return detail::identifiers<MemberId>(*members); });
}

} // namespace eranos

#endif // ERANOS_EXPENSE_HPP
