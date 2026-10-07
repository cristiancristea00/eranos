// Every way to divide an expense: equally, by percentage, by parts (including half a part for a child), by exact
// amounts, and with several payers. The port of the Rust example `split_methods`, which prints the same report.

#include "eranos.hpp"
#include "report.hpp"

#include <algorithm>
#include <cstdlib>
#include <map>
#include <print>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace eranos::examples {
namespace {

/// A row of the table of an expense's shares: a member's rule, what they paid, and what they consumed.
struct ShareRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<4> HEADERS{"Member", "Rule", "Paid", "Consumed"};

    /// Their first name.
    std::string name;
    /// The rule by which they share the expense.
    std::string rule;
    /// What they paid.
    std::string paid;
    /// What they consumed.
    std::string consumed;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<4> cells() const noexcept { return {name, rule, paid, consumed}; }
};

/// A row of the table of totals: what a member paid and consumed over the holiday, and their balance.
struct TotalRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<4> HEADERS{"Member", "Paid", "Consumed", "Balance"};

    /// Their first name.
    std::string name;
    /// What they paid.
    std::string paid;
    /// What they consumed.
    std::string consumed;
    /// What is left: what they paid minus what they consumed.
    std::string balance;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<4> cells() const noexcept { return {name, paid, consumed, balance}; }
};

/// The amount @p text of euros.
[[nodiscard]] Money euros(std::string_view const text) { return must(Money::parse(text, must(Currency::of("EUR")))); }

/// The family and the day every expense happened.
struct Family final {
    /// The family of @p ana, @p bob, and @p mia, whose expenses happened on @p day.
    ///
    /// @param ana The first parent
    /// @param bob The second parent
    /// @param mia Their daughter
    /// @param day The day of every expense
    Family(MemberId ana, MemberId bob, MemberId mia, CalendarDate const day) noexcept
        : ana(std::move(ana)), bob(std::move(bob)), mia(std::move(mia)), day(day) {}

    /// The first parent.
    MemberId ana;
    /// The second parent.
    MemberId bob;
    /// Their daughter.
    MemberId mia;
    /// The day of every expense.
    CalendarDate day;
};

/// The rule by which each member shares an expense, as the report names it.
using Rules = std::map<MemberId, std::string>;

/// The rule of an expense shared equally by the three.
[[nodiscard]] Rules thirds(Family const &family) {
    return {{family.ana, "an equal third"}, {family.bob, "an equal third"}, {family.mia, "an equal third"}};
}

/// The three members of @p family, sharing equally.
[[nodiscard]] EqualSplit everyone(Family const &family) { return EqualSplit{{family.ana, family.bob, family.mia}}; }

/// Prints how the expense @p id divides: every involved member's rule, what they paid, and what they consumed.
void report(ExpenseGroup const &group, ExpenseId const &id, Rules const &rules) {
    auto const expense = must(group.expense(id));
    auto const shares = must(group.expenseShares(id));
    if (!expense.has_value() || !shares.has_value()) {
        return;
    }
    std::println("{}, {}:", expense->name, shares->total.toString());
    std::vector<ShareRow> rows;
    rows.reserve(shares->members.size());
    for (auto const &share : shares->members) {
        auto const rule = rules.find(share.member);
        rows.push_back({
            .name = firstName(group, share.member),
            .rule = rules.end() == rule ? "(payer only)" : rule->second,
            .paid = share.paid.toString(),
            .consumed = share.share.toString(),
        });
    }
    printTable(rows, 2);
}

/// Adds the taxi, which the three share equally, the leftover cent going to one of them.
void equally(ExpenseGroup &group, Family const &family) {
    section("1. Equally");
    printLine("Everyone pays the same. 10.00 does not divide by three, so one member pays the\n"
              "leftover cent; which member it is rotates from expense to expense, so over a trip\n"
              "nobody always pays it.");
    auto const taxi =
        must(group.addExpense(Expense("Taxi", LocalCost{euros("10.00")}, EqualSplit{{family.ana}}, WholeExpenseSplit{everyone(family)}, family.day)));
    report(group, taxi, thirds(family));
}

/// Adds the flat, which the three share by percentage.
void byPercentage(ExpenseGroup &group, Family const &family) {
    section("2. By percentage");
    printLine("Everyone pays a percentage of the amount; the percentages must add up to exactly\n"
              "100 %. Mia's room is smaller, so she takes 20 %.");
    std::vector<MemberPercentage> const rooms{
        MemberPercentage(family.ana, must(Percentage::parse("40"))),
        MemberPercentage(family.bob, must(Percentage::parse("40"))),
        MemberPercentage(family.mia, must(Percentage::parse("20"))),
    };
    auto const rent = must(group.addExpense(
        Expense("Flat", LocalCost{euros("500.00")}, EqualSplit{{family.bob}}, WholeExpenseSplit{PercentagesSplit{rooms}}, family.day)));
    report(group, rent, {{family.ana, "40%"}, {family.bob, "40%"}, {family.mia, "20%"}});
}

/// Adds the food, which the three share by parts, Mia counting as half an adult.
[[nodiscard]] ExpenseId byParts(ExpenseGroup &group, Family const &family) {
    section("3. By parts");
    printLine("Everyone pays in proportion to their parts. Mia, a child, counts as half an adult:\n"
              "2.5 parts in total, so one part is 125.00 / 2.5 = 50.00.");
    std::vector<MemberParts> const portions{
        MemberParts(family.ana, must(Parts::parse("1"))),
        MemberParts(family.bob, must(Parts::parse("1"))),
        MemberParts(family.mia, must(Parts::parse("0.5"))),
    };
    auto const food = must(
        group.addExpense(Expense("Food", LocalCost{euros("125.00")}, EqualSplit{{family.ana}}, WholeExpenseSplit{PartsSplit{portions}}, family.day)));
    report(group, food, {{family.ana, "1 part"}, {family.bob, "1 part"}, {family.mia, "0.5 parts"}});
    return food;
}

/// Adds the museum tickets, which the three share by exact amounts.
void byAmounts(ExpenseGroup &group, Family const &family) {
    section("4. By exact amounts");
    printLine("Everyone pays a fixed amount, such as a ticket price; the amounts must add up to\n"
              "exactly the expense amount.");
    std::vector<MemberAmount> const prices{
        MemberAmount(family.ana, euros("15.00")),
        MemberAmount(family.bob, euros("15.00")),
        MemberAmount(family.mia, euros("5.00")),
    };
    auto const tickets = must(group.addExpense(
        Expense("Museum", LocalCost{euros("35.00")}, EqualSplit{{family.bob}}, WholeExpenseSplit{AmountsSplit{prices}}, family.day)));
    report(group, tickets, {{family.ana, "15.00 EUR"}, {family.bob, "15.00 EUR"}, {family.mia, "5.00 EUR"}});
}

/// Adds the dinner, which two of them paid and the three share equally.
void severalPayers(ExpenseGroup &group, Family const &family) {
    section("5. Several payers");
    printLine("The dinner bill went on two cards: Ana paid 50.00 and Bob 40.00. The payers are\n"
              "divided with the same methods as the participants.");
    AmountsSplit const cards{{MemberAmount(family.ana, euros("50.00")), MemberAmount(family.bob, euros("40.00"))}};
    auto const dinner = must(group.addExpense(Expense("Dinner", LocalCost{euros("90.00")}, cards, WholeExpenseSplit{everyone(family)}, family.day)));
    report(group, dinner, thirds(family));
}

/// Shows that a split whose percentages do not add up to 100 % is refused.
void mistakes(ExpenseGroup &group, Family const &family) {
    section("6. Mistakes are refused");
    printLine("A percentage split that adds up to 90 % is refused when the expense is built:");
    std::vector<MemberPercentage> const ninety{
        MemberPercentage(family.ana, must(Percentage::parse("60"))),
        MemberPercentage(family.bob, must(Percentage::parse("30"))),
    };
    auto const refused = refusal(group.addExpense(
        Expense("Invalid", LocalCost{euros("10.00")}, EqualSplit{{family.ana}}, WholeExpenseSplit{PercentagesSplit{ninety}}, family.day)));
    check(refused.has_value(), "the split is refused");
    std::println("  error: {}", message(refused));
}

/// Prints what each member paid and consumed over the holiday, and their balance.
void totals(ExpenseGroup const &group) {
    section("7. The holiday in total");
    auto const summary = group.summary();
    std::println("{} spent over five expenses:", summary.totalSpent.toString());
    std::vector<TotalRow> rows;
    for (auto const &member : group.members()) {
        auto const found = totalsOf(summary, member.id.value()).value();
        rows.push_back({
            .name = member.firstName,
            .paid = found.paid.toString(),
            .consumed = found.share.toString(),
            .balance = toString(found.balance),
        });
    }
    printTable(rows, 1);
}

/// Runs the example.
void run() {
    auto group = must(ExpenseGroup::create("Family holiday", must(Currency::of("EUR"))));
    auto const ana = must(group.addMember(Member("Ana")));
    auto const bob = must(group.addMember(Member("Bob")));
    auto const mia = must(group.addMember(Member("Mia")));
    Family const family(ana, bob, mia, must(CalendarDate::of(2026, 7, 14)));

    printLine("Family holiday: every way to split an expense\n"
              "═════════════════════════════════════════════");
    printLine("Ana, Bob, and their daughter Mia share a holiday. Each expense below uses a\n"
              "different split method; whatever the method, the shares add up to exactly the\n"
              "amount, to the cent.");
    equally(group, family);
    byPercentage(group, family);
    auto const food = byParts(group, family);
    byAmounts(group, family);
    severalPayers(group, family);
    mistakes(group, family);
    totals(group);

    // Parts: 125.00 over 2.5 parts is 50.00 per part.
    auto const shares = must(group.expenseShares(food)).value();
    auto const mias = std::ranges::find_if(shares.members, [&](MemberShare const &share) { return family.mia == share.member; });
    check(shares.members.end() != mias && euros("25.00") == mias->share, "Mia consumed 25.00 EUR of the food");
}

} // namespace
} // namespace eranos::examples

int main() {
    eranos::examples::run();
    return EXIT_SUCCESS;
}
