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
    /// The same for every participant.
    EQUAL,
    /// In proportion to each participant's share of the amount.
    PROPORTIONAL,
};

/// A line of an itemised bill: `quantity` units of `name` at `unit_price`, which `split` divides.
struct Item final {
    /// The record of its required fields; the others start absent, empty, or at their default.
    ///
    /// @param name What was bought.
    /// @param split How the line's total divides.
    /// @param unitPrice The price of one unit, in the expense's currency.
    Item(std::string name, Split split, Money unitPrice) noexcept : name(std::move(name)), split(std::move(split)), unitPrice(std::move(unitPrice)) {}

    /// What was bought.
    std::string name;

    /// How the line's total divides.
    Split split;

    /// The price of one unit, in the expense's currency.
    Money unitPrice;

    /// How many units, from 1.
    std::int32_t quantity{1};

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Item const &left, Item const &right) noexcept = default;
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

/// How a tip is expressed. One of its alternatives: FixedTipValue, PercentageTipValue.
using TipValue = std::variant<FixedTipValue, PercentageTipValue>;

/// A tip on top of an expense's amount.
struct Tip final {
    /// The record of its fields.
    ///
    /// @param distribution How the tip is shared among the participants.
    /// @param value How the tip is expressed.
    Tip(TipDistribution const distribution, TipValue value) noexcept : distribution(distribution), value(std::move(value)) {}

    /// How the tip is shared among the participants.
    TipDistribution distribution;

    /// How the tip is expressed.
    TipValue value;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Tip const &left, Tip const &right) noexcept = default;
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

/// What an entry cost: an amount in the group currency, or one in another, converted at a rate. One of its alternatives: ConvertedCost, LocalCost.
using Cost = std::variant<ConvertedCost, LocalCost>;

/// Every line is divided by its own split; the lines add up to the amount.
struct ItemisedExpenseSplit final {
    /// The lines of the bill.
    std::vector<Item> items;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(ItemisedExpenseSplit const &left, ItemisedExpenseSplit const &right) noexcept = default;
};

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

/// How an expense's amount is shared. One of its alternatives: ItemisedExpenseSplit, WholeExpenseSplit.
using ExpenseSplit = std::variant<ItemisedExpenseSplit, WholeExpenseSplit>;

/// An expense: what was bought, what it cost, who paid, and how it divides.
struct Expense final {
    /// The record of its required fields; the others start absent, empty, or at their default.
    ///
    /// @param cost What it cost.
    /// @param date The day it was bought.
    /// @param name What was bought.
    /// @param payers Who paid the total, amount and tip, and how much each.
    /// @param split How the amount is shared.
    Expense(Cost cost, CalendarDate const date, std::string name, Split payers, ExpenseSplit split) noexcept
        : cost(std::move(cost)), date(date), name(std::move(name)), payers(std::move(payers)), split(std::move(split)) {}

    /// The identifier: absent for an expense to add, present for one the group holds.
    std::optional<ExpenseId> id;

    /// What it cost.
    Cost cost;

    /// The day it was bought.
    CalendarDate date;

    /// What was bought.
    std::string name;

    /// Who paid the total, amount and tip, and how much each.
    Split payers;

    /// How the amount is shared.
    ExpenseSplit split;

    /// The category's identifier; absent for the built-in one.
    std::optional<CategoryId> category;

    /// A longer description.
    std::optional<std::string> description;

    /// The time of day it was bought.
    std::optional<TimeOfDay> time;

    /// The tip on top of the amount.
    std::optional<Tip> tip;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Expense const &left, Expense const &right) noexcept = default;
};

/// Every member an expense names, payers and participants, each once and in the order of their identifiers.
///
/// @param expense The expense
/// @return The members, or an error of the kind ErrorKind::ID for an identifier that is not one
[[nodiscard]] std::expected<std::vector<MemberId>, Error> participants(Expense const &expense);

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

namespace detail {

[[nodiscard]] inline Item fromDetail(EranosItem const &value);
[[nodiscard]] inline std::expected<std::unique_ptr<EranosItem>, Error> toDetail(Item const &value);
[[nodiscard]] inline TipValue fromDetail(EranosTipValue const &value);
[[nodiscard]] inline std::unique_ptr<EranosTipValue> toDetail(FixedTipValue const &value);
[[nodiscard]] inline std::unique_ptr<EranosTipValue> toDetail(PercentageTipValue const &value);
[[nodiscard]] inline std::unique_ptr<EranosTipValue> toDetail(TipValue const &value);
[[nodiscard]] inline Tip fromDetail(EranosTip const &value);
[[nodiscard]] inline std::unique_ptr<EranosTip> toDetail(Tip const &value);
[[nodiscard]] inline Cost fromDetail(EranosCost const &value);
[[nodiscard]] inline std::unique_ptr<EranosCost> toDetail(ConvertedCost const &value);
[[nodiscard]] inline std::unique_ptr<EranosCost> toDetail(LocalCost const &value);
[[nodiscard]] inline std::unique_ptr<EranosCost> toDetail(Cost const &value);
[[nodiscard]] inline ExpenseSplit fromDetail(EranosExpenseSplit const &value);
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpenseSplit>, Error> toDetail(ItemisedExpenseSplit const &value);
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpenseSplit>, Error> toDetail(WholeExpenseSplit const &value);
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpenseSplit>, Error> toDetail(ExpenseSplit const &value);
[[nodiscard]] inline Expense fromDetail(EranosExpense const &value);
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpense>, Error> toDetail(Expense const &value);
[[nodiscard]] inline std::vector<Expense> fromDetail(EranosExpenseList const &list);
[[nodiscard]] inline std::vector<Item> fromDetail(EranosItemList const &list);
[[nodiscard]] constexpr TipDistribution fromDetail(EranosTipDistribution const value) noexcept;
[[nodiscard]] constexpr EranosTipDistribution toDetail(TipDistribution const value) noexcept;

} // namespace detail

inline std::expected<std::vector<MemberId>, Error> participants(Expense const &expense) {
    return detail::toDetail(expense)
        .and_then([](std::unique_ptr<detail::EranosExpense> const &converted) { return detail::expectedOf(converted->participants()); })
        .transform([](std::unique_ptr<detail::EranosTextList> const &members) { return detail::identifiers<MemberId>(*members); });
}

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

} // namespace eranos

#endif // ERANOS_EXPENSE_HPP
